use crate::color::Color;
use crate::r#move::Move;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TurnState {
    Over(GameOutcome),
    Ongoing(Vec<Move>),
}

#[hotpath::measure_all]
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

#[hotpath::measure_all]
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
