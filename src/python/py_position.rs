use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::position::Position;

#[pyclass(name = "Position")]
#[derive(Clone, Copy, Debug)]
pub struct PyPosition {
    pub(super) pos: Position,
}

#[hotpath::measure_all]
#[pymethods]
impl PyPosition {
    #[new]
    pub fn new(col: u8, row: u8) -> Self {
        PyPosition {
            pos: Position::new(col, row),
        }
    }

    #[classmethod]
    pub fn from_algebraic(_cls: &Bound<'_, PyType>, s: &str) -> PyResult<Self> {
        Position::from_algebraic(s)
            .map(|pos| PyPosition { pos })
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))
    }

    pub fn to_algebraic(&self) -> String {
        self.pos.to_algebraic()
    }

    pub fn col(&self) -> u8 {
        self.pos.col
    }

    pub fn row(&self) -> u8 {
        self.pos.row
    }

    pub fn __str__(&self) -> String {
        self.pos.to_string()
    }

    pub fn __repr__(&self) -> String {
        format!("Position({}, {})", self.pos.col, self.pos.row)
    }

    pub fn __eq__(&self, other: &PyPosition) -> bool {
        self.pos == other.pos
    }

    pub fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.pos.hash(&mut hasher);
        hasher.finish()
    }
}
