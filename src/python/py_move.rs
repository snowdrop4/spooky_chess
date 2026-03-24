use pyo3::prelude::*;
use pyo3::types::PyType;

use crate::encode;
use crate::r#move::{Move, MoveFlags};
use crate::position::Position;

use super::py_position::PyPosition;

#[pyclass(name = "Move")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PyMove {
    pub(super) move_: Move,
}

#[hotpath::measure_all]
#[pymethods]
impl PyMove {
    #[staticmethod]
    pub fn from_rowcol(src_col: u8, src_row: u8, dst_col: u8, dst_row: u8) -> Self {
        PyMove {
            move_: Move::from_position(
                Position::new(src_col, src_row),
                Position::new(dst_col, dst_row),
                MoveFlags::empty(),
            ),
        }
    }

    #[classmethod]
    pub fn from_lan(
        _cls: &Bound<'_, PyType>,
        lan: &str,
        board_width: usize,
        board_height: usize,
    ) -> PyResult<Self> {
        match Move::from_lan(lan, board_width, board_height) {
            Ok(move_) => Ok(PyMove { move_ }),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)),
        }
    }

    #[getter]
    pub fn src(&self) -> PyPosition {
        PyPosition { pos: self.move_.src }
    }

    #[getter]
    pub fn dst(&self) -> PyPosition {
        PyPosition { pos: self.move_.dst }
    }

    pub fn src_square(&self) -> (u8, u8) {
        (self.move_.src.col, self.move_.src.row)
    }

    pub fn dst_square(&self) -> (u8, u8) {
        (self.move_.dst.col, self.move_.dst.row)
    }

    pub fn promotion(&self) -> Option<String> {
        if self.move_.flags.contains(MoveFlags::PROMOTION) {
            if let Some(promo) = self.move_.promotion {
                Some(promo.to_char().to_string())
            } else {
                Some("q".to_string())
            }
        } else {
            None
        }
    }

    pub fn to_lan(&self) -> String {
        self.move_.to_lan()
    }

    // ---------------------------------------------------------------------
    // Move flags
    // ---------------------------------------------------------------------

    #[getter]
    pub fn is_capture(&self) -> bool {
        self.move_.flags.contains(MoveFlags::CAPTURE)
    }

    #[getter]
    pub fn is_castling(&self) -> bool {
        self.move_.flags.contains(MoveFlags::CASTLE)
    }

    #[getter]
    pub fn is_en_passant(&self) -> bool {
        self.move_.flags.contains(MoveFlags::EN_PASSANT)
    }

    #[getter]
    pub fn is_promotion(&self) -> bool {
        self.move_.flags.contains(MoveFlags::PROMOTION)
    }

    #[getter]
    pub fn is_check(&self) -> bool {
        self.move_.flags.contains(MoveFlags::CHECK)
    }

    #[getter]
    pub fn is_double_push(&self) -> bool {
        self.move_.flags.contains(MoveFlags::DOUBLE_PUSH)
    }

    // ---------------------------------------------------------------------
    // Encoding/decoding
    // ---------------------------------------------------------------------

    pub fn encode(&self, width: usize, height: usize) -> Option<usize> {
        encode::encode_action(&self.move_, width, height)
    }

    // ---------------------------------------------------------------------
    // Dunder Methods
    // ---------------------------------------------------------------------

    pub fn __str__(&self) -> String {
        self.move_.to_lan()
    }

    pub fn __repr__(&self) -> String {
        format!("Move({})", self.move_.to_lan())
    }

    pub fn __eq__(&self, other: &PyMove) -> bool {
        self.move_ == other.move_
    }

    pub fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.move_.hash(&mut hasher);
        hasher.finish()
    }
}
