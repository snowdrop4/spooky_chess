use crate::color::Color;

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Color::White => "White",
            Color::Black => "Black",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Piece { piece_type, color }
    }

    pub fn to_char(&self) -> char {
        let c = match self.piece_type {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        };

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

        let piece_type = match c.to_ascii_lowercase() {
            'p' => PieceType::Pawn,
            'n' => PieceType::Knight,
            'b' => PieceType::Bishop,
            'r' => PieceType::Rook,
            'q' => PieceType::Queen,
            'k' => PieceType::King,
            _ => return None,
        };

        Some(Piece::new(piece_type, color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_creation() {
        let piece = Piece::new(PieceType::King, Color::White);
        assert_eq!(piece.piece_type, PieceType::King);
        assert_eq!(piece.color, Color::White);
    }

    #[test]
    fn test_piece_to_char() {
        assert_eq!(Piece::new(PieceType::King, Color::White).to_char(), 'K');
        assert_eq!(Piece::new(PieceType::King, Color::Black).to_char(), 'k');
        assert_eq!(Piece::new(PieceType::Queen, Color::White).to_char(), 'Q');
        assert_eq!(Piece::new(PieceType::Queen, Color::Black).to_char(), 'q');
        assert_eq!(Piece::new(PieceType::Rook, Color::White).to_char(), 'R');
        assert_eq!(Piece::new(PieceType::Rook, Color::Black).to_char(), 'r');
        assert_eq!(Piece::new(PieceType::Bishop, Color::White).to_char(), 'B');
        assert_eq!(Piece::new(PieceType::Bishop, Color::Black).to_char(), 'b');
        assert_eq!(Piece::new(PieceType::Knight, Color::White).to_char(), 'N');
        assert_eq!(Piece::new(PieceType::Knight, Color::Black).to_char(), 'n');
        assert_eq!(Piece::new(PieceType::Pawn, Color::White).to_char(), 'P');
        assert_eq!(Piece::new(PieceType::Pawn, Color::Black).to_char(), 'p');
    }

    #[test]
    fn test_piece_from_char() {
        assert_eq!(
            Piece::from_char('K'),
            Some(Piece::new(PieceType::King, Color::White))
        );
        assert_eq!(
            Piece::from_char('k'),
            Some(Piece::new(PieceType::King, Color::Black))
        );
        assert_eq!(
            Piece::from_char('Q'),
            Some(Piece::new(PieceType::Queen, Color::White))
        );
        assert_eq!(
            Piece::from_char('q'),
            Some(Piece::new(PieceType::Queen, Color::Black))
        );
        assert_eq!(
            Piece::from_char('R'),
            Some(Piece::new(PieceType::Rook, Color::White))
        );
        assert_eq!(
            Piece::from_char('r'),
            Some(Piece::new(PieceType::Rook, Color::Black))
        );
        assert_eq!(
            Piece::from_char('B'),
            Some(Piece::new(PieceType::Bishop, Color::White))
        );
        assert_eq!(
            Piece::from_char('b'),
            Some(Piece::new(PieceType::Bishop, Color::Black))
        );
        assert_eq!(
            Piece::from_char('N'),
            Some(Piece::new(PieceType::Knight, Color::White))
        );
        assert_eq!(
            Piece::from_char('n'),
            Some(Piece::new(PieceType::Knight, Color::Black))
        );
        assert_eq!(
            Piece::from_char('P'),
            Some(Piece::new(PieceType::Pawn, Color::White))
        );
        assert_eq!(
            Piece::from_char('p'),
            Some(Piece::new(PieceType::Pawn, Color::Black))
        );
        assert_eq!(Piece::from_char('x'), None);
    }
}
