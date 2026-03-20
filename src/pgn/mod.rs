use std::fmt;

use tree_sitter::{Node, Parser};

use crate::game::StandardGame;
use crate::r#move::Move;

#[cfg(test)]
mod tests;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum PgnError {
    ParseError(String),
    InvalidMove {
        move_number: u32,
        san: String,
        reason: String,
    },
    InvalidResult(String),
}

impl fmt::Display for PgnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PgnError::ParseError(msg) => write!(f, "PGN parse error: {}", msg),
            PgnError::InvalidMove {
                move_number,
                san,
                reason,
            } => write!(
                f,
                "Invalid move at move {}: '{}' ({})",
                move_number, san, reason
            ),
            PgnError::InvalidResult(msg) => write!(f, "Invalid result: {}", msg),
        }
    }
}

impl std::error::Error for PgnError {}

// ---------------------------------------------------------------------------
// PGN headers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct PgnHeaders {
    pub pairs: Vec<(String, String)>,
}

impl PgnHeaders {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub fn event(&self) -> Option<&str> {
        self.get("Event")
    }
    pub fn site(&self) -> Option<&str> {
        self.get("Site")
    }
    pub fn date(&self) -> Option<&str> {
        self.get("Date")
    }
    pub fn white(&self) -> Option<&str> {
        self.get("White")
    }
    pub fn black(&self) -> Option<&str> {
        self.get("Black")
    }
    pub fn result(&self) -> Option<&str> {
        self.get("Result")
    }
}

// ---------------------------------------------------------------------------
// PGN result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PgnResult {
    WhiteWin,
    BlackWin,
    Draw,
    Unknown,
}

impl PgnResult {
    fn from_str(s: &str) -> Result<Self, PgnError> {
        match s.trim() {
            "1-0" => Ok(PgnResult::WhiteWin),
            "0-1" => Ok(PgnResult::BlackWin),
            "1/2-1/2" => Ok(PgnResult::Draw),
            "*" => Ok(PgnResult::Unknown),
            other => Err(PgnError::InvalidResult(other.to_string())),
        }
    }
}

impl fmt::Display for PgnResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PgnResult::WhiteWin => write!(f, "1-0"),
            PgnResult::BlackWin => write!(f, "0-1"),
            PgnResult::Draw => write!(f, "1/2-1/2"),
            PgnResult::Unknown => write!(f, "*"),
        }
    }
}

// ---------------------------------------------------------------------------
// PGN game
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PgnGame {
    pub headers: PgnHeaders,
    pub moves: Vec<Move>,
    pub result: PgnResult,
    pub final_game: StandardGame,
}

impl PgnGame {
    pub fn starting_fen(&self) -> Option<&str> {
        if self.headers.get("SetUp") == Some("1") {
            self.headers.get("FEN")
        } else {
            None
        }
    }

    pub fn starting_game(&self) -> Result<StandardGame, PgnError> {
        if let Some(fen) = self.starting_fen() {
            StandardGame::new(fen, true).map_err(PgnError::ParseError)
        } else {
            Ok(StandardGame::standard())
        }
    }
}

// ---------------------------------------------------------------------------
// Promotion normalization: e8Q -> e8=Q
// ---------------------------------------------------------------------------

fn normalize_san_promotion(san: &str) -> String {
    let san = san.trim_end_matches(['+', '#']);
    let chars: Vec<char> = san.chars().collect();
    let len = chars.len();

    // Minimum: destination (2) + promo piece (1) = 3, e.g. "e8Q"
    // Already has '=': no change needed
    if len >= 3 && !san.contains('=') {
        let last = chars[len - 1];
        let second_last = chars[len - 2];
        // Promotion piece is uppercase, preceded by rank digit (1 or 8 typically)
        if last.is_ascii_uppercase() && "QRBN".contains(last) && second_last.is_ascii_digit() {
            let mut result: String = chars[..len - 1].iter().collect();
            result.push('=');
            result.push(last);
            return result;
        }
    }
    san.to_string()
}

// ---------------------------------------------------------------------------
// Tree-sitter helpers
// ---------------------------------------------------------------------------

fn node_text<'a>(node: &Node, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}

fn child_by_field<'a>(node: &Node<'a>, field: &str) -> Option<Node<'a>> {
    node.child_by_field_name(field)
}

// ---------------------------------------------------------------------------
// Core parsing
// ---------------------------------------------------------------------------

fn parse_tagpair(node: &Node, source: &[u8]) -> Option<(String, String)> {
    let key_node = child_by_field(node, "tagpair_key")?;
    let key = node_text(&key_node, source).to_string();
    let value = child_by_field(node, "tagpair_value_contents")
        .map(|n| node_text(&n, source).to_string())
        .unwrap_or_default();
    Some((key, value))
}

fn parse_headers(header_node: &Node, source: &[u8]) -> PgnHeaders {
    let mut pairs = Vec::new();
    let mut cursor = header_node.walk();
    for child in header_node.children_by_field_name("tagpair", &mut cursor) {
        if let Some(pair) = parse_tagpair(&child, source) {
            pairs.push(pair);
        }
    }
    PgnHeaders { pairs }
}

fn parse_result_code(node: &Node, source: &[u8]) -> Result<PgnResult, PgnError> {
    let text = node_text(node, source);
    PgnResult::from_str(text)
}

