use crate::color::Color;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameOutcome {
    WhiteWin,
    BlackWin,
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule,
    Other,
}

impl GameOutcome {
    pub fn winner(&self) -> Option<Color> {
        match self {
            GameOutcome::WhiteWin => Some(Color::White),
            GameOutcome::BlackWin => Some(Color::Black),
            _ => None,
        }
    }

    pub fn encode_winner_absolute(&self) -> f32 {
        match self {
            GameOutcome::WhiteWin => 1.0,
            GameOutcome::BlackWin => -1.0,
            _ => 0.0,
        }
    }

    pub fn encode_winner_from_perspective(&self, perspective: Color) -> f32 {
        match perspective {
            Color::White => match self {
                GameOutcome::WhiteWin => 1.0,
                GameOutcome::BlackWin => -1.0,
                _ => 0.0,
            },
            Color::Black => match self {
                GameOutcome::WhiteWin => -1.0,
                GameOutcome::BlackWin => 1.0,
                _ => 0.0,
            },
        }
    }

    pub fn is_draw(&self) -> bool {
        !matches!(self, GameOutcome::WhiteWin | GameOutcome::BlackWin)
    }
}

impl fmt::Display for GameOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            GameOutcome::WhiteWin => "white_win",
            GameOutcome::BlackWin => "black_win",
            GameOutcome::Stalemate => "stalemate",
            GameOutcome::InsufficientMaterial => "insufficient_material",
            GameOutcome::ThreefoldRepetition => "threefold_repetition",
            GameOutcome::FiftyMoveRule => "fifty_move_rule",
            GameOutcome::Other => "other_draw",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winner() {
        assert_eq!(GameOutcome::WhiteWin.winner(), Some(Color::White));
        assert_eq!(GameOutcome::BlackWin.winner(), Some(Color::Black));
        assert_eq!(GameOutcome::Stalemate.winner(), None);
        assert_eq!(GameOutcome::InsufficientMaterial.winner(), None);
        assert_eq!(GameOutcome::ThreefoldRepetition.winner(), None);
        assert_eq!(GameOutcome::FiftyMoveRule.winner(), None);
        assert_eq!(GameOutcome::Other.winner(), None);
    }

    #[test]
    fn test_is_draw() {
        assert!(!GameOutcome::WhiteWin.is_draw());
        assert!(!GameOutcome::BlackWin.is_draw());
        assert!(GameOutcome::Stalemate.is_draw());
        assert!(GameOutcome::InsufficientMaterial.is_draw());
        assert!(GameOutcome::ThreefoldRepetition.is_draw());
        assert!(GameOutcome::FiftyMoveRule.is_draw());
        assert!(GameOutcome::Other.is_draw());
    }

    #[test]
    fn test_to_string() {
        assert_eq!(GameOutcome::WhiteWin.to_string(), "white_win");
        assert_eq!(GameOutcome::BlackWin.to_string(), "black_win");
        assert_eq!(GameOutcome::Stalemate.to_string(), "stalemate");
        assert_eq!(
            GameOutcome::InsufficientMaterial.to_string(),
            "insufficient_material"
        );
        assert_eq!(
            GameOutcome::ThreefoldRepetition.to_string(),
            "threefold_repetition"
        );
        assert_eq!(GameOutcome::FiftyMoveRule.to_string(), "fifty_move_rule");
        assert_eq!(GameOutcome::Other.to_string(), "other_draw");
    }
}
