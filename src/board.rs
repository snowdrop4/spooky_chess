use crate::color::Color;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use std::fmt;

pub const STANDARD_COLS: usize = 8;
pub const STANDARD_ROWS: usize = 8;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Board {
    squares: Vec<Option<Piece>>,
    width: usize,
    height: usize,
}

impl Board {
    pub fn new(width: usize, height: usize, fen: &str) -> Result<Self, String> {
        let mut board = Board {
            squares: vec![None; width * height],
            width,
            height,
        };
        board.load_fen(fen)?;
        Ok(board)
    }

    pub fn standard() -> Self {
        Self::new(
            STANDARD_COLS,
            STANDARD_ROWS,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
        )
        .expect("Failed to create standard board")
    }

    pub fn empty(width: usize, height: usize) -> Self {
        Board {
            squares: vec![None; width * height],
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn index(&self, col: usize, row: usize) -> usize {
        row * self.width + col
    }

    pub fn get_piece(&self, pos: &Position) -> Option<Piece> {
        if pos.is_valid(self.width, self.height) {
            self.squares[self.index(pos.col, pos.row)]
        } else {
            None
        }
    }

    pub fn set_piece(&mut self, pos: &Position, piece: Option<Piece>) {
        if pos.is_valid(self.width, self.height) {
            let index = self.index(pos.col, pos.row);
            self.squares[index] = piece;
        }
    }

    pub fn clear(&mut self) {
        self.squares = vec![None; self.width * self.height];
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        for row in (0..self.height).rev() {
            let mut empty_count = 0;

            for col in 0..self.width {
                let pos = Position::new(col, row);
                if let Some(piece) = self.get_piece(&pos) {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    fen.push(piece.to_char());
                } else {
                    empty_count += 1;
                }
            }

            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }

            if row > 0 {
                fen.push('/');
            }
        }

        fen
    }

    fn load_fen(&mut self, fen: &str) -> Result<(), String> {
        self.clear();

        let parts: Vec<&str> = fen.split('/').collect();

        if parts.len() != self.height {
            return Err(format!(
                "Invalid FEN: expected {} rows, got {}",
                self.height,
                parts.len()
            ));
        }

        for (row_idx, row_str) in parts.iter().enumerate() {
            let row = self.height - 1 - row_idx;
            let mut col = 0;
            let mut chars = row_str.chars().peekable();

            while let Some(c) = chars.next() {
                if c.is_ascii_digit() {
                    // Collect all consecutive digits to handle multi-digit numbers
                    let mut num_str = String::new();
                    num_str.push(c);

                    while let Some(&next_c) = chars.peek() {
                        if next_c.is_ascii_digit() {
                            num_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }

                    let skip = num_str
                        .parse::<usize>()
                        .map_err(|_| format!("Invalid FEN number: {}", num_str))?;
                    col += skip;
                } else if let Some(piece) = Piece::from_char(c) {
                    if col >= self.width {
                        return Err("Invalid FEN: col index out of bounds".to_string());
                    }
                    self.set_piece(&Position::new(col, row), Some(piece));
                    col += 1;
                } else {
                    return Err(format!("Invalid FEN character: {}", c));
                }
            }

            if col != self.width {
                return Err(format!(
                    "Invalid FEN: row {} has wrong number of squares",
                    row
                ));
            }
        }

        Ok(())
    }

    pub fn pieces(&self, color: Color) -> Vec<(Position, Piece)> {
        let mut result = Vec::new();
        for row in 0..self.height {
            for col in 0..self.width {
                let pos = Position::new(col, row);
                if let Some(piece) = self.get_piece(&pos) {
                    if piece.color == color {
                        result.push((pos, piece));
                    }
                }
            }
        }
        result
    }

    pub fn find_king(&self, color: Color) -> Option<Position> {
        for row in 0..self.height {
            for col in 0..self.width {
                let pos = Position::new(col, row);
                if let Some(piece) = self.get_piece(&pos) {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::standard()
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in (0..self.height).rev() {
            write!(f, "{:2} ", row + 1)?;
            for col in 0..self.width {
                let pos = Position::new(col, row);
                if let Some(piece) = self.get_piece(&pos) {
                    write!(f, "{} ", piece.to_char())?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }

        write!(f, "   ")?;
        for col in 0..self.width {
            if col < 26 {
                write!(f, "{} ", (b'a' + col as u8) as char)?;
            } else {
                write!(f, "{} ", col)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_board_creation() {
        let board = Board::empty(6, 6);
        assert_eq!(board.width(), 6);
        assert_eq!(board.height(), 6);

        let board = Board::empty(10, 10);
        assert_eq!(board.width(), 10);
        assert_eq!(board.height(), 10);
    }

    #[test]
    fn test_standard_board_creation() {
        let board = Board::standard();
        assert_eq!(board.width(), 8);
        assert_eq!(board.height(), 8);
    }

    #[test]
    fn test_custom_board_creation() {
        let board = Board::new(6, 6, "rnbqk1/pppppp/6/6/PPPPPP/RNBQK1")
            .expect("Failed to create custom board");
        assert_eq!(board.width(), 6);
        assert_eq!(board.height(), 6);
    }

    #[test]
    fn test_custom_board_creation_invalid() {
        let board = Board::new(6, 6, "rnbqk1/pppppp/1/6/PPPPPP/RNBQK1");
        assert!(board.is_err(), "Expected error for invalid FEN");
    }

    #[test]
    fn test_board_piece_placement() {
        let mut board = Board::empty(8, 8);
        let king = Piece::new(PieceType::King, Color::White);
        let pos = Position::new(4, 0);

        board.set_piece(&pos, Some(king));
        assert_eq!(board.get_piece(&pos), Some(king));

        board.set_piece(&pos, None);
        assert_eq!(board.get_piece(&pos), None);
    }

    #[test]
    fn test_board_standard_position() {
        let board = Board::standard();

        // Check white pieces
        assert_eq!(
            board
                .get_piece(&Position::new(0, 0))
                .expect("Expected piece at position (0,0)")
                .piece_type,
            PieceType::Rook
        );
        assert_eq!(
            board
                .get_piece(&Position::new(0, 0))
                .expect("Expected piece at position (0,0)")
                .color,
            Color::White
        );
        assert_eq!(
            board
                .get_piece(&Position::new(4, 0))
                .expect("Expected piece at position (4,0)")
                .piece_type,
            PieceType::King
        );

        // Check white pawns
        for col in 0..8 {
            assert_eq!(
                board
                    .get_piece(&Position::new(col, 1))
                    .unwrap_or_else(|| panic!("Expected piece at position ({},1)", col))
                    .piece_type,
                PieceType::Pawn
            );
            assert_eq!(
                board
                    .get_piece(&Position::new(col, 1))
                    .unwrap_or_else(|| panic!("Expected piece at position ({},1)", col))
                    .color,
                Color::White
            );
        }

        // Check black pieces
        assert_eq!(
            board
                .get_piece(&Position::new(0, 7))
                .expect("Expected piece at position (0,7)")
                .piece_type,
            PieceType::Rook
        );
        assert_eq!(
            board
                .get_piece(&Position::new(0, 7))
                .expect("Expected piece at position (0,7)")
                .color,
            Color::Black
        );
        assert_eq!(
            board
                .get_piece(&Position::new(4, 7))
                .expect("Expected piece at position (4,7)")
                .piece_type,
            PieceType::King
        );

        // Check black pawns
        for col in 0..8 {
            assert_eq!(
                board
                    .get_piece(&Position::new(col, 6))
                    .unwrap_or_else(|| panic!("Expected piece at position ({},6)", col))
                    .piece_type,
                PieceType::Pawn
            );
            assert_eq!(
                board
                    .get_piece(&Position::new(col, 6))
                    .unwrap_or_else(|| panic!("Expected piece at position ({},6)", col))
                    .color,
                Color::Black
            );
        }
    }

    #[test]
    fn test_board_fen_conversion() {
        let board = Board::standard();

        let fen = board.to_fen();
        assert_eq!(fen, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");

        let new_board = Board::new(8, 8, &fen).expect("Failed to parse FEN string");

        // Verify the boards are identical
        for row in 0..8 {
            for col in 0..8 {
                let pos = Position::new(col, row);
                assert_eq!(board.get_piece(&pos), new_board.get_piece(&pos));
            }
        }
    }
}
