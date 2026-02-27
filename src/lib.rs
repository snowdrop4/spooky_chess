pub mod bitboard;
pub mod board;
pub mod color;
pub mod encode;
pub mod game;
pub mod r#move;
pub mod outcome;
pub mod pieces;
pub mod position;

#[cfg(feature = "python")]
extern crate pyo3;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use pyo3::types::PyType;

#[cfg(feature = "python")]
#[pymodule(gil_used = false)]
fn spooky_chess(m: &Bound<'_, PyModule>) -> PyResult<()> {
    use color::Color;
    use python_bindings::*;
    m.add_class::<PyBoard>()?;
    m.add_class::<PyGame>()?;
    m.add_class::<PyMove>()?;
    m.add_class::<PyPiece>()?;
    m.add_class::<PyPosition>()?;
    m.add_class::<PyGameOutcome>()?;
    m.add("WHITE", Color::White as i8)?;
    m.add("BLACK", Color::Black as i8)?;
    m.add("TOTAL_INPUT_PLANES", encode::TOTAL_INPUT_PLANES)?;
    Ok(())
}

#[cfg(feature = "python")]
mod python_bindings {
    use super::*;
    use crate::bitboard::nw_for_board;
    use crate::board::Board;
    use crate::color::Color;
    use crate::game::Game;
    use crate::outcome::GameOutcome;
    use crate::pieces::{Piece, PieceType};
    use crate::position::Position;
    use crate::r#move::{Move, MoveFlags};

    // -----------------------------------------------------------------------
    // Enum dispatch via paste! for Game<NW> and Board<NW>
    // -----------------------------------------------------------------------

