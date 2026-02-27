use arrayvec::ArrayVec;

use crate::bitboard::nw_for_board;
use crate::board::{Board, STANDARD_COLS, STANDARD_ROWS};
use crate::color::Color;
use crate::outcome::GameOutcome;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use crate::r#move::{Move, MoveFlags};

#[derive(Clone)]
struct MoveHistoryEntry {
    mv: Move,
    captured: Option<Piece>,
    castling_rights: CastlingRights,
    en_passant: Option<Position>,
    halfmove_clock: u32,
}

#[derive(Clone)]
pub struct Game<const NW: usize> {
    board: Board<NW>,
    turn: Color,
    move_history: Vec<MoveHistoryEntry>,

    castling_rights: CastlingRights,
    castling_enabled: bool,

    en_passant: Option<Position>,

    halfmove_clock: u32,
    fullmove_number: u32,

    white_king_pos: Position,
    black_king_pos: Position,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CastlingRights {
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self::new()
    }
}

impl CastlingRights {
    pub fn new() -> Self {
        CastlingRights {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }

    pub fn none() -> Self {
        CastlingRights {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    pub fn has_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_kingside,
            Color::Black => self.black_kingside,
        }
    }

    pub fn has_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.white_queenside,
            Color::Black => self.black_queenside,
        }
    }
}

impl<const NW: usize> Game<NW> {
    pub fn new(
        width: usize,
        height: usize,
        fen: &str,
        castling_enabled: bool,
    ) -> Result<Self, String> {
        let parts: ArrayVec<&str, 6> = fen.split(' ').collect();

        if parts.is_empty() {
            return Err("Empty FEN string".to_string());
        }

        // FEN must have exactly 6 parts: position, turn, castling, en_passant, halfmove, fullmove
        if parts.len() != 6 {
            return Err(format!(
                "Invalid FEN: expected 6 parts, got {}",
                parts.len()
            ));
        }

        let board = Board::new(width, height, parts[0])?;

        // Turn
        let turn = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid turn in FEN".to_string()),
        };

        // Castling rights
        let mut castling_rights = CastlingRights::none();
        if castling_enabled {
            for c in parts[2].chars() {
                match c {
                    'K' => castling_rights.white_kingside = true,
                    'Q' => castling_rights.white_queenside = true,
                    'k' => castling_rights.black_kingside = true,
                    'q' => castling_rights.black_queenside = true,
                    '-' => {}
                    _ => return Err("Invalid castling rights in FEN".to_string()),
                }
            }
        }

        // En passant
        let en_passant = if parts[3] != "-" {
            Some(Position::from_algebraic(parts[3])?)
        } else {
            None
        };

        // Halfmove clock
        let halfmove_clock = parts[4]
            .parse()
            .map_err(|_| "Invalid halfmove clock in FEN".to_string())?;

        // Fullmove number
        let fullmove_number = parts[5]
            .parse()
            .map_err(|_| "Invalid fullmove number in FEN".to_string())?;

        // Find king positions
        let white_king_pos = board
            .find_king(Color::White)
            .ok_or("No white king found in FEN position".to_string())?;
        let black_king_pos = board
            .find_king(Color::Black)
            .ok_or("No black king found in FEN position".to_string())?;

