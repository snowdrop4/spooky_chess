use arrayvec::ArrayVec;

use crate::bitboard::BoardGeometry;
use crate::board::Board;
use crate::color::Color;
use crate::r#move::Move;
use crate::pieces::Piece;
use crate::position::Position;

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

#[derive(Clone)]
pub struct MoveHistoryEntry {
    pub mv: Move,
    captured: Option<Piece>,
    castling_rights: CastlingRights,
    en_passant: Option<Position>,
    halfmove_clock: u32,
}

#[derive(Clone)]
pub struct Game<const W: usize, const H: usize>
where
    [(); (W * H).div_ceil(64)]:,
{
    board: Board<W, H>,
    turn: Color,
    move_history: Vec<MoveHistoryEntry>,

    castling_rights: CastlingRights,
    castling_enabled: bool,

    en_passant: Option<Position>,

    halfmove_clock: u32,
    fullmove_number: u32,

    white_king_pos: Position,
    black_king_pos: Position,
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
        } else if pos.col == last_col && pos.row == 0 {
            self.white_kingside = false;
        } else if pos.col == 0 && pos.row == last_row {
            self.black_queenside = false;
        } else if pos.col == last_col && pos.row == last_row {
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

        Ok(Game {
            board,
            turn,
            move_history: Vec::new(),
            castling_rights,
            castling_enabled,
            en_passant,
            halfmove_clock,
            fullmove_number,
            white_king_pos,
            black_king_pos,
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
        self.board.set_piece(pos, piece)
    }

    pub fn board(&self) -> &Board<W, H> {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board<W, H> {
        &mut self.board
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
