mod protocol;

pub use protocol::{InfoLine, SearchResult, UciError};

use crate::color::Color;
use crate::game::StandardGame;
use crate::r#move::Move;

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct UciEngine {
    process: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    engine_name: Option<String>,
    engine_author: Option<String>,
    game: StandardGame,
    move_history_lan: Vec<String>,
    started_from_fen: Option<String>,
}

impl UciEngine {
    /// Spawn a UCI engine process and perform the UCI handshake.
    pub fn new(program: &str, args: &[&str]) -> Result<Self, UciError> {
        let mut process = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = BufWriter::new(
            process
                .stdin
                .take()
                .ok_or_else(|| UciError::ProtocolError("Failed to open engine stdin".into()))?,
        );
        let stdout = BufReader::new(
            process
                .stdout
                .take()
                .ok_or_else(|| UciError::ProtocolError("Failed to open engine stdout".into()))?,
        );

        let mut engine = UciEngine {
            process,
            stdin,
            stdout,
            engine_name: None,
            engine_author: None,
            game: StandardGame::standard(),
            move_history_lan: Vec::new(),
            started_from_fen: None,
        };

        // Send "uci" and wait for "uciok"
        engine.send_line("uci")?;
        engine.read_until_uciok()?;

        // Sync with "isready"/"readyok"
        engine.is_ready()?;

        Ok(engine)
    }

    pub fn engine_name(&self) -> Option<&str> {
        self.engine_name.as_deref()
    }

    pub fn engine_author(&self) -> Option<&str> {
        self.engine_author.as_deref()
    }

    pub fn game(&self) -> &StandardGame {
        &self.game
    }

    /// Send `setoption name <name> value <value>` to the engine.
    pub fn set_option(&mut self, name: &str, value: &str) -> Result<(), UciError> {
        let cmd = protocol::cmd_setoption(name, value);
        self.send_line(&cmd)?;
        Ok(())
    }

    /// Send `isready` and block until `readyok` is received.
    pub fn is_ready(&mut self) -> Result<(), UciError> {
        self.send_line("isready")?;
        loop {
            let line = self.read_line()?;
            if line.trim() == "readyok" {
                return Ok(());
            }
        }
    }

    /// Reset the internal game to the standard starting position.
    pub fn set_position_startpos(&mut self) {
        self.game = StandardGame::standard();
        self.move_history_lan.clear();
        self.started_from_fen = None;
    }

    /// Reset the internal game to a position given by FEN.
    pub fn set_position_fen(&mut self, fen: &str) -> Result<(), UciError> {
        let game = StandardGame::new(fen, true)
            .map_err(|e| UciError::ProtocolError(format!("Invalid FEN: {}", e)))?;
        self.game = game;
        self.move_history_lan.clear();
        self.started_from_fen = Some(fen.to_string());
        Ok(())
    }

    /// Apply a `Move` to the internal game state and record it in history.
    /// Returns whether the move was legal and successfully applied.
    pub fn make_move(&mut self, mv: &Move) -> Result<bool, UciError> {
        let lan = mv.to_lan();
        let ok = self.game.make_move(mv);
        if ok {
            self.move_history_lan.push(lan);
        }
        Ok(ok)
    }

    /// Parse a LAN string using the current game context, then apply it.
    pub fn make_move_lan(&mut self, lan: &str) -> Result<bool, UciError> {
        let mv = self
            .game
            .move_from_lan(lan)
            .map_err(UciError::IllegalMove)?;
        self.make_move(&mv)
    }

    /// Search with a depth limit. Returns the full `SearchResult`.
    pub fn go_depth(&mut self, depth: u32) -> Result<SearchResult, UciError> {
        self.send_position()?;
        let cmd = protocol::cmd_go_depth(depth);
        self.send_line(&cmd)?;
        self.read_search_result()
    }

    /// Search with a time limit in milliseconds.
    pub fn go_movetime(&mut self, ms: u64) -> Result<SearchResult, UciError> {
        self.send_position()?;
        let cmd = protocol::cmd_go_movetime(ms);
        self.send_line(&cmd)?;
        self.read_search_result()
    }

