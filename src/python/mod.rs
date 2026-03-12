use pyo3::prelude::*;

#[macro_use]
mod dispatch;

mod py_board;
mod py_game;
mod py_move;
mod py_outcome;
mod py_pgn;
mod py_piece;
mod py_position;
mod py_uci;

pub use py_board::PyBoard;
pub use py_game::PyGame;
pub use py_move::PyMove;
pub use py_outcome::PyGameOutcome;
pub use py_pgn::{PyPgnGame, py_parse_pgn};
pub use py_piece::PyPiece;
pub use py_position::PyPosition;
pub use py_uci::{PySearchResult, PyUciEngine};

pub(crate) fn validate_dimensions(width: usize, height: usize) -> PyResult<()> {
    if width < 5 || width > 16 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Board width must be between 5 and 16",
        ));
    }
    if height < 5 || height > 16 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Board height must be between 5 and 16",
        ));
    }
    Ok(())
}
