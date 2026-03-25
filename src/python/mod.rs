use pyo3::prelude::*;

use crate::limits;

#[macro_use]
mod dispatch;

mod py_game;
mod py_move;
mod py_outcome;
mod py_pgn;
mod py_piece;
mod py_position;
mod py_turn_state;
mod py_uci;

pub use py_game::PyGame;
pub use py_move::PyMove;
pub use py_outcome::PyGameOutcome;
pub use py_pgn::{PyPgnGame, py_parse_pgn};
pub use py_piece::PyPiece;
pub use py_position::PyPosition;
pub use py_turn_state::PyTurnState;
pub use py_uci::{PySearchResult, PyUciEngine};

pub(crate) fn validate_dimensions(width: usize, height: usize) -> PyResult<()> {
    limits::validate_board_dimensions(width, height)
        .map_err(PyErr::new::<pyo3::exceptions::PyValueError, _>)
}
