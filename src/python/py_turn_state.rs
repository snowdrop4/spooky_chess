use pyo3::prelude::*;

use super::py_move::PyMove;
use super::py_outcome::PyGameOutcome;
use crate::outcome::TurnState;

#[pyclass(name = "TurnState")]
#[derive(Clone, Debug)]
pub struct PyTurnState {
    pub(super) state: TurnState,
}

#[hotpath::measure_all]
#[pymethods]
impl PyTurnState {
    pub fn is_over(&self) -> bool {
        matches!(self.state, TurnState::Over(_))
    }

    pub fn outcome(&self) -> Option<PyGameOutcome> {
        match self.state {
            TurnState::Over(outcome) => Some(PyGameOutcome { outcome }),
            TurnState::Ongoing(_) => None,
        }
    }

    pub fn legal_moves(&self) -> Vec<PyMove> {
        match &self.state {
            TurnState::Over(_) => Vec::new(),
            TurnState::Ongoing(moves) => moves
                .iter()
                .copied()
                .map(|move_| PyMove { move_ })
                .collect(),
        }
    }

    pub fn __repr__(&self) -> String {
        match &self.state {
            TurnState::Over(outcome) => format!("TurnState(over={})", outcome),
            TurnState::Ongoing(moves) => format!("TurnState(ongoing_moves={})", moves.len()),
        }
    }
}
