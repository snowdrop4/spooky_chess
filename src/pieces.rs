use crate::color::Color;

/// L-shaped knight move offsets as (col_delta, row_delta).
/// The ordering is stable and used by the encode module for plane indices.
pub const KNIGHT_DELTAS: [(i32, i32); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceType {
    pub fn to_char(self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            'p' => Some(PieceType::Pawn),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

#[hotpath::measure_all]
impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Piece { piece_type, color }
    }

    pub fn to_char(&self) -> char {
        let c = self.piece_type.to_char();
        match self.color {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        PieceType::from_char(c).map(|pt| Piece::new(pt, color))
    }
}
