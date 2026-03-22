use pyo3::prelude::*;

use super::dispatch::*;
use super::py_move::PyMove;
use super::py_outcome::PyGameOutcome;
use super::py_piece::PyPiece;
use super::py_position::PyPosition;
use super::py_turn_state::PyTurnState;
use crate::color::Color;
use crate::encode;
use crate::position::Position;

#[pyclass(name = "Game")]
pub struct PyGame {
    pub(super) inner: GameInner,
}

#[hotpath::measure_all]
#[pymethods]
impl PyGame {
    #[new]
    pub fn new(width: usize, height: usize, fen: &str, castling_enabled: bool) -> PyResult<Self> {
        let inner = make_game_inner(width, height, fen, castling_enabled)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
        Ok(PyGame { inner })
    }

    #[staticmethod]
    pub fn standard() -> Self {
        PyGame {
            inner: make_standard_game_inner(),
        }
    }

    // ---------------------------------------------------------------------
    // Game Methods
    // ---------------------------------------------------------------------

    pub fn turn(&self) -> i8 {
        dispatch_game!(&self.inner, g => g.turn() as i8)
    }

    pub fn fullmove_number(&self) -> u32 {
        dispatch_game!(&self.inner, g => g.fullmove_number())
    }

    pub fn halfmove_clock(&self) -> u32 {
        dispatch_game!(&self.inner, g => g.halfmove_clock())
    }

    pub fn castling_enabled(&self) -> bool {
        dispatch_game!(&self.inner, g => g.castling_enabled())
    }

