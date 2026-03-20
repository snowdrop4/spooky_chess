use pyo3::prelude::*;

use crate::uci::{SearchResult, UciEngine, UciError};

use super::py_move::PyMove;
use super::py_pgn::PyPgnGame;

fn uci_err_to_py(e: UciError) -> PyErr {
    match e {
        UciError::IoError(e) => PyErr::new::<pyo3::exceptions::PyOSError, _>(e.to_string()),
        UciError::ProtocolError(msg) => PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(msg),
        UciError::EngineExited => {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Engine process exited unexpectedly")
        }
        UciError::IllegalMove(msg) => PyErr::new::<pyo3::exceptions::PyValueError, _>(msg),
    }
}

#[pyclass(name = "SearchResult")]
#[derive(Clone)]
pub struct PySearchResult {
    #[pyo3(get)]
    pub best_move: PyMove,
    #[pyo3(get)]
    pub best_move_lan: String,
    #[pyo3(get)]
    pub ponder_move: Option<PyMove>,
    #[pyo3(get)]
    pub ponder_move_lan: Option<String>,
    #[pyo3(get)]
    pub score_cp: Option<i32>,
    #[pyo3(get)]
    pub score_mate: Option<i32>,
    #[pyo3(get)]
    pub depth: Option<u32>,
    #[pyo3(get)]
    pub nodes: Option<u64>,
    #[pyo3(get)]
    pub pv: Vec<String>,
}

#[pymethods]
impl PySearchResult {
    fn __repr__(&self) -> String {
        format!(
            "SearchResult(best_move={}, depth={:?}, score_cp={:?}, score_mate={:?})",
            self.best_move_lan, self.depth, self.score_cp, self.score_mate
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }
}

impl PySearchResult {
    fn from_rust(result: SearchResult) -> Self {
        // Extract score/depth/nodes from the last info line (deepest)
        let last_info = result.info.last();
        PySearchResult {
            best_move: PyMove {
                move_: result.best_move,
            },
            best_move_lan: result.best_move_lan,
            ponder_move: result.ponder_move.map(|m| PyMove { move_: m }),
            ponder_move_lan: result.ponder_move_lan,
            score_cp: last_info.and_then(|i| i.score_cp),
            score_mate: last_info.and_then(|i| i.score_mate),
            depth: last_info.and_then(|i| i.depth),
            nodes: last_info.and_then(|i| i.nodes),
            pv: last_info.map(|i| i.pv.clone()).unwrap_or_default(),
        }
    }
}

#[pyclass(name = "UciEngine")]
pub struct PyUciEngine {
    engine: Option<UciEngine>,
}

impl PyUciEngine {
    fn engine(&self) -> PyResult<&UciEngine> {
        self.engine.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("UCI engine has been shut down")
        })
    }

    fn engine_mut(&mut self) -> PyResult<&mut UciEngine> {
        self.engine.as_mut().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("UCI engine has been shut down")
        })
    }
}

#[pymethods]
impl PyUciEngine {
    #[new]
    #[pyo3(signature = (program, args=vec![]))]
    fn new(program: &str, args: Vec<String>) -> PyResult<Self> {
        let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let engine = UciEngine::new(program, &arg_refs).map_err(uci_err_to_py)?;
        Ok(PyUciEngine {
            engine: Some(engine),
        })
    }

    /// Get the engine's name (from UCI handshake).
    fn engine_name(&self) -> PyResult<Option<String>> {
        Ok(self.engine()?.engine_name().map(|s| s.to_string()))
    }

    /// Get the engine's author (from UCI handshake).
    fn engine_author(&self) -> PyResult<Option<String>> {
        Ok(self.engine()?.engine_author().map(|s| s.to_string()))
    }

    /// Set a UCI option.
    fn set_option(&mut self, name: &str, value: &str) -> PyResult<()> {
        self.engine_mut()?
            .set_option(name, value)
            .map_err(uci_err_to_py)
    }

    /// Send `isready` and wait for `readyok`.
    fn is_ready(&mut self) -> PyResult<()> {
        self.engine_mut()?.is_ready().map_err(uci_err_to_py)
    }

    /// Reset to standard starting position.
    fn set_position_startpos(&mut self) -> PyResult<()> {
        self.engine_mut()?.set_position_startpos();
        Ok(())
    }