fn parse_game_node(game_node: &Node, source: &[u8]) -> Result<PgnGame, PgnError> {
    // Parse headers
    let headers = child_by_field(game_node, "header")
        .map(|h| parse_headers(&h, source))
        .unwrap_or_default();

    // Determine starting position
    let mut game = PgnGame {
        headers: headers.clone(),
        moves: Vec::new(),
        result: PgnResult::Unknown,
        final_game: StandardGame::standard(),
    }
    .starting_game()?;

    // Parse moves from movetext
    let mut moves = Vec::new();
    if let Some(movetext_node) = child_by_field(game_node, "movetext") {
        let mut cursor = movetext_node.walk();

        // Collect san_move and lan_move nodes in document order
        let mut move_nodes: Vec<Node> = Vec::new();
        for i in 0..movetext_node.named_child_count() {
            if let Some(child) = movetext_node.named_child(i) {
                let kind = child.kind();
                if kind == "san_move" || kind == "lan_move" {
                    move_nodes.push(child);
                }
            }
        }

        // Also collect via field names (these should overlap but let's be thorough)
        // Actually, named_child iterates all named children. The field-based access
        // might give the same nodes. Let's just use the direct iteration approach.
        // Re-do: iterate all children in order, picking san_move and lan_move
        move_nodes.clear();
        cursor.reset(movetext_node);
        if cursor.goto_first_child() {
            loop {
                let node = cursor.node();
                let kind = node.kind();
                if kind == "san_move" || kind == "lan_move" {
                    move_nodes.push(node);
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        let move_number = |idx: usize| -> u32 { (idx as u32 / 2) + 1 };

        for (idx, move_node) in move_nodes.iter().enumerate() {
            let raw_text = node_text(move_node, source).trim();
            let kind = move_node.kind();

            if kind == "san_move" {
                let san = normalize_san_promotion(raw_text);
                let mv = game
                    .move_from_san(&san)
                    .map_err(|reason| PgnError::InvalidMove {
                        move_number: move_number(idx),
                        san: raw_text.to_string(),
                        reason,
                    })?;
                game.make_move_unchecked(&mv);
                moves.push(mv);
            } else if kind == "lan_move" {
                let mv = game
                    .move_from_lan(raw_text)
                    .map_err(|reason| PgnError::InvalidMove {
                        move_number: move_number(idx),
                        san: raw_text.to_string(),
                        reason,
                    })?;
                let success = game.make_move(&mv);
                if !success {
                    return Err(PgnError::InvalidMove {
                        move_number: move_number(idx),
                        san: raw_text.to_string(),
                        reason: "Illegal move".to_string(),
                    });
                }
                moves.push(mv);
            }
        }
    }

    // Parse result
    let result = child_by_field(game_node, "result_code")
        .map(|n| parse_result_code(&n, source))
        .transpose()?
        .unwrap_or(PgnResult::Unknown);

    Ok(PgnGame {
        headers,
        moves,
        result,
        final_game: game,
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// An iterator that yields [`PgnGame`]s one at a time from a parsed PGN tree.
///
/// Created by [`PgnIter::new`]. The tree-sitter parse tree is built upfront,
/// but individual games are only converted to [`PgnGame`] as they are consumed.
pub struct PgnIter {
    source: String,
    tree: tree_sitter::Tree,
    child_index: usize,
}

impl PgnIter {
    /// Parse the PGN source into a tree and return an iterator over its games.
    pub fn new(pgn: String) -> Result<Self, PgnError> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_pgn::LANGUAGE.into())
            .map_err(|e| PgnError::ParseError(format!("Failed to set language: {}", e)))?;

        let tree = parser
            .parse(&pgn, None)
            .ok_or_else(|| PgnError::ParseError("Failed to parse PGN".to_string()))?;

        Ok(Self {
            source: pgn,
            tree,
            child_index: 0,
        })
    }

    /// Extract raw PGN text for each game without parsing moves.
    ///
    /// This is much faster than iterating and fully parsing each game, because
    /// it only reads byte ranges from the tree-sitter tree. Useful when you want
    /// to distribute game parsing across multiple threads.
    pub fn raw_game_texts(&self) -> Vec<String> {
        let root = self.tree.root_node();
        let mut texts = Vec::new();
        for i in 0..root.child_count() {
            if let Some(child) = root.child(i)
                && child.kind() == "game"
            {
                texts.push(self.source[child.start_byte()..child.end_byte()].to_string());
            }
        }
        texts
    }
}

impl Iterator for PgnIter {
    type Item = Result<PgnGame, PgnError>;

    fn next(&mut self) -> Option<Self::Item> {
        let root = self.tree.root_node();
        let source = self.source.as_bytes();

        loop {
            let child = root.child(self.child_index)?;
            self.child_index += 1;
            if child.kind() == "game" {
                return Some(parse_game_node(&child, source));
            }
        }
    }
}

/// Parse a PGN string that may contain multiple games.
pub fn parse_pgn(pgn: &str) -> Result<Vec<PgnGame>, PgnError> {
    PgnIter::new(pgn.to_string())?.collect()
}

/// Parse a PGN string containing exactly one game.
pub fn parse_pgn_single_game(pgn: &str) -> Result<PgnGame, PgnError> {
    let mut games = parse_pgn(pgn)?;
    match games.len() {
        0 => Err(PgnError::ParseError("No games found in PGN".to_string())),
        1 => Ok(games.remove(0)),
        n => Err(PgnError::ParseError(format!(
            "Expected 1 game, found {}",
            n
        ))),
    }
}
