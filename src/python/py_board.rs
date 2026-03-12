use pyo3::prelude::*;

use super::dispatch::*;
use super::py_piece::PyPiece;
use super::validate_dimensions;
use crate::position::Position;

#[pyclass(name = "Board")]
#[derive(Clone)]
pub struct PyBoard {
    pub(super) inner: BoardInner,
}

#[hotpath::measure_all]
#[pymethods]
impl PyBoard {
    #[new]
    pub fn new(width: usize, height: usize, fen: &str) -> PyResult<Self> {
        validate_dimensions(width, height)?;
        let inner = make_board_inner(width, height, fen)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(PyBoard { inner })
    }

    #[staticmethod]
    pub fn standard() -> Self {
        PyBoard {
            inner: make_standard_board_inner(),
        }
    }

    #[staticmethod]
    pub fn empty(width: usize, height: usize) -> PyResult<Self> {
        validate_dimensions(width, height)?;
        let inner = make_empty_board_inner(width, height)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(PyBoard { inner })
    }

    pub fn to_fen(&self) -> String {
        dispatch_board!(&self.inner, b => b.to_fen())
    }

    pub fn clear(&mut self) {
        dispatch_board!(&mut self.inner, b => b.clear())
    }

    pub fn width(&self) -> usize {
        dispatch_board!(&self.inner, b => b.width())
    }

    pub fn height(&self) -> usize {
        dispatch_board!(&self.inner, b => b.height())
    }

    pub fn get_piece(&self, col: usize, row: usize) -> Option<PyPiece> {
        let pos = Position::new(col, row);
        dispatch_board!(&self.inner, b => b.get_piece(&pos).map(|p| PyPiece { piece: p }))
    }

    pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<PyPiece>) {
        let pos = Position::new(col, row);
        dispatch_board!(&mut self.inner, b => b.set_piece(&pos, piece.map(|p| p.piece)))
    }

    pub fn __str__(&self) -> String {
        dispatch_board!(&self.inner, b => b.to_string())
    }

    pub fn __repr__(&self) -> String {
        format!("Board(width={}, height={})", self.width(), self.height())
    }
}