        Ok(Game {
            board,
            turn,
            move_history: Vec::new(),
            castling_rights,
            castling_enabled,
            en_passant,
            halfmove_clock,
            fullmove_number,
            white_king_pos,
            black_king_pos,
        })
    }

    pub fn width(&self) -> usize {
        self.board.width()
    }

    pub fn height(&self) -> usize {
        self.board.height()
    }

    pub fn get_piece(&self, pos: &Position) -> Option<Piece> {
        self.board.get_piece(pos)
    }

    pub fn set_piece(&mut self, pos: &Position, piece: Option<Piece>) {
        self.board.set_piece(pos, piece)
    }

    pub fn board(&self) -> &Board<NW> {
        &self.board
    }

    pub fn board_mut(&mut self) -> &mut Board<NW> {
        &mut self.board
    }

    pub fn turn(&self) -> Color {
        self.turn
    }

    pub fn fullmove_number(&self) -> u32 {
        self.fullmove_number
    }

    pub fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    pub fn move_count(&self) -> usize {
        self.move_history.len()
    }

    pub fn move_history(&self) -> Vec<Move> {
        self.move_history.iter().map(|e| e.mv).collect()
    }

    pub fn castling_enabled(&self) -> bool {
        self.castling_enabled
    }

    pub fn castling_rights(&self) -> &CastlingRights {
        &self.castling_rights
    }

    pub fn board_clear(&mut self) {
        self.board.clear();
    }

    /// Make a move without validating whether it's legal.
    /// Used for testing moves during legal move generation.
    fn make_move_without_legality_checking(&mut self, mv: &Move) -> bool {
        // Validate the move is from a piece
        let piece = match self.board.get_piece(&mv.src) {
            Some(p) => p,
            _ => return false,
        };

        // Store state for unmake
        let captured = self.board.get_piece(&mv.dst);
        let old_castling = self.castling_rights;
        let old_en_passant = self.en_passant;
        let old_halfmove = self.halfmove_clock;

        // Make the move on the board
        self.board.set_piece(&mv.src, None);

        // Handle promotion
        if mv.flags.contains(MoveFlags::PROMOTION) {
            let promo_piece = Piece::new(mv.promotion.unwrap_or(PieceType::Queen), piece.color);
            self.board.set_piece(&mv.dst, Some(promo_piece));
        } else {
            self.board.set_piece(&mv.dst, Some(piece));
        }

        // Update king position if a king moved
        if piece.piece_type == PieceType::King {
            match piece.color {
                Color::White => self.white_king_pos = mv.dst,
                Color::Black => self.black_king_pos = mv.dst,
            }
        }

        // Handle en passant capture
        if mv.flags.contains(MoveFlags::EN_PASSANT) {
            let captured_pawn_pos = Position::new(mv.dst.col, mv.src.row);
            self.board.set_piece(&captured_pawn_pos, None);
        }

        // Reset en passant square
        self.en_passant = None;

        // Store move in history for potential unmake
        self.move_history.push(MoveHistoryEntry {
            mv: *mv,
            captured,
            castling_rights: old_castling,
            en_passant: old_en_passant,
            halfmove_clock: old_halfmove,
        });

        true
    }

    /// Returns: whether the move was successfully made
    pub fn make_move(&mut self, mv: &Move) -> bool {
        // Validate the move is from a piece of the correct color
        let piece = match self.board.get_piece(&mv.src) {
            Some(p) if p.color == self.turn => p,
            _ => return false,
        };

        // Check if the move is legal
        if !self.is_legal_move(mv) {
            return false;
        }

        // Store state for unmake
        let captured = self.board.get_piece(&mv.dst);
        let old_castling = self.castling_rights;
        let old_en_passant = self.en_passant;
        let old_halfmove = self.halfmove_clock;

        // Make the move on the board
        self.board.set_piece(&mv.src, None);

        // Handle promotion
        if mv.flags.contains(MoveFlags::PROMOTION) {
            let promo_piece = Piece::new(mv.promotion.unwrap_or(PieceType::Queen), piece.color);
            self.board.set_piece(&mv.dst, Some(promo_piece));
        } else {
            self.board.set_piece(&mv.dst, Some(piece));
        }

        // Update king position if a king moved
        if piece.piece_type == PieceType::King {
            match piece.color {
                Color::White => self.white_king_pos = mv.dst,
                Color::Black => self.black_king_pos = mv.dst,
            }
        }

        // Handle en passant capture
        if mv.flags.contains(MoveFlags::EN_PASSANT) {
            let captured_pawn_pos = Position::new(mv.dst.col, mv.src.row);
            self.board.set_piece(&captured_pawn_pos, None);
        }

        // Handle castling
        if mv.flags.contains(MoveFlags::CASTLE) {
            let (rook_from, rook_to) = if mv.dst.col > mv.src.col {
                // Kingside
                (
                    Position::new(self.board.width() - 1, mv.src.row),
                    Position::new(mv.dst.col - 1, mv.dst.row),
                )
            } else {
                // Queenside
                (
                    Position::new(0, mv.src.row),
                    Position::new(mv.dst.col + 1, mv.dst.row),
                )
            };

            if let Some(rook) = self.board.get_piece(&rook_from) {
                self.board.set_piece(&rook_from, None);
                self.board.set_piece(&rook_to, Some(rook));
            }
        }

        // Update castling rights
        self.update_castling_rights(mv, &piece);

        // Update en passant square
        self.en_passant = None;
        if piece.piece_type == PieceType::Pawn && (mv.dst.row as i32 - mv.src.row as i32).abs() == 2
        {
            // Set en passant square immediately after double pawn push
            let ep_row = (mv.src.row + mv.dst.row) / 2;
            self.en_passant = Some(Position::new(mv.src.col, ep_row));
        }

        // Update clocks
        if piece.piece_type == PieceType::Pawn || captured.is_some() {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        if self.turn == Color::Black {
            self.fullmove_number += 1;
        }

        // Store move in history
        self.move_history.push(MoveHistoryEntry {
            mv: *mv,
            captured,
            castling_rights: old_castling,
            en_passant: old_en_passant,
            halfmove_clock: old_halfmove,
        });

        // Switch turns (always, even if the game is over)
        self.turn = self.turn.opposite();

        true
    }

    pub fn unmake_move(&mut self) -> bool {
        if let Some(entry) = self.move_history.pop() {
            let mv = entry.mv;
            let captured = entry.captured;
            let old_castling = entry.castling_rights;
            let old_en_passant = entry.en_passant;
            let old_halfmove = entry.halfmove_clock;

            // Switch turn back
            self.turn = self.turn.opposite();

            // Restore piece to original position
            let piece = self.board.get_piece(&mv.dst);

            // Handle promotion - restore original pawn
            if mv.flags.contains(MoveFlags::PROMOTION) {
                self.board
                    .set_piece(&mv.src, Some(Piece::new(PieceType::Pawn, self.turn)));
            } else {
                self.board.set_piece(&mv.src, piece);
            }

            // Restore king position if a king moved
            if let Some(piece) = piece {
                if piece.piece_type == PieceType::King {
                    match piece.color {
                        Color::White => self.white_king_pos = mv.src,
                        Color::Black => self.black_king_pos = mv.src,
                    }
                }
            }

            self.board.set_piece(&mv.dst, captured);

            // Handle en passant
            if mv.flags.contains(MoveFlags::EN_PASSANT) {
                let captured_pawn_pos = Position::new(mv.dst.col, mv.src.row);
                self.board.set_piece(
                    &captured_pawn_pos,
                    Some(Piece::new(PieceType::Pawn, self.turn.opposite())),
                );
            }

            // Handle castling
            if mv.flags.contains(MoveFlags::CASTLE) {
                let (rook_from, rook_to) = if mv.dst.col > mv.src.col {
                    // Kingside
                    (
                        Position::new(self.board.width() - 1, mv.src.row),
                        Position::new(mv.dst.col - 1, mv.dst.row),
                    )
                } else {
                    // Queenside
                    (
                        Position::new(0, mv.src.row),
                        Position::new(mv.dst.col + 1, mv.dst.row),
                    )
                };

                if let Some(rook) = self.board.get_piece(&rook_to) {
                    self.board.set_piece(&rook_to, None);
                    self.board.set_piece(&rook_from, Some(rook));
                }
            }

            // Restore state
            self.castling_rights = old_castling;
            self.en_passant = old_en_passant;
            self.halfmove_clock = old_halfmove;

            if self.turn == Color::Black {
                self.fullmove_number -= 1;
            }

            true
        } else {
            false
        }
    }

    fn update_castling_rights(&mut self, mv: &Move, piece: &Piece) {
        // King moves
        if piece.piece_type == PieceType::King {
            match piece.color {
                Color::White => {
                    self.castling_rights.white_kingside = false;
                    self.castling_rights.white_queenside = false;
                }
                Color::Black => {
                    self.castling_rights.black_kingside = false;
                    self.castling_rights.black_queenside = false;
                }
            }
        }

        // Rook moves or captures
        let size = self.board.width();
        if piece.piece_type == PieceType::Rook {
            if mv.src.row == 0 && mv.src.col == 0 {
                self.castling_rights.white_queenside = false;
            } else if mv.src.row == 0 && mv.src.col == size - 1 {
                self.castling_rights.white_kingside = false;
            } else if mv.src.row == size - 1 && mv.src.col == 0 {
                self.castling_rights.black_queenside = false;
            } else if mv.src.row == size - 1 && mv.src.col == size - 1 {
                self.castling_rights.black_kingside = false;
            }
        }

        // Rook captures
        if mv.dst.row == 0 && mv.dst.col == 0 {
            self.castling_rights.white_queenside = false;
        } else if mv.dst.row == 0 && mv.dst.col == size - 1 {
            self.castling_rights.white_kingside = false;
        } else if mv.dst.row == size - 1 && mv.dst.col == 0 {
            self.castling_rights.black_queenside = false;
        } else if mv.dst.row == size - 1 && mv.dst.col == size - 1 {
            self.castling_rights.black_kingside = false;
        }
    }

    pub fn is_legal_move(&self, mv: &Move) -> bool {
        // Basic validation
        if let Some(piece) = self.board.get_piece(&mv.src) {
            if piece.color != self.turn {
                return false;
            }

            // Check if move is in legal moves list
            let legal = self.legal_moves_for_position(&mv.src);
            legal.iter().any(|m| m.src == mv.src && m.dst == mv.dst)
        } else {
            false
        }
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        for (pos, _piece) in self.board.pieces(self.turn) {
            let piece_moves = self.legal_moves_for_position(&pos);
            moves.extend(piece_moves);
        }

        moves
    }

    pub fn psuedo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        for (pos, _piece) in self.board.pieces(self.turn) {
            let piece_moves = self.psuedo_legal_moves_for_position(&pos);
            moves.extend(piece_moves);
        }

        moves
    }

    pub fn legal_moves_for_position(&self, src: &Position) -> Vec<Move> {
        let mut moves = Vec::new();

        if let Some(piece) = self.board.get_piece(src) {
            if piece.color != self.turn {
                return moves;
            }

            let pseudo_legal = self.generate_pseudo_legal_moves_for_piece(src, &piece);

            // Filter out moves that leave king in check
            for mv in pseudo_legal {
                let mut test_game = self.clone();
                test_game.board.set_piece(&mv.src, None);

                if mv.flags.contains(MoveFlags::PROMOTION) {
                    let promo_piece =
                        Piece::new(mv.promotion.unwrap_or(PieceType::Queen), piece.color);
                    test_game.board.set_piece(&mv.dst, Some(promo_piece));
                } else {
                    test_game.board.set_piece(&mv.dst, Some(piece));
                }

                // Update king position in test game if a king moved
                if piece.piece_type == PieceType::King {
                    match piece.color {
                        Color::White => test_game.white_king_pos = mv.dst,
                        Color::Black => test_game.black_king_pos = mv.dst,
                    }
                }

                // Handle en passant capture
                if mv.flags.contains(MoveFlags::EN_PASSANT) {
                    let captured_pawn_pos = Position::new(mv.dst.col, mv.src.row);
                    test_game.board.set_piece(&captured_pawn_pos, None);
                }

                if !test_game.is_in_check(self.turn) {
                    moves.push(mv);
                }
            }
        }

        moves
    }

    fn psuedo_legal_moves_for_position(&self, src: &Position) -> Vec<Move> {
        let mut moves = Vec::new();

        if let Some(piece) = self.board.get_piece(src) {
            if piece.color != self.turn {
                return moves;
            }

            // Generate pseudo-legal moves without check filtering
            let pseudo_legal = self.generate_pseudo_legal_moves_for_piece(src, &piece);
            moves.extend(pseudo_legal);
        }

        moves
    }

    fn generate_pseudo_legal_moves_for_piece(&self, src: &Position, piece: &Piece) -> Vec<Move> {
        match piece.piece_type {
            PieceType::Pawn => self.generate_psuedo_legal_pawn_moves(src, piece),
            PieceType::Knight => self.generate_psuedo_legal_knight_moves(src, piece),
            PieceType::Bishop => self.generate_psuedo_legal_bishop_moves(src, piece),
            PieceType::Rook => self.generate_psuedo_legal_rook_moves(src, piece),
            PieceType::Queen => self.generate_psuedo_legal_queen_moves(src, piece),
            PieceType::King => self.generate_psuedo_legal_king_moves(src, piece),
        }
    }

    fn generate_psuedo_legal_pawn_moves(&self, src: &Position, piece: &Piece) -> Vec<Move> {
        let mut moves = Vec::new();

        let direction = if piece.color == Color::White {
            1i32
        } else {
            -1i32
        };
        let start_row = if piece.color == Color::White {
            1
        } else {
            self.board.height() - 2
        };
        let promo_row = if piece.color == Color::White {
            self.board.height() - 2
        } else {
            1
        };

        // Single push
        let dst_row = (src.row as i32 + direction) as usize;
        if dst_row < self.board.height() {
            let dst_position = Position::new(src.col, dst_row);

            if self.board.get_piece(&dst_position).is_none() {
                if src.row == promo_row {
                    // Promotion
                    for piece_type in &[
                        PieceType::Queen,
                        PieceType::Rook,
                        PieceType::Bishop,
                        PieceType::Knight,
                    ] {
                        moves.push(Move::from_position_with_promotion(
                            *src,
                            dst_position,
                            MoveFlags::PROMOTION,
                            *piece_type,
                        ));
                    }
                } else {
                    moves.push(Move::from_position(*src, dst_position, MoveFlags::empty()));
                }
            }
        }

        // Double push from starting position
        if src.row == start_row {
            let to_row = (src.row as i32 + 2 * direction) as usize;
            let to = Position::new(src.col, to_row);
            let between = Position::new(src.col, (src.row as i32 + direction) as usize);

            if self.board.get_piece(&to).is_none() && self.board.get_piece(&between).is_none() {
                moves.push(Move::from_position(*src, to, MoveFlags::DOUBLE_PUSH));
            }
        }

        // Captures
        for col_offset in &[-1i32, 1i32] {
            let dst_col = (src.col as i32 + col_offset) as usize;
            let dst_row = (src.row as i32 + direction) as usize;

            if dst_col < self.board.width() && dst_row < self.board.height() {
                let dst_position = Position::new(dst_col, dst_row);

                if let Some(target) = self.board.get_piece(&dst_position) {
                    if target.color != piece.color {
                        if src.row == promo_row {
                            // Capture with promotion
                            for piece_type in &[
                                PieceType::Queen,
                                PieceType::Rook,
                                PieceType::Bishop,
                                PieceType::Knight,
                            ] {
                                moves.push(Move::from_position_with_promotion(
                                    *src,
                                    dst_position,
                                    MoveFlags::CAPTURE | MoveFlags::PROMOTION,
                                    *piece_type,
                                ));
                            }
                        } else {
                            moves.push(Move::from_position(*src, dst_position, MoveFlags::CAPTURE));
                        }
                    }
                }

                // En passant
                if let Some(ep) = self.en_passant {
                    if ep == dst_position {
                        moves.push(Move::from_position(
                            *src,
                            dst_position,
                            MoveFlags::CAPTURE | MoveFlags::EN_PASSANT,
                        ));
                    }
                }
            }
        }

        moves
    }

    fn generate_psuedo_legal_knight_moves(&self, src: &Position, piece: &Piece) -> Vec<Move> {
        let mut moves = Vec::new();

        let offsets = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];

        for (col_offset, row_offset) in &offsets {
            let dst_col = (src.col as i32 + col_offset) as usize;
            let dst_row = (src.row as i32 + row_offset) as usize;

            if dst_col < self.board.width() && dst_row < self.board.height() {
                let to = Position::new(dst_col, dst_row);

                if let Some(target) = self.board.get_piece(&to) {
                    if target.color != piece.color {
                        moves.push(Move::from_position(*src, to, MoveFlags::CAPTURE));
                    }
                } else {
                    moves.push(Move::from_position(*src, to, MoveFlags::empty()));
                }
            }
        }

        moves
    }

    fn generate_sliding_moves(
        &self,
        src: &Position,
        piece: &Piece,
        directions: &[(i32, i32)],
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        for (col_dir, row_dir) in directions {
            let mut distance = 1;

            loop {
                let dst_col = (src.col as i32 + col_dir * distance) as usize;
                let dst_row = (src.row as i32 + row_dir * distance) as usize;

                if dst_col >= self.board.width() || dst_row >= self.board.height() {
                    break;
                }

                let dst_position = Position::new(dst_col, dst_row);

                if let Some(target) = self.board.get_piece(&dst_position) {
                    if target.color != piece.color {
                        moves.push(Move::from_position(*src, dst_position, MoveFlags::CAPTURE));
                    }
                    break;
                } else {
                    moves.push(Move::from_position(*src, dst_position, MoveFlags::empty()));
                }

                distance += 1;
            }
        }

        moves
    }

    fn generate_psuedo_legal_bishop_moves(&self, src: &Position, piece: &Piece) -> Vec<Move> {
        let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        self.generate_sliding_moves(src, piece, &directions)
    }

    fn generate_psuedo_legal_rook_moves(&self, src: &Position, piece: &Piece) -> Vec<Move> {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        self.generate_sliding_moves(src, piece, &directions)
    }

    fn generate_psuedo_legal_queen_moves(&self, src: &Position, piece: &Piece) -> Vec<Move> {
        let directions = [
            (0, 1),
            (0, -1),
            (1, 0),
            (-1, 0),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];
        self.generate_sliding_moves(src, piece, &directions)
    }

    fn generate_psuedo_legal_king_moves(&self, src: &Position, piece: &Piece) -> Vec<Move> {
        let mut moves = Vec::new();

        // Regular moves
        for col_offset in -1..=1 {
            for row_offset in -1..=1 {
                if col_offset == 0 && row_offset == 0 {
                    continue;
                }

                let dst_col = (src.col as i32 + col_offset) as usize;
                let dst_row = (src.row as i32 + row_offset) as usize;

                if dst_col < self.board.width() && dst_row < self.board.height() {
                    let dst_position = Position::new(dst_col, dst_row);

                    if let Some(target) = self.board.get_piece(&dst_position) {
                        if target.color != piece.color {
                            moves.push(Move::from_position(*src, dst_position, MoveFlags::CAPTURE));
                        }
                    } else {
                        moves.push(Move::from_position(*src, dst_position, MoveFlags::empty()));
                    }
                }
            }
        }

        // Castling (only if enabled and for 8x8 boards)
        if self.castling_enabled
            && self.board.width() == 8
            && self.board.height() == 8
            && !self.is_in_check(piece.color)
        {
            let row = if piece.color == Color::White { 0 } else { 7 };

            // Kingside
            if ((piece.color == Color::White && self.castling_rights.white_kingside)
                || (piece.color == Color::Black && self.castling_rights.black_kingside))
                && self.board.get_piece(&Position::new(5, row)).is_none()
                && self.board.get_piece(&Position::new(6, row)).is_none()
            {
                // Check if squares are not attacked
                let mut can_castle = true;

                for col in 5..=6 {
                    if self.is_square_attacked(&Position::new(col, row), piece.color.opposite()) {
                        can_castle = false;
                        break;
                    }
                }

                if can_castle {
                    moves.push(Move::from_position(
                        *src,
                        Position::new(6, row),
                        MoveFlags::CASTLE,
                    ));
                }
            }

            // Queenside
            if ((piece.color == Color::White && self.castling_rights.white_queenside)
                || (piece.color == Color::Black && self.castling_rights.black_queenside))
                && self.board.get_piece(&Position::new(1, row)).is_none()
                && self.board.get_piece(&Position::new(2, row)).is_none()
                && self.board.get_piece(&Position::new(3, row)).is_none()
            {
                // Check if squares are not attacked
                let mut can_castle = true;

                for col in 2..=4 {
                    if self.is_square_attacked(&Position::new(col, row), piece.color.opposite()) {
                        can_castle = false;
                        break;
                    }
                }

                if can_castle {
                    moves.push(Move::from_position(
                        *src,
                        Position::new(2, row),
                        MoveFlags::CASTLE,
                    ));
                }
            }
        }

        moves
    }

    fn is_square_attacked(&self, square: &Position, by_color: Color) -> bool {
        for (pos, piece) in self.board.pieces(by_color) {
            if self.can_piece_attack(&pos, &piece, square) {
                return true;
            }
        }

        false
    }

    fn can_piece_attack(&self, src: &Position, piece: &Piece, dst: &Position) -> bool {
        match piece.piece_type {
            PieceType::Pawn => self.can_pawn_attack(src, piece, dst),
            PieceType::Knight => self.can_knight_attack(src, dst),
            PieceType::Bishop => self.can_bishop_attack(src, dst),
            PieceType::Rook => self.can_rook_attack(src, dst),
            PieceType::Queen => self.can_queen_attack(src, dst),
            PieceType::King => self.can_king_attack(src, dst),
        }
    }

    fn can_pawn_attack(&self, src: &Position, piece: &Piece, dst: &Position) -> bool {
        let direction = if piece.color == Color::White {
            1i32
        } else {
            -1i32
        };
        let target_row = src.row as i32 + direction;

        if target_row < 0 || target_row >= self.board.height() as i32 {
            return false;
        }

        if dst.row != target_row as usize {
            return false;
        }

        let col_diff = (dst.col as i32 - src.col as i32).abs();
        col_diff == 1
    }

    fn can_knight_attack(&self, src: &Position, dst: &Position) -> bool {
        let col_diff = (dst.col as i32 - src.col as i32).abs();
        let row_diff = (dst.row as i32 - src.row as i32).abs();

        (col_diff == 2 && row_diff == 1) || (col_diff == 1 && row_diff == 2)
    }

    fn can_bishop_attack(&self, src: &Position, dst: &Position) -> bool {
        let col_diff = (dst.col as i32 - src.col as i32).abs();
        let row_diff = (dst.row as i32 - src.row as i32).abs();

        if col_diff != row_diff || col_diff == 0 {
            return false;
        }

        // Check if path is clear
        let col_dir = if dst.col > src.col { 1 } else { -1 };
        let row_dir = if dst.row > src.row { 1 } else { -1 };

        for i in 1..col_diff {
            let check_col = src.col as i32 + i * col_dir;
            let check_row = src.row as i32 + i * row_dir;

            if check_col < 0
                || check_col >= self.board.width() as i32
                || check_row < 0
                || check_row >= self.board.height() as i32
            {
                return false;
            }

            let check_pos = Position::new(check_col as usize, check_row as usize);

            if self.board.get_piece(&check_pos).is_some() {
                return false;
            }
        }

        true
    }

    fn can_rook_attack(&self, src: &Position, dst: &Position) -> bool {
        if src.col != dst.col && src.row != dst.row {
            return false;
        }

        // Check if path is clear
        if src.col == dst.col {
            let start_row = src.row.min(dst.row) + 1;
            let end_row = src.row.max(dst.row);

            for row in start_row..end_row {
                let check_pos = Position::new(src.col, row);
                if self.board.get_piece(&check_pos).is_some() {
                    return false;
                }
            }
        } else {
            let start_col = src.col.min(dst.col) + 1;
            let end_col = src.col.max(dst.col);

            for col in start_col..end_col {
                let check_pos = Position::new(col, src.row);
                if self.board.get_piece(&check_pos).is_some() {
                    return false;
                }
            }
        }

        true
    }

    fn can_queen_attack(&self, src: &Position, dst: &Position) -> bool {
        self.can_rook_attack(src, dst) || self.can_bishop_attack(src, dst)
    }

    fn can_king_attack(&self, src: &Position, dst: &Position) -> bool {
        let col_diff = (dst.col as i32 - src.col as i32).abs();
        let row_diff = (dst.row as i32 - src.row as i32).abs();

        col_diff <= 1 && row_diff <= 1 && (col_diff + row_diff) > 0
    }

    fn is_in_check(&self, color: Color) -> bool {
        let king_pos = match color {
            Color::White => self.white_king_pos,
            Color::Black => self.black_king_pos,
        };
        self.is_square_attacked(&king_pos, color.opposite())
    }

    pub fn is_check(&self) -> bool {
        self.is_in_check(self.turn)
    }

    pub fn is_checkmate(&self) -> bool {
        self.is_check() && self.legal_moves().is_empty()
    }

    pub fn is_stalemate(&self) -> bool {
        !self.is_check() && self.legal_moves().is_empty()
    }

    pub fn is_over(&self) -> bool {
        self.is_checkmate()
            || self.is_stalemate()
            || self.halfmove_clock >= 150
            || self.is_insufficient_material()
    }

    pub fn en_passant_square(&self) -> Option<Position> {
        self.en_passant
    }

    pub fn has_legal_en_passant(&self) -> bool {
        if let Some(ep_square) = self.en_passant {
            // Check if any pawn can legally capture en passant
            // Look for pawns of the current player that can attack the en passant square
            let pawn_row = if self.turn == Color::White { 4 } else { 3 }; // 5th row (index 4) for white, 4th row (index 3) for black

            // Check squares to the left and right of the en passant square
            for col_offset in [-1i32, 1i32] {
                let pawn_col = ep_square.col as i32 + col_offset;
                if pawn_col >= 0 && pawn_col < self.board.width() as i32 {
                    let pawn_pos = Position::new(pawn_col as usize, pawn_row);
                    if let Some(piece) = self.board.get_piece(&pawn_pos) {
                        if piece.piece_type == PieceType::Pawn && piece.color == self.turn {
                            // Found a pawn that can potentially capture
                            // Create the en passant move and check if it's legal
                            let ep_move = Move::from_position(
                                pawn_pos,
                                ep_square,
                                MoveFlags::CAPTURE | MoveFlags::EN_PASSANT,
                            );

                            // Test if this move would leave our king in check
                            let mut test_game = self.clone();
                            if test_game.make_move_without_legality_checking(&ep_move)
                                && !test_game.is_in_check(self.turn)
                            {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Parse a LAN move string, with game context to set proper flags (castling, en passant, etc.)
    /// The `from_lan()` method on Move itself lacks game context.
    pub fn move_from_lan(&self, lan: &str) -> Result<Move, String> {
        let base_move = Move::from_lan(lan, self.board.width(), self.board.height())?;

        let piece = self.board.get_piece(&base_move.src);
        if piece.is_none() {
            return Err("No piece at source square".to_string());
        }
        let piece = piece.unwrap();

        let mut flags = base_move.flags;

        // Check for capture
        if self.board.get_piece(&base_move.dst).is_some() {
            flags |= MoveFlags::CAPTURE;
        }

        // Check for castling (king moving 2 squares)
        if piece.piece_type == PieceType::King {
            let col_diff = (base_move.dst.col as i32 - base_move.src.col as i32).abs();
            if col_diff == 2 {
                flags |= MoveFlags::CASTLE;
            }
        }

        // Check for en passant
        if piece.piece_type == PieceType::Pawn {
            if let Some(ep_square) = self.en_passant {
                if base_move.dst == ep_square {
                    flags |= MoveFlags::CAPTURE | MoveFlags::EN_PASSANT;
                }
            }

            // Check for double push
            let row_diff = (base_move.dst.row as i32 - base_move.src.row as i32).abs();
            if row_diff == 2 {
                flags |= MoveFlags::DOUBLE_PUSH;
            }
        }

        Ok(Move {
            src: base_move.src,
            dst: base_move.dst,
            flags,
            promotion: base_move.promotion,
        })
    }

    pub fn outcome(&self) -> Option<GameOutcome> {
        if !self.is_over() {
            return None;
        }

        if self.is_checkmate() {
            // The current player is checkmated, so the other player wins
            if self.turn == Color::White {
                Some(GameOutcome::BlackWin)
            } else {
                Some(GameOutcome::WhiteWin)
            }
        } else if self.is_stalemate() {
            Some(GameOutcome::Stalemate)
        } else if self.is_insufficient_material() {
            Some(GameOutcome::InsufficientMaterial)
        } else if self.halfmove_clock >= 100 {
            Some(GameOutcome::FiftyMoveRule)
        } else {
            // Other draw conditions (could be extended to include threefold repetition)
            Some(GameOutcome::Other)
        }
    }

    pub fn is_insufficient_material(&self) -> bool {
        // Count specific piece types for both sides
        let mut white_pawns = 0;
        let mut white_queens = 0;
        let mut white_rooks = 0;
        let mut white_bishops = 0;
        let mut white_knights = 0;

        for (_, piece) in self.board.pieces(Color::White) {
            match piece.piece_type {
                PieceType::Pawn => white_pawns += 1,
                PieceType::Queen => white_queens += 1,
                PieceType::Rook => white_rooks += 1,
                PieceType::Bishop => white_bishops += 1,
                PieceType::Knight => white_knights += 1,
                PieceType::King => {}
            }
        }

        let mut black_pawns = 0;
        let mut black_queens = 0;
        let mut black_rooks = 0;
        let mut black_bishops = 0;
        let mut black_knights = 0;

        for (_, piece) in self.board.pieces(Color::Black) {
            match piece.piece_type {
                PieceType::Pawn => black_pawns += 1,
                PieceType::Queen => black_queens += 1,
                PieceType::Rook => black_rooks += 1,
                PieceType::Bishop => black_bishops += 1,
                PieceType::Knight => black_knights += 1,
                PieceType::King => {}
            }
        }

        // If either side has pawns, queens, or rooks, there's sufficient material
        if white_pawns + white_queens + white_rooks > 0
            || black_pawns + black_queens + black_rooks > 0
        {
            return false;
        }

        // Now we only have kings, bishops, and knights
        let white_minor_pieces = white_bishops + white_knights;
        let black_minor_pieces = black_bishops + black_knights;

        // If only bishops and knights remain, check for insufficient material
        // Special case: if both sides only have bishops (no knights), and all bishops are on the same color, it's insufficient
        if white_knights == 0 && black_knights == 0 && (white_bishops > 0 || black_bishops > 0) {
            // Check if all bishops on the board are on the same color
            if self.are_all_bishops_on_same_color() {
                return true;
            }
        }

        // Check for other insufficient material cases:
        match (white_minor_pieces, black_minor_pieces) {
            // K vs K
            (0, 0) => true,
            // K vs K+B or K vs K+N
            (0, 1) => true,
            (1, 0) => true,
            // Any other combination can potentially mate
            _ => false,
        }
    }

    fn are_all_bishops_on_same_color(&self) -> bool {
        let bishops = self.board.piece_type_bb(PieceType::Bishop);
        let mut first_color: Option<usize> = None;
        let w = self.board.width();

        // Check square colors of all bishops (both white and black)
        for idx in bishops.iter_ones() {
            // A square is light if (col + row) is even
            let col = idx % w;
            let row = idx / w;
            let square_color = (col + row) % 2;
            match first_color {
                None => first_color = Some(square_color),
                Some(c) if c != square_color => return false,
                _ => {}
            }
        }

        // If there are no bishops, return false; otherwise all matched
        first_color.is_some()
    }

    pub fn to_fen(&self) -> String {
        let mut fen = self.board.to_fen();

        // Turn
        fen.push(' ');
        fen.push(if self.turn == Color::White { 'w' } else { 'b' });

        // Castling rights
        fen.push(' ');
        let has_any = self.castling_rights.white_kingside
            || self.castling_rights.white_queenside
            || self.castling_rights.black_kingside
            || self.castling_rights.black_queenside;

        if !has_any {
            fen.push('-');
        } else {
            if self.castling_rights.white_kingside {
                fen.push('K');
            }
            if self.castling_rights.white_queenside {
                fen.push('Q');
            }
            if self.castling_rights.black_kingside {
                fen.push('k');
            }
            if self.castling_rights.black_queenside {
                fen.push('q');
            }
        }

        // En passant (only include if there are legal en passant moves)
        fen.push(' ');
        if self.has_legal_en_passant() {
            if let Some(ep) = self.en_passant {
                fen.push_str(&ep.to_algebraic());
            } else {
                fen.push('-');
            }
        } else {
            fen.push('-');
        }

        // Halfmove clock
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());

        // Fullmove number
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }
}

/// Type alias for a standard 8x8 game
pub type StandardGame = Game<{ nw_for_board(STANDARD_COLS as u8, STANDARD_ROWS as u8) }>;

impl StandardGame {
    pub fn standard() -> Self {
        Self::new(
            8,
            8,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            true,
        )
        .expect("Failed to create standard game")
    }
}

impl<const NW: usize> std::fmt::Display for Game<NW> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Game(current_player: {}, is_over: {}, outcome: {:?})\n{}",
            self.turn(),
            self.is_over(),
            self.outcome(),
            self.board
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type StdGame = Game<{ nw_for_board(STANDARD_COLS as u8, STANDARD_ROWS as u8) }>;

    #[test]
    fn test_standard_game_creation() {
        let game = StdGame::standard();
        assert_eq!(game.board().width(), 8);
        assert_eq!(game.board().height(), 8);
        assert_eq!(game.turn(), Color::White);
        assert_eq!(game.fullmove_number(), 1);
        assert_eq!(game.halfmove_clock(), 0);
    }

    #[test]
    fn test_standard_game_initial_position() {
        let game = StdGame::standard();
        let fen = game.to_fen();
        assert_eq!(
            fen,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    fn test_standard_game_king_tracking() {
        let game = StdGame::standard();

        assert_eq!(game.white_king_pos, Position::new(4, 0));
        assert_eq!(game.black_king_pos, Position::new(4, 7));
    }

    #[test]
    fn test_standard_game_rook_attack_patterns() {
        let mut game = StdGame::standard();
        game.board.clear();

        let rook = Piece::new(PieceType::Rook, Color::White);
        let rook_pos = Position::new(4, 4); // e5
        game.board.set_piece(&rook_pos, Some(rook));

        // Rook can attack along rows and cols
        assert!(game.can_rook_attack(&rook_pos, &Position::new(4, 0))); // e1
        assert!(game.can_rook_attack(&rook_pos, &Position::new(4, 7))); // e8
        assert!(game.can_rook_attack(&rook_pos, &Position::new(0, 4))); // a5
        assert!(game.can_rook_attack(&rook_pos, &Position::new(7, 4))); // h5

        // Cannot attack diagonally
        assert!(!game.can_rook_attack(&rook_pos, &Position::new(5, 5))); // f6

        // Test blocked path
        let blocker = Piece::new(PieceType::Pawn, Color::Black);
        game.board.set_piece(&Position::new(4, 6), Some(blocker)); // e7

        assert!(!game.can_rook_attack(&rook_pos, &Position::new(4, 7))); // e8 (blocked)
        assert!(game.can_rook_attack(&rook_pos, &Position::new(4, 6))); // e7 (can capture)
    }

    #[test]
    fn test_standard_game_bishop_attack_patterns() {
        let mut game = StdGame::standard();
        game.board.clear();

        let bishop = Piece::new(PieceType::Bishop, Color::White);
        let bishop_pos = Position::new(4, 4); // e5
        game.board.set_piece(&bishop_pos, Some(bishop));

        // Bishop can attack diagonally
        assert!(game.can_bishop_attack(&bishop_pos, &Position::new(0, 0))); // a1
        assert!(game.can_bishop_attack(&bishop_pos, &Position::new(7, 7))); // h8
        assert!(game.can_bishop_attack(&bishop_pos, &Position::new(1, 7))); // b8
        assert!(game.can_bishop_attack(&bishop_pos, &Position::new(7, 1))); // h2

        // Cannot attack along rows/cols
        assert!(!game.can_bishop_attack(&bishop_pos, &Position::new(4, 0))); // e1

        // Test blocked path
        let blocker = Piece::new(PieceType::Pawn, Color::Black);
        game.board.set_piece(&Position::new(6, 6), Some(blocker)); // g7

        assert!(!game.can_bishop_attack(&bishop_pos, &Position::new(7, 7))); // h8 (blocked)
        assert!(game.can_bishop_attack(&bishop_pos, &Position::new(6, 6))); // g7 (can capture)
    }

    #[test]
    fn test_standard_game_fen_parsing_valid_en_passant() {
        // Test with a valid en passant scenario
        let valid_ep_fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
        let game = StdGame::new(8, 8, valid_ep_fen, true).expect("Failed to parse FEN");

        assert_eq!(game.to_fen(), valid_ep_fen);
        assert_eq!(game.turn(), Color::White);
        assert_eq!(game.fullmove_number(), 3);
        assert_eq!(game.halfmove_clock(), 0);
    }

    #[test]
    fn test_standard_game_fen_parsing_invalid_en_passant() {
        let invalid_ep_fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let game = StdGame::new(8, 8, invalid_ep_fen, true).expect("Failed to parse FEN");

        // Note: en passant square e3 is ignored because there's no enemy pawn that can capture
        assert_eq!(
            game.to_fen(),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1"
        );
        assert_eq!(game.turn(), Color::Black);
        assert_eq!(game.fullmove_number(), 1);
        assert_eq!(game.halfmove_clock(), 0);
    }

    #[test]
    fn test_standard_game_move_making_basic() {
        let mut game = StdGame::standard();

        // Make a simple pawn move
        let e4_move =
            Move::from_position(Position::new(4, 1), Position::new(4, 3), MoveFlags::empty());
        let success = game.make_move(&e4_move);
        assert!(success, "Move should be successful");

        // Verify the move was made
        assert_eq!(game.board.get_piece(&Position::new(4, 1)), None);
        assert_eq!(
            game.board
                .get_piece(&Position::new(4, 3))
                .expect("Expected piece at position e4 after move")
                .piece_type,
            PieceType::Pawn
        );
    }

    #[test]
    fn test_custom_game_board_sizes() {
        // Create custom FENs for different board sizes
        let fen_6x6 = "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1";

        let game: Game<{ nw_for_board(6, 6) }> =
            Game::new(6, 6, fen_6x6, true).expect("Failed to create 6x6 game");
        assert_eq!(game.board().width(), 6);
        assert_eq!(game.board().height(), 6);

        assert_eq!(
            game.board()
                .get_piece(&game.white_king_pos)
                .unwrap()
                .piece_type,
            PieceType::King
        );
        assert_eq!(
            game.board()
                .get_piece(&game.black_king_pos)
                .unwrap()
                .piece_type,
            PieceType::King
        );

        // Check that tracked positions match actual king positions
        // Should be able to generate FEN
        let fen = game.to_fen();
        assert!(!fen.is_empty());
    }

    #[test]
    fn test_custom_game_piece_placement() {
        let fen_6x6 = "rnbkqr/pppppp/6/6/PPPPPP/RNBKQR w - - 0 1";
        let game: Game<{ nw_for_board(6, 6) }> =
            Game::new(6, 6, fen_6x6, true).expect("Failed to create 6x6 game");
        let white_pieces = game.board().pieces(Color::White);
        let black_pieces = game.board().pieces(Color::Black);

        // Should have pieces placed
        assert!(!white_pieces.is_empty());
        assert!(!black_pieces.is_empty());

        // Should have exactly one king each
        let white_kings: Vec<_> = white_pieces
            .iter()
            .filter(|(_, piece)| piece.piece_type == PieceType::King)
            .collect();
        let black_kings: Vec<_> = black_pieces
            .iter()
            .filter(|(_, piece)| piece.piece_type == PieceType::King)
            .collect();

        assert_eq!(white_kings.len(), 1);
        assert_eq!(black_kings.len(), 1);
    }

    #[test]
    fn test_standard_game_is_square_attacked_basic() {
        let mut game = StdGame::standard();
        game.board.clear();

        // Place a white rook at e5
        let rook = Piece::new(PieceType::Rook, Color::White);
        game.board.set_piece(&Position::new(4, 4), Some(rook)); // e5

        // The rook should attack squares along its row and col
        assert!(game.is_square_attacked(&Position::new(4, 0), Color::White)); // e1
        assert!(game.is_square_attacked(&Position::new(0, 4), Color::White)); // a5
        assert!(!game.is_square_attacked(&Position::new(5, 5), Color::White)); // f6 (diagonal)

        // Place a black king at e8
        let king = Piece::new(PieceType::King, Color::Black);
        game.board.set_piece(&Position::new(4, 7), Some(king)); // e8

        // The king should be attacked by the rook
        assert!(game.is_square_attacked(&Position::new(4, 7), Color::White));
    }

    #[test]
    fn test_standard_game_outcome_checkmate_white_wins() {
        // Scholar's mate - white wins
        // 1. e4 e5, 2. Bc4 Nc6, 3. Qh5 Nf6??, 4. Qxf7#
        let mut game = StdGame::standard();

        game.make_move(&Move::from_lan("e2e4", 8, 8).unwrap());
        game.make_move(&Move::from_lan("e7e5", 8, 8).unwrap());
        game.make_move(&Move::from_lan("f1c4", 8, 8).unwrap());
        game.make_move(&Move::from_lan("b8c6", 8, 8).unwrap());
        game.make_move(&Move::from_lan("d1h5", 8, 8).unwrap());
        game.make_move(&Move::from_lan("g8f6", 8, 8).unwrap());
        game.make_move(&Move::from_lan("h5f7", 8, 8).unwrap());

        assert!(game.is_checkmate());
        let outcome = game.outcome();
        assert_eq!(outcome, Some(GameOutcome::WhiteWin));
    }

    #[test]
    fn test_standard_game_outcome_checkmate_black_wins() {
        // Fool's mate - black wins
        let mut game = StdGame::standard();

        game.make_move(&Move::from_lan("f2f3", 8, 8).unwrap());
        game.make_move(&Move::from_lan("e7e5", 8, 8).unwrap());
        game.make_move(&Move::from_lan("g2g4", 8, 8).unwrap());
        game.make_move(&Move::from_lan("d8h4", 8, 8).unwrap());

        assert!(game.is_checkmate());
        let outcome = game.outcome();
        assert_eq!(outcome, Some(GameOutcome::BlackWin));
    }

    #[test]
    fn test_standard_game_outcome_stalemate() {
        // White king on a8, black queen on b6, black king on c7
        let fen = "K7/8/1q6/8/8/8/8/2k5 w - - 0 1";
        let game = StdGame::new(8, 8, fen, false).expect("Failed to parse stalemate FEN");

        assert!(
            !game.is_check(),
            "King should not be in check for stalemate"
        );
        let moves = game.legal_moves();
        assert!(
            moves.is_empty(),
            "King should have no legal moves for stalemate"
        );
        assert!(game.is_stalemate());
        let outcome = game.outcome();
        assert_eq!(outcome, Some(GameOutcome::Stalemate));
    }

    #[test]
    fn test_standard_game_outcome_insufficient_material() {
        let mut game = StdGame::standard();
        game.board.clear();

        // King vs King - insufficient material
        game.board.set_piece(
            &Position::new(4, 0),
            Some(Piece::new(PieceType::King, Color::White)),
        );
        game.board.set_piece(
            &Position::new(4, 7),
            Some(Piece::new(PieceType::King, Color::Black)),
        );

        assert!(game.is_insufficient_material());
        assert!(game.is_over());
        let outcome = game.outcome();
        assert_eq!(outcome, Some(GameOutcome::InsufficientMaterial));
    }

    #[test]
    fn test_standard_game_outcome_insufficient_material_bishop() {
        let mut game = StdGame::standard();
        game.board.clear();

        // King + Bishop vs King - insufficient material
        game.board.set_piece(
            &Position::new(4, 0),
            Some(Piece::new(PieceType::King, Color::White)),
        );
        game.board.set_piece(
            &Position::new(2, 2),
            Some(Piece::new(PieceType::Bishop, Color::White)),
        );
        game.board.set_piece(
            &Position::new(4, 7),
            Some(Piece::new(PieceType::King, Color::Black)),
        );

        assert!(game.is_insufficient_material());
        assert!(game.is_over());
        let outcome = game.outcome();
        assert_eq!(outcome, Some(GameOutcome::InsufficientMaterial));
    }

    #[test]
    fn test_standard_game_outcome_fifty_move_rule() {
        let mut game = StdGame::standard();
        game.board.clear();

        // Set up a simple position with just kings and a rook
        game.board.set_piece(
            &Position::new(4, 0),
            Some(Piece::new(PieceType::King, Color::White)),
        );
        game.board.set_piece(
            &Position::new(0, 0),
            Some(Piece::new(PieceType::Rook, Color::White)),
        );
        game.board.set_piece(
            &Position::new(4, 7),
            Some(Piece::new(PieceType::King, Color::Black)),
        );

        // Manually set halfmove clock to trigger fifty-move rule (150 half-moves = 75 full moves)
        game.halfmove_clock = 150;

        assert!(game.is_over());
        let outcome = game.outcome();
        assert_eq!(outcome, Some(GameOutcome::FiftyMoveRule));
    }

    #[test]
    fn test_standard_game_halfmove_clock_reset_on_pawn_move() {
        let mut game = StdGame::standard();

        // Make some non-pawn moves to increase halfmove clock
        game.make_move(&Move::from_lan("g1f3", 8, 8).unwrap());
        game.make_move(&Move::from_lan("g8f6", 8, 8).unwrap());
        game.make_move(&Move::from_lan("f3g1", 8, 8).unwrap());
        game.make_move(&Move::from_lan("f6g8", 8, 8).unwrap());

        assert_eq!(game.halfmove_clock, 4);

        // Make a pawn move - should reset halfmove clock
        game.make_move(&Move::from_lan("e2e4", 8, 8).unwrap());
        assert_eq!(game.halfmove_clock, 0);
    }

    #[test]
    fn test_standard_game_halfmove_clock_reset_on_capture() {
        let mut game = StdGame::standard();

        // Set up a position where a capture is possible
        game.make_move(&Move::from_lan("e2e4", 8, 8).unwrap());
        game.make_move(&Move::from_lan("d7d5", 8, 8).unwrap());

        assert_eq!(game.halfmove_clock, 0); // Both were pawn moves

        // Make some knight moves to increase halfmove clock
        game.make_move(&Move::from_lan("g1f3", 8, 8).unwrap());
        game.make_move(&Move::from_lan("b8c6", 8, 8).unwrap());
        assert_eq!(game.halfmove_clock, 2);

        // Make a capture - should reset halfmove clock
        game.make_move(&Move::from_lan("e4d5", 8, 8).unwrap()); // Capture pawn
        assert_eq!(game.halfmove_clock, 0);
    }

    #[test]
    fn test_standard_game_castling_rights_methods() {
        let mut game = StdGame::standard();

        // Initial position should have all castling rights
        assert!(game.castling_rights().has_kingside(Color::White));
        assert!(game.castling_rights().has_queenside(Color::White));
        assert!(game.castling_rights().has_kingside(Color::Black));
        assert!(game.castling_rights().has_queenside(Color::Black));

        // Move white king
        game.make_move(&Move::from_lan("e2e3", 8, 8).unwrap());
        game.make_move(&Move::from_lan("e7e6", 8, 8).unwrap());
        game.make_move(&Move::from_lan("e1e2", 8, 8).unwrap());

        // White should lose all castling rights
        assert!(!game.castling_rights().has_kingside(Color::White));
        assert!(!game.castling_rights().has_queenside(Color::White));
        // Black should still have both
        assert!(game.castling_rights().has_kingside(Color::Black));
        assert!(game.castling_rights().has_queenside(Color::Black));
    }

    #[test]
    fn test_standard_game_castling_rights_rook_move() {
        let mut game = StdGame::standard();

        // Clear path for rook movement
        game.board_clear();
        game.board.set_piece(
            &Position::new(4, 0),
            Some(Piece::new(PieceType::King, Color::White)),
        );
        game.board.set_piece(
            &Position::new(4, 7),
            Some(Piece::new(PieceType::King, Color::Black)),
        );
        game.board.set_piece(
            &Position::new(0, 0),
            Some(Piece::new(PieceType::Rook, Color::White)),
        );
        game.board.set_piece(
            &Position::new(7, 0),
            Some(Piece::new(PieceType::Rook, Color::White)),
        );

        // Move queenside rook
        game.make_move(&Move::from_lan("a1a2", 8, 8).unwrap());

        // Should lose queenside castling only
        assert!(game.castling_rights().has_kingside(Color::White));
        assert!(!game.castling_rights().has_queenside(Color::White));
    }
}
