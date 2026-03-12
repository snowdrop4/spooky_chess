use pyo3::prelude::*;

use crate::color::Color;
use crate::pieces::{Piece, PieceType};

#[pyclass(name = "Piece")]
#[derive(Clone, Copy, Debug)]
pub struct PyPiece {
    pub(super) piece: Piece,
}

#[hotpath::measure_all]
#[pymethods]
impl PyPiece {
    #[new]
    pub fn new(piece_type: &str, color: i8) -> PyResult<Self> {
        let pt = piece_type
            .chars()
            .next()
            .and_then(PieceType::from_char)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid piece type"))?;

        let c = match color {
            1 => Color::White,
            -1 => Color::Black,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid color: must be 1 (White) or -1 (Black), got {}",
                    color
                )));
            }
        };

        Ok(PyPiece {
            piece: Piece::new(pt, c),
        })
    }

    pub fn color(&self) -> i8 {
        self.piece.color as i8
    }

    pub fn symbol(&self) -> String {
        self.piece.to_char().to_string()
    }

    pub fn __str__(&self) -> String {
        self.piece.to_char().to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Piece({:?}, {:?})", self.piece.piece_type, self.piece.color)
    }
}
