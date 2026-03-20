use pyo3::prelude::*;

use super::dispatch::GameInner;
use super::py_game::PyGame;
use super::py_move::PyMove;

#[pyclass(name = "PgnGame")]
pub struct PyPgnGame {
    pub(super) inner: crate::pgn::PgnGame,
}

#[pymethods]
impl PyPgnGame {
    pub fn headers(&self) -> Vec<(String, String)> {
        self.inner.headers.pairs.clone()
    }

    pub fn header(&self, key: &str) -> Option<String> {
        self.inner.headers.get(key).map(|s| s.to_string())
    }

    pub fn white(&self) -> Option<String> {
        self.inner.headers.white().map(|s| s.to_string())
    }

    pub fn black(&self) -> Option<String> {
        self.inner.headers.black().map(|s| s.to_string())
    }

    pub fn result(&self) -> String {
        self.inner.result.to_string()
    }

    pub fn moves(&self) -> Vec<PyMove> {
        self.inner
            .moves
            .iter()
            .map(|m| PyMove { move_: *m })
            .collect()
    }

    pub fn starting_fen(&self) -> Option<String> {
        self.inner.starting_fen().map(str::to_string)
    }

    pub fn starting_game(&self) -> PyResult<PyGame> {
        let game = self
            .inner
            .starting_game()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(PyGame {
            inner: GameInner::W8H8(game),
        })
    }

    pub fn game(&self) -> PyGame {
        PyGame {
            inner: GameInner::W8H8(self.inner.final_game.clone()),
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "PgnGame({} vs {}, {}, {} moves)",
            self.inner.headers.white().unwrap_or("?"),
            self.inner.headers.black().unwrap_or("?"),
            self.inner.result,
            self.inner.moves.len(),
        )
    }
}

#[pyfunction(name = "parse_pgn")]
pub fn py_parse_pgn(pgn: &str) -> PyResult<Vec<PyPgnGame>> {
    crate::pgn::parse_pgn(pgn)
        .map(|games| games.into_iter().map(|g| PyPgnGame { inner: g }).collect())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}