    /// Search with clock parameters.
    pub fn go_clock(
        &mut self,
        wtime: u64,
        btime: u64,
        winc: u64,
        binc: u64,
    ) -> Result<SearchResult, UciError> {
        self.send_position()?;
        let cmd = protocol::cmd_go_clock(wtime, btime, winc, binc);
        self.send_line(&cmd)?;
        self.read_search_result()
    }

    /// Search to a given depth, then automatically apply the best move.
    /// Returns the best move.
    pub fn go_bestmove_depth(&mut self, depth: u32) -> Result<Move, UciError> {
        let result = self.go_depth(depth)?;
        let mv = result.best_move;
        self.make_move(&mv)?;
        Ok(mv)
    }

    /// Search for a given time, then automatically apply the best move.
    pub fn go_bestmove_movetime(&mut self, ms: u64) -> Result<Move, UciError> {
        let result = self.go_movetime(ms)?;
        let mv = result.best_move;
        self.make_move(&mv)?;
        Ok(mv)
    }

    /// Get the current turn color.
    pub fn turn(&self) -> Color {
        self.game.turn()
    }

    /// Check if the game is over.
    pub fn is_over(&mut self) -> bool {
        self.game.is_over()
    }

    /// Get legal moves from the current position.
    pub fn legal_moves(&mut self) -> Vec<Move> {
        self.game.legal_moves()
    }

    /// Undo the last move. Returns false if there is no move to undo.
    pub fn undo(&mut self) -> bool {
        if self.game.unmake_move() {
            self.move_history_lan.pop();
            true
        } else {
            false
        }
    }

    /// Send a raw UCI command and return the response line.
    pub fn send_command(&mut self, cmd: &str) -> Result<String, UciError> {
        self.send_line(cmd)?;
        self.read_line()
    }

    /// Send `quit` to the engine.
    pub fn quit(&mut self) -> Result<(), UciError> {
        let _ = self.send_line("quit");
        Ok(())
    }

    // --- Private helpers ---

    fn send_line(&mut self, line: &str) -> Result<(), UciError> {
        writeln!(self.stdin, "{}", line)?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_line(&mut self) -> Result<String, UciError> {
        let mut line = String::new();
        let n = self.stdout.read_line(&mut line)?;
        if n == 0 {
            return Err(UciError::EngineExited);
        }
        Ok(line)
    }

    fn read_until_uciok(&mut self) -> Result<(), UciError> {
        loop {
            let line = self.read_line()?;
            let trimmed = line.trim();

            if let Some((key, value)) = protocol::parse_id_line(trimmed) {
                match key {
                    "name" => self.engine_name = Some(value),
                    "author" => self.engine_author = Some(value),
                    _ => {}
                }
            }

            if trimmed == "uciok" {
                return Ok(());
            }
        }
    }

    /// Send the current position to the engine.
    fn send_position(&mut self) -> Result<(), UciError> {
        let cmd = protocol::cmd_position(&self.started_from_fen, &self.move_history_lan);
        self.send_line(&cmd)?;
        Ok(())
    }

    /// Read engine output until `bestmove` is received, collecting `info` lines.
    fn read_search_result(&mut self) -> Result<SearchResult, UciError> {
        let mut info_lines = Vec::new();

        loop {
            let line = self.read_line()?;
            let trimmed = line.trim();

            if let Some(info) = protocol::parse_info_line(trimmed) {
                info_lines.push(info);
            }

            if let Some((best_lan, ponder_lan)) = protocol::parse_bestmove_line(trimmed) {
                let best_move = self
                    .game
                    .move_from_lan(&best_lan)
                    .map_err(UciError::IllegalMove)?;

                let ponder_move = if let Some(ref ponder_str) = ponder_lan {
                    // To parse ponder move, we need to temporarily apply the best move
                    let mut temp_game = self.game.clone();
                    temp_game.make_move_unchecked(&best_move);
                    Some(
                        temp_game
                            .move_from_lan(ponder_str)
                            .map_err(UciError::IllegalMove)?,
                    )
                } else {
                    None
                };

                return Ok(SearchResult {
                    best_move,
                    best_move_lan: best_lan,
                    ponder_move,
                    ponder_move_lan: ponder_lan,
                    info: info_lines,
                });
            }
        }
    }
}

impl Drop for UciEngine {
    fn drop(&mut self) {
        let _ = self.send_line("quit");
        let _ = self.process.wait();
    }
}

#[cfg(test)]
mod tests;
