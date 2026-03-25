use arrayvec::ArrayVec;
use smallvec::SmallVec;

use crate::bitboard::BoardGeometry;
use crate::board::Board;
use crate::color::Color;
use crate::limits::validate_board_dimensions;
use crate::r#move::Move;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use std::hash::Hash;

mod action;
mod check_pin;
mod make_move;
#[macro_use]
mod movegen;
mod state;

#[cfg(test)]
mod tests_standard;

#[cfg(test)]
mod tests_parametrised;

#[cfg(test)]
mod tests_san;

#[cfg(test)]
mod tests_actions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PieceCounts {
    /// counts[piece_type as usize][color_index] where color_index: White=0, Black=1
    counts: [[u8; 2]; 6],
}

impl Default for PieceCounts {
    fn default() -> Self {
        Self::new()
    }
}

impl PieceCounts {
    #[inline]
    fn color_idx(color: Color) -> usize {
        match color {
            Color::White => 0,
            Color::Black => 1,
        }
    }

    pub fn new() -> Self {
        PieceCounts {
            counts: [[0; 2]; 6],
        }
    }

    pub(crate) fn from_board<const W: usize, const H: usize>(board: &Board<W, H>) -> Self
    where
        [(); (W * H).div_ceil(64)]:,
    {
        let mut counts = PieceCounts::new();
        for piece_type in [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
        ] {
            let bb = board.piece_type_bb(piece_type);
            let white_count = (bb & board.color_bb(Color::White)).count() as u8;
            let black_count = (bb & board.color_bb(Color::Black)).count() as u8;
            counts.counts[piece_type as usize][0] = white_count;
            counts.counts[piece_type as usize][1] = black_count;
        }
        counts
    }

    #[inline]
    pub fn increment(&mut self, piece_type: PieceType, color: Color) {
        self.counts[piece_type as usize][Self::color_idx(color)] += 1;
    }

    #[inline]
    pub fn decrement(&mut self, piece_type: PieceType, color: Color) {
        debug_assert!(
            self.counts[piece_type as usize][Self::color_idx(color)] > 0,
            "PieceCounts underflow for {:?} {:?}",
            piece_type,
            color,
        );
        self.counts[piece_type as usize][Self::color_idx(color)] -= 1;
    }

    #[inline]
    pub fn get(&self, piece_type: PieceType, color: Color) -> u8 {
        self.counts[piece_type as usize][Self::color_idx(color)]
    }
}

#[derive(Clone)]
pub struct MoveHistoryEntry {
    pub mv: Move,
    captured: Option<Piece>,
    castling_rights: CastlingRights,
    en_passant: Option<Position>,
    halfmove_clock: u32,
    piece_counts: PieceCounts,
}

#[derive(Clone)]
pub struct Game<const W: usize, const H: usize>
where
    [(); (W * H).div_ceil(64)]:,
{
    board: Board<W, H>,
    turn: Color,
    move_history: SmallVec<[MoveHistoryEntry; 256]>,

    castling_rights: CastlingRights,
    castling_enabled: bool,

    en_passant: Option<Position>,

    halfmove_clock: u32,
    fullmove_number: u32,

    white_king_pos: Position,
    black_king_pos: Position,

    piece_counts: PieceCounts,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

#[hotpath::measure_all]
impl Default for CastlingRights {
    fn default() -> Self {
        Self::new()
    }
}

#[hotpath::measure_all]
impl CastlingRights {
    pub fn new() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }

    pub fn none() -> Self {
        CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    pub fn has_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_kingside,
            Color::Black => self.black_kingside,
        }
    }

    pub fn has_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_queenside,
            Color::Black => self.black_queenside,
        }
    }

    fn set_kingside(&mut self, color: Color, value: bool) {
        match color {
            Color::White => self.white_kingside = value,
            Color::Black => self.black_kingside = value,
        }
    }

    fn set_queenside(&mut self, color: Color, value: bool) {
        match color {
            Color::White => self.white_queenside = value,
            Color::Black => self.black_queenside = value,
        }
    }

    /// Revoke castling rights associated with a rook at the given corner position.
    fn revoke_at(&mut self, pos: &Position, width: usize, height: usize) {
        let last_col = width - 1;
        let last_row = height - 1;
        if pos.col == 0 && pos.row == 0 {
            self.white_queenside = false;
        } else if usize::from(pos.col) == last_col && pos.row == 0 {
            self.white_kingside = false;
        } else if pos.col == 0 && usize::from(pos.row) == last_row {
            self.black_queenside = false;
        } else if usize::from(pos.col) == last_col && usize::from(pos.row) == last_row {
            self.black_kingside = false;
        }
    }
}

