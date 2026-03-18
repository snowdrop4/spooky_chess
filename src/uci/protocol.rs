use crate::r#move::Move;
use std::fmt;
use std::io;

/// Result of a UCI `go` command.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub best_move_lan: String,
    pub ponder_move: Option<Move>,
    pub ponder_move_lan: Option<String>,
    pub info: Vec<InfoLine>,
}

/// A parsed UCI `info` line.
#[derive(Debug, Clone)]
pub struct InfoLine {
    pub depth: Option<u32>,
    pub score_cp: Option<i32>,
    pub score_mate: Option<i32>,
    pub nodes: Option<u64>,
    pub nps: Option<u64>,
    pub time_ms: Option<u64>,
    pub pv: Vec<String>,
}

/// Errors that can occur during UCI communication.
#[derive(Debug)]
pub enum UciError {
    IoError(io::Error),
    ProtocolError(String),
    EngineExited,
    IllegalMove(String),
}

impl fmt::Display for UciError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UciError::IoError(e) => write!(f, "IO error: {}", e),
            UciError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            UciError::EngineExited => write!(f, "Engine process exited unexpectedly"),
            UciError::IllegalMove(msg) => write!(f, "Illegal move: {}", msg),
        }
    }
}

impl std::error::Error for UciError {}

impl From<io::Error> for UciError {
    fn from(e: io::Error) -> Self {
        UciError::IoError(e)
    }
}

// --- Command builders ---

#[hotpath::measure]
#[allow(dead_code)]
pub fn cmd_position(started_from_fen: &Option<String>, moves: &[String]) -> String {
    let mut cmd = String::from("position ");
    match started_from_fen {
        Some(fen) => {
            cmd.push_str("fen ");
            cmd.push_str(fen);
        }
        None => {
            cmd.push_str("startpos");
        }
    }
    if !moves.is_empty() {
        cmd.push_str(" moves");
        for m in moves {
            cmd.push(' ');
            cmd.push_str(m);
        }
    }
    cmd
}

#[hotpath::measure]
pub fn cmd_go_depth(depth: u32) -> String {
    format!("go depth {}", depth)
}

#[hotpath::measure]
pub fn cmd_go_movetime(ms: u64) -> String {
    format!("go movetime {}", ms)
}

#[hotpath::measure]
pub fn cmd_go_clock(wtime: u64, btime: u64, winc: u64, binc: u64) -> String {
    format!(
        "go wtime {} btime {} winc {} binc {}",
        wtime, btime, winc, binc
    )
}

#[hotpath::measure]
pub fn cmd_setoption(name: &str, value: &str) -> String {
    format!("setoption name {} value {}", name, value)
}

// --- Response parsers ---

/// Parse a `id name ...` or `id author ...` line.
/// Returns `(key, value)` where key is "name" or "author".
#[hotpath::measure]
pub fn parse_id_line(line: &str) -> Option<(&str, String)> {
    let line = line.trim();
    if !line.starts_with("id ") {
        return None;
    }
    let rest = &line[3..];
    if let Some(value) = rest.strip_prefix("name ") {
        Some(("name", value.to_string()))
    } else if let Some(value) = rest.strip_prefix("author ") {
        Some(("author", value.to_string()))
    } else {
        None
    }
}

/// Parse a `bestmove <move> [ponder <move>]` line.
/// Returns `(bestmove_lan, ponder_lan)`.
#[hotpath::measure]
pub fn parse_bestmove_line(line: &str) -> Option<(String, Option<String>)> {
    let line = line.trim();
    let rest = line.strip_prefix("bestmove ")?;
    let mut tokens = rest.split_ascii_whitespace();
    let best = tokens.next()?.to_string();
    let ponder = if tokens.next() == Some("ponder") {
        tokens.next().map(|t| t.to_string())
    } else {
        None
    };
    Some((best, ponder))
}

/// Parse a UCI `info` line into an `InfoLine`.
#[hotpath::measure]
pub fn parse_info_line(line: &str) -> Option<InfoLine> {
    let line = line.trim();
    let rest = line.strip_prefix("info ")?;
    let mut tokens = rest.split_ascii_whitespace();

    let mut depth = None;
    let mut score_cp = None;
    let mut score_mate = None;
    let mut nodes = None;
    let mut nps = None;
    let mut time_ms = None;
    let mut pv = Vec::new();

    while let Some(token) = tokens.next() {
        match token {
            "depth" => {
                depth = tokens.next().and_then(|t| t.parse().ok());
            }
            "score" => match tokens.next() {
                Some("cp") => {
                    score_cp = tokens.next().and_then(|t| t.parse().ok());
                }
                Some("mate") => {
                    score_mate = tokens.next().and_then(|t| t.parse().ok());
                }
                _ => {}
            },
            "nodes" => {
                nodes = tokens.next().and_then(|t| t.parse().ok());
            }
            "nps" => {
                nps = tokens.next().and_then(|t| t.parse().ok());
            }
            "time" => {
                time_ms = tokens.next().and_then(|t| t.parse().ok());
            }
            "pv" => {
                pv = tokens.map(|t| t.to_string()).collect();
                break;
            }
            _ => {}
        }
    }

    Some(InfoLine {
        depth,
        score_cp,
        score_mate,
        nodes,
        nps,
        time_ms,
        pv,
    })
}