    macro_rules! define_dispatch {
        ($($nw:literal),*) => {
            paste::paste! {
                #[derive(Clone)]
                enum GameInner {
                    $( [<Nw $nw>](Game<$nw>), )*
                }

                #[derive(Clone)]
                enum BoardInner {
                    $( [<Nw $nw>](Board<$nw>), )*
                }

                macro_rules! dispatch_game {
                    ($self_:expr, $g:ident => $body:expr) => {
                        match $self_ {
                            $( GameInner::[<Nw $nw>]($g) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_game_mut {
                    ($self_:expr, $g:ident => $body:expr) => {
                        match $self_ {
                            $( GameInner::[<Nw $nw>]($g) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_board {
                    ($self_:expr, $b:ident => $body:expr) => {
                        match $self_ {
                            $( BoardInner::[<Nw $nw>]($b) => $body, )*
                        }
                    };
                }

                macro_rules! dispatch_board_mut {
                    ($self_:expr, $b:ident => $body:expr) => {
                        match $self_ {
                            $( BoardInner::[<Nw $nw>]($b) => $body, )*
                        }
                    };
                }

                fn make_game_inner(width: usize, height: usize, fen: &str, castling_enabled: bool) -> Result<GameInner, String> {
                    let nw = nw_for_board(width as u8, height as u8);
                    match nw {
                        $( $nw => Ok(GameInner::[<Nw $nw>](Game::new(width, height, fen, castling_enabled)?)), )*
                        _ => Err(format!("NW out of range: {}", nw)),
                    }
                }

                fn make_standard_game_inner() -> GameInner {
                    GameInner::Nw1(Game::standard())
                }

                fn make_board_inner(width: usize, height: usize, fen: &str) -> Result<BoardInner, String> {
                    let nw = nw_for_board(width as u8, height as u8);
                    match nw {
                        $( $nw => Ok(BoardInner::[<Nw $nw>](Board::new(width, height, fen)?)), )*
                        _ => Err(format!("NW out of range: {}", nw)),
                    }
                }

                fn make_empty_board_inner(width: usize, height: usize) -> Result<BoardInner, String> {
                    let nw = nw_for_board(width as u8, height as u8);
                    match nw {
                        $( $nw => Ok(BoardInner::[<Nw $nw>](Board::empty(width, height))), )*
                        _ => Err(format!("NW out of range: {}", nw)),
                    }
                }

                fn make_standard_board_inner() -> BoardInner {
                    BoardInner::Nw1(Board::standard())
                }
            }
        }
    }

    define_dispatch!(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);

    // -----------------------------------------------------------------------
    // PyBoard
    // -----------------------------------------------------------------------

    #[pyclass(name = "Board")]
    #[derive(Clone)]
    pub struct PyBoard {
        inner: BoardInner,
    }

    #[pymethods]
    impl PyBoard {
        #[new]
        pub fn new(width: usize, height: usize, fen: &str) -> PyResult<Self> {
            if width < 1 || width > 32 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 1 and 32",
                ));
            }
            if height < 1 || height > 32 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 1 and 32",
                ));
            }
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
            if width < 1 || width > 32 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board width must be between 1 and 32",
                ));
            }
            if height < 1 || height > 32 {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Board height must be between 1 and 32",
                ));
            }
            let inner = make_empty_board_inner(width, height)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;
            Ok(PyBoard { inner })
        }

        pub fn to_fen(&self) -> String {
            dispatch_board!(&self.inner, b => b.to_fen())
        }

        pub fn clear(&mut self) {
            dispatch_board_mut!(&mut self.inner, b => b.clear())
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
            dispatch_board_mut!(&mut self.inner, b => b.set_piece(&pos, piece.map(|p| p.piece)))
        }

        pub fn __str__(&self) -> String {
            dispatch_board!(&self.inner, b => b.to_string())
        }

        pub fn __repr__(&self) -> String {
            format!("Board(width={}, height={})", self.width(), self.height())
        }
    }

    // -----------------------------------------------------------------------
    // PyGame
    // -----------------------------------------------------------------------

    #[pyclass(name = "Game")]
    pub struct PyGame {
        inner: GameInner,
    }

    #[pymethods]
    impl PyGame {
        #[new]
        pub fn new(
            width: usize,
            height: usize,
            fen: &str,
            castling_enabled: bool,
        ) -> PyResult<Self> {
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

        pub fn has_kingside_castling_rights(&self, color: i8) -> bool {
            let color = if color == 1 {
                Color::White
            } else {
                Color::Black
            };
            dispatch_game!(&self.inner, g => g.castling_rights().has_kingside(color))
        }

        pub fn has_queenside_castling_rights(&self, color: i8) -> bool {
            let color = if color == 1 {
                Color::White
            } else {
                Color::Black
            };
            dispatch_game!(&self.inner, g => g.castling_rights().has_queenside(color))
        }

        pub fn make_move(&mut self, move_: PyMove) -> PyResult<bool> {
            Ok(dispatch_game_mut!(&mut self.inner, g => g.make_move(&move_.move_)))
        }

        pub fn unmake_move(&mut self) -> bool {
            dispatch_game_mut!(&mut self.inner, g => g.unmake_move())
        }

        pub fn is_legal_move(&self, move_: PyMove) -> bool {
            dispatch_game!(&self.inner, g => g.is_legal_move(&move_.move_))
        }

        pub fn legal_moves(&self) -> Vec<PyMove> {
            dispatch_game!(&self.inner, g => {
                g.legal_moves()
                    .into_iter()
                    .map(|m| PyMove { move_: m })
                    .collect()
            })
        }

        pub fn psuedo_legal_moves(&self) -> Vec<PyMove> {
            dispatch_game!(&self.inner, g => {
                g.psuedo_legal_moves()
                    .into_iter()
                    .map(|m| PyMove { move_: m })
                    .collect()
            })
        }

        pub fn legal_moves_for_position(&self, col: usize, row: usize) -> Vec<PyMove> {
            let pos = Position::new(col, row);
            dispatch_game!(&self.inner, g => {
                g.legal_moves_for_position(&pos)
                    .into_iter()
                    .map(|m| PyMove { move_: m })
                    .collect()
            })
        }

        pub fn move_from_lan(&self, lan: &str) -> PyResult<PyMove> {
            dispatch_game!(&self.inner, g => {
                match g.move_from_lan(lan) {
                    Ok(move_) => Ok(PyMove { move_ }),
                    Err(e) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(e)),
                }
            })
        }

        pub fn is_check(&self) -> bool {
            dispatch_game!(&self.inner, g => g.is_check())
        }

        pub fn is_checkmate(&self) -> bool {
            dispatch_game!(&self.inner, g => g.is_checkmate())
        }

        pub fn is_stalemate(&self) -> bool {
            dispatch_game!(&self.inner, g => g.is_stalemate())
        }

        pub fn is_over(&self) -> bool {
            dispatch_game!(&self.inner, g => g.is_over())
        }

        // ---------------------------------------------------------------------
        // Unified Game Protocol Methods
        // ---------------------------------------------------------------------

        pub fn width(&self) -> usize {
            dispatch_game!(&self.inner, g => g.board().width())
        }

        pub fn height(&self) -> usize {
            dispatch_game!(&self.inner, g => g.board().height())
        }

        pub fn get_piece(&self, col: usize, row: usize) -> Option<PyPiece> {
            let pos = Position::new(col, row);
            dispatch_game!(&self.inner, g => g.get_piece(&pos).map(|p| PyPiece { piece: p }))
        }

        pub fn set_piece(&mut self, col: usize, row: usize, piece: Option<PyPiece>) {
            let pos = Position::new(col, row);
            dispatch_game_mut!(&mut self.inner, g => g.set_piece(&pos, piece.map(|p| p.piece)))
        }

        pub fn legal_action_indices(&self) -> Vec<usize> {
            dispatch_game!(&self.inner, g => {
                let width = g.board().width();
                let height = g.board().height();
                g.legal_moves()
                    .into_iter()
                    .filter_map(|m| encode::encode_move(&m, width, height))
                    .collect()
            })
        }

        pub fn apply_action(&mut self, action: usize) -> bool {
            dispatch_game_mut!(&mut self.inner, g => {
                let width = g.board().width();
                let height = g.board().height();
                for move_ in g.legal_moves() {
                    if let Some(encoded) = encode::encode_move(&move_, width, height) {
                        if encoded == action {
                            return g.make_move(&move_);
                        }
                    }
                }
                false
            })
        }

        pub fn action_size(&self) -> usize {
            dispatch_game!(&self.inner, g => {
                encode::get_move_planes_count(g.board().width(), g.board().height())
            })
        }

        pub fn board_shape(&self) -> (usize, usize) {
            dispatch_game!(&self.inner, g => (g.board().height(), g.board().width()))
        }

        pub fn input_plane_count(&self) -> usize {
            encode::TOTAL_INPUT_PLANES
        }

        pub fn reward_absolute(&self) -> f32 {
            dispatch_game!(&self.inner, g => {
                g.outcome()
                    .map(|o| o.encode_winner_absolute())
                    .unwrap_or(0.0)
            })
        }

        pub fn reward_from_perspective(&self, perspective: i8) -> f32 {
            dispatch_game!(&self.inner, g => {
                g.outcome()
                    .map(|o| {
                        o.encode_winner_from_perspective(
                            Color::from_int(perspective).expect("Invalid perspective"),
                        )
                    })
                    .unwrap_or(0.0)
            })
        }

        pub fn name(&self) -> String {
            dispatch_game!(&self.inner, g => {
                format!(
                    "chess_{}x{}",
                    g.board().width(),
                    g.board().height()
                )
            })
        }

        pub fn is_insufficient_material(&self) -> bool {
            dispatch_game!(&self.inner, g => g.is_insufficient_material())
        }

        pub fn has_legal_en_passant(&self) -> bool {
            dispatch_game!(&self.inner, g => g.has_legal_en_passant())
        }

        pub fn en_passant_square(&self) -> Option<PyPosition> {
            dispatch_game!(&self.inner, g => g.en_passant_square().map(|pos| PyPosition { pos }))
        }

        pub fn outcome(&self) -> Option<PyGameOutcome> {
            dispatch_game!(&self.inner, g => g.outcome().map(|outcome| PyGameOutcome { outcome }))
        }

        pub fn to_fen(&self) -> String {
            dispatch_game!(&self.inner, g => g.to_fen())
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
                g.board().hash(&mut hasher);
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
        // Encoding/decoding
        // ---------------------------------------------------------------------

        pub fn encode_game_planes(&self) -> (Vec<f32>, usize, usize, usize) {
            dispatch_game!(&self.inner, g => encode::encode_game_planes(g))
        }

        #[staticmethod]
        pub fn get_move_planes_count(width: usize, height: usize) -> usize {
            encode::get_move_planes_count(width, height)
        }

        pub fn decode_action(&self, action: usize) -> Option<PyMove> {
            dispatch_game!(&self.inner, g => {
                let width = g.board().width();
                let height = g.board().height();
                for move_ in g.legal_moves() {
                    if let Some(encoded) = encode::encode_move(&move_, width, height) {
                        if encoded == action {
                            return Some(PyMove { move_ });
                        }
                    }
                }
                None
            })
        }

        // ---------------------------------------------------------------------
        // Dunder Methods
        // ---------------------------------------------------------------------

        pub fn __str__(&self) -> String {
            dispatch_game!(&self.inner, g => g.to_string())
        }

        pub fn __repr__(&self) -> String {
            dispatch_game!(&self.inner, g => {
                format!(
                    "Game(width={}, height={}, turn={:?}, over={})",
                    g.board().width(),
                    g.board().height(),
                    g.turn(),
                    g.is_over(),
                )
            })
        }
    }

    // -----------------------------------------------------------------------
    // Non-generic Python types
    // -----------------------------------------------------------------------

    #[pyclass(name = "Move")]
    #[derive(Clone, Debug)]
    pub struct PyMove {
        move_: Move,
    }

    #[pymethods]
    impl PyMove {
        #[staticmethod]
        pub fn from_rowcol(src_col: usize, src_row: usize, dst_col: usize, dst_row: usize) -> Self {
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

        pub fn src_square(&self) -> (usize, usize) {
            (self.move_.src.col, self.move_.src.row)
        }

        pub fn dst_square(&self) -> (usize, usize) {
            (self.move_.dst.col, self.move_.dst.row)
        }

        pub fn promotion(&self) -> Option<String> {
            if self.move_.flags.contains(MoveFlags::PROMOTION) {
                if let Some(promo) = self.move_.promotion {
                    let promo_char = match promo {
                        PieceType::Queen => "q",
                        PieceType::Rook => "r",
                        PieceType::Bishop => "b",
                        PieceType::Knight => "n",
                        _ => "q",
                    };
                    Some(promo_char.to_string())
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
        // Encoding/decoding
        // ---------------------------------------------------------------------

        pub fn encode(&self, width: usize, height: usize) -> Option<usize> {
            encode::encode_move(&self.move_, width, height)
        }

        #[staticmethod]
        pub fn decode_from_plane(
            plane_idx: usize,
            src_col: usize,
            src_row: usize,
            width: usize,
            height: usize,
        ) -> Option<Self> {
            encode::decode_move_from_plane(plane_idx, src_col, src_row, width, height)
                .map(|move_| PyMove { move_ })
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
            self.move_.src.col.hash(&mut hasher);
            self.move_.src.row.hash(&mut hasher);
            self.move_.dst.col.hash(&mut hasher);
            self.move_.dst.row.hash(&mut hasher);
            hasher.finish()
        }
    }

    #[pyclass(name = "Piece")]
    #[derive(Clone, Copy, Debug)]
    pub struct PyPiece {
        piece: Piece,
    }

    #[pymethods]
    impl PyPiece {
        #[new]
        pub fn new(piece_type: &str, color: i8) -> PyResult<Self> {
            let pt = match piece_type.to_lowercase().as_str() {
                "p" | "pawn" => PieceType::Pawn,
                "n" | "knight" => PieceType::Knight,
                "b" | "bishop" => PieceType::Bishop,
                "r" | "rook" => PieceType::Rook,
                "q" | "queen" => PieceType::Queen,
                "k" | "king" => PieceType::King,
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Invalid piece type",
                    ))
                }
            };

            let c = match color {
                1 => Color::White,
                -1 => Color::Black,
                _ => {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "Invalid color: must be 1 (White) or -1 (Black), got {}",
                        color
                    )))
                }
            };

            Ok(PyPiece {
                piece: Piece::new(pt, c),
            })
        }

        pub fn piece_type(&self) -> String {
            format!("{:?}", self.piece.piece_type).to_lowercase()
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

    #[pyclass(name = "Position")]
    #[derive(Clone, Copy, Debug)]
    pub struct PyPosition {
        pos: Position,
    }

    #[pymethods]
    impl PyPosition {
        #[new]
        pub fn new(col: usize, row: usize) -> Self {
            PyPosition {
                pos: Position::new(col, row),
            }
        }

        pub fn col(&self) -> usize {
            self.pos.col
        }

        pub fn row(&self) -> usize {
            self.pos.row
        }

        pub fn __str__(&self) -> String {
            self.pos.to_string()
        }

        pub fn __repr__(&self) -> String {
            format!("Position({}, {})", self.pos.col, self.pos.row)
        }
    }

    #[pyclass(name = "GameOutcome")]
    #[derive(Clone, Copy, Debug)]
    pub struct PyGameOutcome {
        outcome: GameOutcome,
    }

    #[pymethods]
    impl PyGameOutcome {
        pub fn winner(&self) -> Option<i8> {
            self.outcome.winner().map(|color| color as i8)
        }

        pub fn encode_winner_absolute(&self) -> f32 {
            self.outcome.encode_winner_absolute()
        }

        pub fn encode_winner_from_perspective(&self, perspective: i8) -> f32 {
            self.outcome.encode_winner_from_perspective(
                Color::from_int(perspective).expect("Unrecognized perspective"),
            )
        }

        pub fn is_draw(&self) -> bool {
            self.outcome.is_draw()
        }

        pub fn name(&self) -> String {
            self.outcome.to_string()
        }

        pub fn __str__(&self) -> String {
            self.outcome.to_string()
        }

        pub fn __repr__(&self) -> String {
            format!("GameOutcome({})", self.outcome.to_string())
        }

        pub fn __eq__(&self, other: &PyGameOutcome) -> bool {
            self.outcome == other.outcome
        }
    }
} // end python_bindings module