    pub fn has_kingside_castling_rights(&self, color: i8) -> PyResult<bool> {
        let color = Color::from_int(color).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("color must be 1 (white) or -1 (black)")
        })?;
        Ok(dispatch_game!(&self.inner, g => g.castling_rights().has_kingside(color)))
    }

    pub fn has_queenside_castling_rights(&self, color: i8) -> PyResult<bool> {
        let color = Color::from_int(color).ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>("color must be 1 (white) or -1 (black)")
        })?;
        Ok(dispatch_game!(&self.inner, g => g.castling_rights().has_queenside(color)))
    }

    pub fn make_move(&mut self, move_: PyMove) -> PyResult<bool> {
        Ok(dispatch_game!(&mut self.inner, g => g.make_move(&move_.move_)))
    }

    /// Apply a move that is already known to be legal. Skips legality checking.
    /// Caller must guarantee the move came from `legal_moves()` or equivalent.
    pub fn make_move_unchecked(&mut self, move_: PyMove) {
        dispatch_game!(&mut self.inner, g => g.make_move_unchecked(&move_.move_))
    }

    pub fn unmake_move(&mut self) -> bool {
        dispatch_game!(&mut self.inner, g => g.unmake_move())
    }

    pub fn is_legal_move(&mut self, move_: PyMove) -> bool {
        dispatch_game!(&mut self.inner, g => g.is_legal_move(&move_.move_))
    }

    pub fn legal_moves(&mut self) -> Vec<PyMove> {
        dispatch_game!(&mut self.inner, g => {
            g.legal_moves()
                .into_iter()
                .map(|m| PyMove { move_: m })
                .collect()
        })
    }

    pub fn pseudo_legal_moves(&self) -> Vec<PyMove> {
        dispatch_game!(&self.inner, g => {
            g.pseudo_legal_moves()
                .into_iter()
                .map(|m| PyMove { move_: m })
                .collect()
        })
    }

    pub fn legal_moves_for_position(&mut self, col: u8, row: u8) -> Vec<PyMove> {
        let pos = Position::new(col, row);
        dispatch_game!(&mut self.inner, g => {
            g.legal_moves_for_position(&pos)
                .into_iter()
                .map(|m| PyMove { move_: m })
                .collect()
        })
    }

    pub fn move_to_lan(&mut self, move_: PyMove) -> String {
        dispatch_game!(&mut self.inner, g => g.move_to_lan(&move_.move_))
    }

    pub fn move_from_lan(&self, lan: &str) -> PyResult<PyMove> {
        dispatch_game!(&self.inner, g => {
            match g.move_from_lan(lan) {
                Ok(move_) => Ok(PyMove { move_ }),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)),
            }
        })
    }

    pub fn move_to_san(&mut self, move_: PyMove) -> String {
        dispatch_game!(&mut self.inner, g => g.move_to_san(&move_.move_))
    }

    pub fn move_from_san(&mut self, san: &str) -> PyResult<PyMove> {
        dispatch_game!(&mut self.inner, g => {
            match g.move_from_san(san) {
                Ok(move_) => Ok(PyMove { move_ }),
                Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)),
            }
        })
    }

    pub fn is_check(&self) -> bool {
        dispatch_game!(&self.inner, g => g.is_check())
    }

    pub fn is_checkmate(&mut self) -> bool {
        dispatch_game!(&mut self.inner, g => g.is_checkmate())
    }

    pub fn is_stalemate(&mut self) -> bool {
        dispatch_game!(&mut self.inner, g => g.is_stalemate())
    }

    pub fn is_over(&mut self) -> bool {
        dispatch_game!(&mut self.inner, g => g.is_over())
    }

    // ---------------------------------------------------------------------
    // Unified Game Protocol Methods
    // ---------------------------------------------------------------------

    pub fn width(&self) -> usize {
        dispatch_game!(&self.inner, g => g.width())
    }

    pub fn height(&self) -> usize {
        dispatch_game!(&self.inner, g => g.height())
    }

    pub fn get_piece(&self, col: u8, row: u8) -> Option<PyPiece> {
        let pos = Position::new(col, row);
        dispatch_game!(&self.inner, g => g.get_piece(&pos).map(|p| PyPiece { piece: p }))
    }

    pub fn set_piece(&mut self, col: u8, row: u8, piece: Option<PyPiece>) {
        let pos = Position::new(col, row);
        dispatch_game!(&mut self.inner, g => g.set_piece(&pos, piece.map(|p| p.piece)))
    }

    pub fn legal_action_indices(&mut self) -> Vec<usize> {
        dispatch_game!(&mut self.inner, g => {
            let width = g.width();
            let height = g.height();
            g.legal_moves()
                .into_iter()
                .filter_map(|m| encode::encode_action(&m, width, height))
                .collect()
        })
    }

    pub fn apply_action(&mut self, action: usize) -> bool {
        dispatch_game!(&mut self.inner, g => g.apply_action(action))
    }

    // ---------------------------------------------------------------------
    // Encoding/decoding
    // ---------------------------------------------------------------------

    pub fn encode_game_planes(&mut self) -> (Vec<f32>, usize, usize, usize) {
        dispatch_game!(&mut self.inner, g => encode::encode_game_planes(g))
    }

    pub fn action_planes_count(&self) -> usize {
        dispatch_game!(&self.inner, g => {
            encode::get_move_planes_count(g.width(), g.height())
        })
    }

    pub fn decode_action(&self, action: usize) -> Option<PyMove> {
        dispatch_game!(&self.inner, g => {
            g.decode_action(action).map(|m| PyMove { move_: m })
        })
    }

    pub fn total_actions(&self) -> usize {
        dispatch_game!(&self.inner, g => {
            encode::get_total_actions(g.width(), g.height())
        })
    }

    pub fn board_shape(&self) -> (usize, usize) {
        dispatch_game!(&self.inner, g => (g.height(), g.width()))
    }

    pub fn input_plane_count(&self) -> usize {
        encode::TOTAL_INPUT_PLANES
    }

    pub fn reward_absolute(&mut self) -> f32 {
        dispatch_game!(&mut self.inner, g => {
            g.outcome()
                .map(|o| o.encode_winner_absolute())
                .unwrap_or(0.0)
        })
    }

    pub fn reward_from_perspective(&mut self, perspective: i8) -> f32 {
        dispatch_game!(&mut self.inner, g => {
            g.outcome()
                .map(|o| {
                    o.encode_winner_from_perspective(
                        Color::from_int(perspective).expect("Invalid perspective"),
                    )
                })
                .unwrap_or(0.0)
        })
    }

    pub fn is_insufficient_material(&self) -> bool {
        dispatch_game!(&self.inner, g => g.is_insufficient_material())
    }

    pub fn has_legal_en_passant(&mut self) -> bool {
        dispatch_game!(&mut self.inner, g => g.has_legal_en_passant())
    }

    pub fn en_passant_square(&self) -> Option<PyPosition> {
        dispatch_game!(&self.inner, g => g.en_passant_square().map(|pos| PyPosition { pos }))
    }

    pub fn outcome(&mut self) -> Option<PyGameOutcome> {
        dispatch_game!(&mut self.inner, g => g.outcome().map(|outcome| PyGameOutcome { outcome }))
    }

    pub fn turn_state(&mut self) -> PyTurnState {
        dispatch_game!(&mut self.inner, g => PyTurnState { state: g.turn_state() })
    }

    pub fn to_fen(&mut self) -> String {
        dispatch_game!(&mut self.inner, g => g.to_fen())
    }

    pub fn clone(&self) -> PyGame {
        PyGame {
            inner: self.inner.clone(),
        }
    }

    pub fn __hash__(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        dispatch_game!(&self.inner, g => {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            g.board_hash(&mut hasher);
            (g.turn() as i8).hash(&mut hasher);
            g.castling_rights().hash(&mut hasher);
            if let Some(ep_square) = g.en_passant_square() {
                ep_square.hash(&mut hasher);
            }
            g.halfmove_clock().hash(&mut hasher);
            hasher.finish()
        })
    }

    // ---------------------------------------------------------------------
    // Dunder Methods
    // ---------------------------------------------------------------------

    pub fn __str__(&self) -> String {
        dispatch_game!(&self.inner, g => g.to_string())
    }

    pub fn __repr__(&mut self) -> String {
        dispatch_game!(&mut self.inner, g => {
            format!(
                "Game(width={}, height={}, turn={:?}, over={})",
                g.width(),
                g.height(),
                g.turn(),
                g.is_over(),
            )
        })
    }
}