#[hotpath::measure_all]
impl<const W: usize, const H: usize> Game<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    #[inline]
    fn geo() -> &'static BoardGeometry<W, H> {
        &BoardGeometry::<W, H>::INSTANCE
    }

    pub fn new(fen: &str, castling_enabled: bool) -> Result<Self, String> {
        validate_board_dimensions(W, H)?;

        let parts: ArrayVec<&str, 6> = fen.split(' ').collect();

        if parts.is_empty() {
            return Err("Empty FEN string".to_string());
        }

        // FEN must have exactly 6 parts: position, turn, castling, en_passant, halfmove, fullmove
        if parts.len() != 6 {
            return Err(format!(
                "Invalid FEN: expected 6 parts, got {}",
                parts.len()
            ));
        }

        let board = Board::new(parts[0])?;

        // Turn
        let turn = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid turn in FEN".to_string()),
        };

        // Castling rights
        let mut castling_rights = CastlingRights::none();
        if castling_enabled {
            for c in parts[2].chars() {
                match c {
                    'K' => castling_rights.white_kingside = true,
                    'Q' => castling_rights.white_queenside = true,
                    'k' => castling_rights.black_kingside = true,
                    'q' => castling_rights.black_queenside = true,
                    '-' => {}
                    _ => return Err("Invalid castling rights in FEN".to_string()),
                }
            }
        }

        // En passant
        let en_passant = if parts[3] != "-" {
            Some(Position::from_algebraic(parts[3])?)
        } else {
            None
        };

        // Halfmove clock
        let halfmove_clock = parts[4]
            .parse()
            .map_err(|_| "Invalid halfmove clock in FEN".to_string())?;

        // Fullmove number
        let fullmove_number = parts[5]
            .parse()
            .map_err(|_| "Invalid fullmove number in FEN".to_string())?;

        // Find king positions
        let white_king_pos = board
            .find_king(Color::White)
            .ok_or("No white king found in FEN position".to_string())?;
        let black_king_pos = board
            .find_king(Color::Black)
            .ok_or("No black king found in FEN position".to_string())?;

        debug_assert!(
            board.get_piece(&white_king_pos)
                == Some(Piece::new(crate::pieces::PieceType::King, Color::White)),
            "white_king_pos {:?} does not point to a white king after FEN parse",
            white_king_pos,
        );
        debug_assert!(
            board.get_piece(&black_king_pos)
                == Some(Piece::new(crate::pieces::PieceType::King, Color::Black)),
            "black_king_pos {:?} does not point to a black king after FEN parse",
            black_king_pos,
        );

        // Count pieces from the board
        let piece_counts = PieceCounts::from_board(&board);

        Ok(Game {
            board,
            turn,
            move_history: SmallVec::new(),
            castling_rights,
            castling_enabled,
            en_passant,
            halfmove_clock,
            fullmove_number,
            white_king_pos,
            black_king_pos,
            piece_counts,
        })
    }

    pub fn width(&self) -> usize {
        W
    }

    pub fn height(&self) -> usize {
        H
    }

    pub fn get_piece(&self, pos: &Position) -> Option<Piece> {
        self.board.get_piece(pos)
    }

    pub fn set_piece(&mut self, pos: &Position, piece: Option<Piece>) {
        // Update piece counts for the removed piece
        if let Some(existing) = self.board.get_piece(pos) {
            self.piece_counts
                .decrement(existing.piece_type, existing.color);
        }
        // Update piece counts for the new piece
        if let Some(ref p) = piece {
            self.piece_counts.increment(p.piece_type, p.color);
        }
        self.board.set_piece(pos, piece)
    }

    /// Clear the board and reset piece counts.
    pub fn clear_board(&mut self) {
        self.board.clear();
        self.piece_counts = PieceCounts::new();
    }

    /// Recompute piece counts from the board. Use after direct board manipulation.
    pub fn sync_piece_counts(&mut self) {
        self.piece_counts = PieceCounts::from_board(&self.board);
    }

    pub fn pieces(&self, color: Color) -> Vec<(Position, Piece)> {
        self.board.pieces(color)
    }

    pub(crate) fn pieces_iter(&self, color: Color) -> crate::board::PieceIterator<'_, W, H> {
        self.board.pieces_iter(color)
    }

    pub fn board_hash<HH: std::hash::Hasher>(&self, state: &mut HH) {
        self.board.hash(state);
    }

    pub fn turn(&self) -> Color {
        self.turn
    }

    pub fn fullmove_number(&self) -> u32 {
        self.fullmove_number
    }

    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    pub fn move_count(&self) -> usize {
        self.move_history.len()
    }

    pub fn move_history(&self) -> &[MoveHistoryEntry] {
        &self.move_history
    }

    pub fn castling_enabled(&self) -> bool {
        self.castling_enabled
    }

    pub fn castling_rights(&self) -> &CastlingRights {
        &self.castling_rights
    }

    pub fn piece_counts(&self) -> &PieceCounts {
        &self.piece_counts
    }
}

/// Type alias for a standard 8x8 game
pub type StandardGame = Game<8, 8>;

#[hotpath::measure_all]
impl StandardGame {
    pub fn standard() -> Self {
        Self::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            true,
        )
        .expect("Failed to create standard game")
    }
}

#[hotpath::measure_all]
impl<const W: usize, const H: usize> std::fmt::Display for Game<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Game(current_player: {})\n{}", self.turn(), self.board)
    }
}