    /// Set position from FEN string.
    fn set_position_fen(&mut self, fen: &str) -> PyResult<()> {
        self.engine_mut()?
            .set_position_fen(fen)
            .map_err(uci_err_to_py)
    }

    /// Set position from the starting position of a parsed PGN game.
    fn set_position_pgn_start(&mut self, pgn_game: &PyPgnGame) -> PyResult<()> {
        self.engine_mut()?
            .set_position_pgn_start(&pgn_game.inner)
            .map_err(uci_err_to_py)
    }

    /// Apply a move (by Move object). Returns True if legal and applied.
    fn make_move(&mut self, mv: PyMove) -> PyResult<bool> {
        self.engine_mut()?
            .make_move(&mv.move_)
            .map_err(uci_err_to_py)
    }

    /// Apply a move by LAN string (e.g. "e2e4"). Returns True if legal and applied.
    fn make_move_lan(&mut self, lan: &str) -> PyResult<bool> {
        self.engine_mut()?.make_move_lan(lan).map_err(uci_err_to_py)
    }

    /// Search to a given depth. Returns SearchResult.
    fn go_depth(&mut self, depth: u32) -> PyResult<PySearchResult> {
        let result = self.engine_mut()?.go_depth(depth).map_err(uci_err_to_py)?;
        Ok(PySearchResult::from_rust(result))
    }

    /// Search for a given time in milliseconds. Returns SearchResult.
    fn go_movetime(&mut self, ms: u64) -> PyResult<PySearchResult> {
        let result = self.engine_mut()?.go_movetime(ms).map_err(uci_err_to_py)?;
        Ok(PySearchResult::from_rust(result))
    }

    /// Search with clock parameters. Returns SearchResult.
    fn go_clock(
        &mut self,
        wtime: u64,
        btime: u64,
        winc: u64,
        binc: u64,
    ) -> PyResult<PySearchResult> {
        let result = self
            .engine_mut()?
            .go_clock(wtime, btime, winc, binc)
            .map_err(uci_err_to_py)?;
        Ok(PySearchResult::from_rust(result))
    }

    /// Search to depth, auto-apply bestmove, return the Move.
    fn go_bestmove_depth(&mut self, depth: u32) -> PyResult<PyMove> {
        let mv = self
            .engine_mut()?
            .go_bestmove_depth(depth)
            .map_err(uci_err_to_py)?;
        Ok(PyMove { move_: mv })
    }

    /// Search for movetime ms, auto-apply bestmove, return the Move.
    fn go_bestmove_movetime(&mut self, ms: u64) -> PyResult<PyMove> {
        let mv = self
            .engine_mut()?
            .go_bestmove_movetime(ms)
            .map_err(uci_err_to_py)?;
        Ok(PyMove { move_: mv })
    }

    /// Get the current turn (1=White, -1=Black).
    fn turn(&self) -> PyResult<i8> {
        Ok(self.engine()?.turn() as i8)
    }

    /// Check if the game is over.
    fn is_over(&mut self) -> PyResult<bool> {
        Ok(self.engine_mut()?.is_over())
    }

    /// Get legal moves from the current position.
    fn legal_moves(&mut self) -> PyResult<Vec<PyMove>> {
        Ok(self
            .engine_mut()?
            .legal_moves()
            .into_iter()
            .map(|m| PyMove { move_: m })
            .collect())
    }

    /// Undo the last move.
    fn undo(&mut self) -> PyResult<()> {
        self.engine_mut()?.undo();
        Ok(())
    }

    /// Send a raw UCI command string. Returns the next response line.
    fn send_command(&mut self, cmd: &str) -> PyResult<String> {
        self.engine_mut()?.send_command(cmd).map_err(uci_err_to_py)
    }

    /// Shut down the engine process.
    fn quit(&mut self) -> PyResult<()> {
        if let Some(mut engine) = self.engine.take() {
            let _ = engine.quit();
        }
        Ok(())
    }

    fn __repr__(&self) -> String {
        match &self.engine {
            Some(engine) => format!(
                "UciEngine(name={:?})",
                engine.engine_name().unwrap_or("unknown")
            ),
            None => "UciEngine(shut down)".to_string(),
        }
    }
}
