use crate::bitboard::Bitboard;
use crate::color::Color;
use crate::r#move::{Move, MoveFlags};
use crate::outcome::MoveList;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;

use super::Game;

#[hotpath::measure_all]
impl<const W: usize, const H: usize> Game<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    pub fn is_legal_move(&mut self, mv: &Move) -> bool {
        let piece = match self.board.get_piece(&mv.src) {
            Some(p) if p.color == self.turn => p,
            _ => return false,
        };

        // Generate pseudo-legal moves only for the source piece
        let mut pseudo_legal = MoveList::new();
        self.generate_pseudo_legal_moves_for_piece_into(&mv.src, &piece, &mut pseudo_legal);

        // Find the matching pseudo-legal move (which has correct flags/promotion)
        // then check only that one for legality
        if let Some(m) = pseudo_legal
            .iter()
            .find(|m| m.src == mv.src && m.dst == mv.dst)
        {
            self.is_pseudo_legal_move_legal(m, &piece)
        } else {
            false
        }
    }

    /// Test whether a pseudo-legal move is actually legal (doesn't leave own king in check).
    /// Temporarily makes the move on the board, checks, then unmakes.
    pub(super) fn is_pseudo_legal_move_legal(&mut self, mv: &Move, piece: &Piece) -> bool {
        debug_assert!(
            self.board.get_piece(&self.white_king_pos)
                == Some(Piece::new(PieceType::King, Color::White)),
            "white_king_pos {:?} doesn't contain a white king",
            self.white_king_pos,
        );
        debug_assert!(
            self.board.get_piece(&self.black_king_pos)
                == Some(Piece::new(PieceType::King, Color::Black)),
            "black_king_pos {:?} doesn't contain a black king",
            self.black_king_pos,
        );
        let opponent = piece.color.opposite();

        let captured =
            if mv.flags.contains(MoveFlags::CAPTURE) && !mv.flags.contains(MoveFlags::EN_PASSANT) {
                let dst_idx = mv.dst.to_index(W);
                debug_assert!(
                    self.board.piece_type_at(dst_idx).is_some(),
                    "CAPTURE flag set but no piece at destination index {}",
                    dst_idx,
                );
                let pt = self.board.piece_type_at(dst_idx).expect(
                    "is_pseudo_legal_move_legal: piece type must exist at capture destination",
                );
                Some(Piece::new(pt, opponent))
            } else {
                None
            };

        // Handle castling rook: must move rook before placing king so pieces
        // don't overlap on the same square (which would corrupt bitboards).
        let castle_rook = if mv.flags.contains(MoveFlags::CASTLE) {
            let rook = Piece::new(PieceType::Rook, piece.color);
            let (rook_from, rook_to) = mv.castling_rook_positions(W);
            debug_assert!(
                self.board.get_piece(&rook_from) == Some(rook),
                "castling legality: expected rook at ({}, {}), found {:?}",
                rook_from.col,
                rook_from.row,
                self.board.get_piece(&rook_from),
            );
            self.board.remove_piece(&rook_from, &rook);
            self.board.place_piece(&rook_to, &rook);
            Some((rook_from, rook_to, rook))
        } else {
            None
        };

        // Make the move on the board
        self.board.remove_piece(&mv.src, piece);
        if let Some(ref cap) = captured {
            self.board.remove_piece(&mv.dst, cap);
        }
        let placed_piece = if mv.flags.contains(MoveFlags::PROMOTION) {
            Piece::new(
                mv.promotion.unwrap_or(PieceType::DEFAULT_PROMOTION),
                piece.color,
            )
        } else {
            *piece
        };
        self.board.place_piece(&mv.dst, &placed_piece);

        // Update king position if a king moved
        let old_king_pos = if piece.piece_type == PieceType::King {
            let old = match piece.color {
                Color::White => self.white_king_pos,
                Color::Black => self.black_king_pos,
            };
            match piece.color {
                Color::White => self.white_king_pos = mv.dst,
                Color::Black => self.black_king_pos = mv.dst,
            }
            Some(old)
        } else {
            None
        };

        // Handle en passant capture
        let ep_captured = if mv.flags.contains(MoveFlags::EN_PASSANT) {
            let ep_pos = Position::new(mv.dst.col, mv.src.row);
            let ep_piece = Piece::new(PieceType::Pawn, opponent);
            debug_assert!(
                self.board.get_piece(&ep_pos) == Some(ep_piece),
                "en passant legality: expected opponent pawn at ({}, {}), found {:?}",
                ep_pos.col,
                ep_pos.row,
                self.board.get_piece(&ep_pos),
            );
            self.board.remove_piece(&ep_pos, &ep_piece);
            Some((ep_pos, ep_piece))
        } else {
            None
        };

        let in_check = self.is_in_check(piece.color);

        // Unmake: restore board state
        if let Some((ep_pos, ep_piece)) = ep_captured {
            self.board.place_piece(&ep_pos, &ep_piece);
        }
        if let Some(old) = old_king_pos {
            match piece.color {
                Color::White => self.white_king_pos = old,
                Color::Black => self.black_king_pos = old,
            }
        }
        self.board.remove_piece(&mv.dst, &placed_piece);
        if let Some(ref cap) = captured {
            self.board.place_piece(&mv.dst, cap);
        }
        self.board.place_piece(&mv.src, piece);

        // Restore castling rook
        if let Some((rook_from, rook_to, rook)) = castle_rook {
            self.board.remove_piece(&rook_to, &rook);
            self.board.place_piece(&rook_from, &rook);
        }

        !in_check
    }

    pub fn legal_moves(&mut self) -> MoveList {
        let mut moves = MoveList::new();
        self.for_each_legal_move(|mv| {
            moves.push(mv);
            false
        });
        moves
    }

    /// Iterates over all legal moves, invoking `f` for each.
    /// `f` returns `true` to stop iteration (short-circuit), `false` to continue.
    /// Returns `true` if short-circuited, `false` otherwise.
    pub(super) fn for_each_legal_move(&mut self, mut f: impl FnMut(Move) -> bool) -> bool {
        let info = self.compute_check_pin_info();
        let color = self.turn;
        let opponent = color.opposite();
        let king_pos = match color {
            Color::White => self.white_king_pos,
            Color::Black => self.black_king_pos,
        };
        let king_idx = king_pos.to_index(W);
        let own = self.board.color_bb(color);
        let occupied = self.board.occupied();
        let occupied_no_king = occupied.andnot(Bitboard::single(king_idx));
        let geo = Self::geo();

        // -----------------------------------------------------------------
        // King normal moves
        // -----------------------------------------------------------------
        let targets = geo
            .king_attacks(king_idx)
            .andnot(own)
            .andnot(info.king_danger_squares);
        for dst_idx in targets.iter_ones() {
            let is_capture = occupied.get(dst_idx);
            if is_capture {
                let occupied_after = occupied_no_king.andnot(Bitboard::single(dst_idx));
                if self.is_square_attacked_on(dst_idx, opponent, occupied_after) {
                    continue;
                }
            }
            let dst = Position::from_index(dst_idx, W);
            let flags = if is_capture {
                MoveFlags::CAPTURE
            } else {
                MoveFlags::empty()
            };
            if f(Move::from_position(king_pos, dst, flags)) {
                return true;
            }
        }

        // -----------------------------------------------------------------
        // King castling
        // -----------------------------------------------------------------
        if self.castling_enabled && W >= 5 && info.num_checkers == 0 {
            let row = usize::from(king_pos.row);
            if self.castling_rights.has_kingside(color) {
                let king_dst_col = usize::from(king_pos.col) + 2;
                let rook_col = W - 1;
                if king_dst_col < rook_col
                    && let Some(mv) = self.try_castle_legal(
                        &king_pos,
                        row,
                        king_dst_col,
                        rook_col,
                        opponent,
                        occupied_no_king,
                    )
                    && f(mv)
                {
                    return true;
                }
            }
            if self.castling_rights.has_queenside(color) && king_pos.col >= 2 {
                let king_dst_col = usize::from(king_pos.col) - 2;
                if let Some(mv) = self.try_castle_legal(
                    &king_pos,
                    row,
                    king_dst_col,
                    0,
                    opponent,
                    occupied_no_king,
                ) && f(mv)
                {
                    return true;
                }
            }
        }

        // -----------------------------------------------------------------
        // Double check: only king moves are legal
        // -----------------------------------------------------------------
        if info.num_checkers >= 2 {
            return false;
        }

        let enemy = self.board.color_bb(opponent);

        for idx in self.board.color_bb(color).iter_ones() {
            if idx == king_idx {
                continue;
            }

            let pt = self
                .board
                .piece_type_at(idx)
                .expect("for_each_legal_move: piece type must exist");

            if pt == PieceType::Knight && info.pinned.get(idx) {
                continue;
            }

            let move_mask = if info.pinned.get(idx) {
                info.get_pin_mask(idx) & info.check_mask
            } else {
                info.check_mask
            };

            if move_mask.is_empty() {
                continue;
            }

            let pos = Position::from_index(idx, W);

            match pt {
                PieceType::Knight => {
                    let targets = geo.knight_attacks(idx).andnot(own) & move_mask;
                    for dst_idx in targets.iter_ones() {
                        let dst = Position::from_index(dst_idx, W);
                        let flags = if occupied.get(dst_idx) {
                            MoveFlags::CAPTURE
                        } else {
                            MoveFlags::empty()
                        };
                        if f(Move::from_position(pos, dst, flags)) {
                            return true;
                        }
                    }
                }
                PieceType::Pawn => {
                    let piece = Piece::new(PieceType::Pawn, color);
                    let is_white = color == Color::White;
                    let src_bb = Bitboard::single(idx);
                    let start_row = if is_white { 1 } else { H - 2 };
                    let promo_row = if is_white { H - 2 } else { 1 };

                    // Single push
                    let push = geo.pawn_push(src_bb, is_white).andnot(occupied);
                    let legal_push = push & move_mask;
                    for pidx in legal_push.iter_ones() {
                        let dst = Position::from_index(pidx, W);
                        if usize::from(pos.row) == promo_row {
                            for promo_pt in &PieceType::PROMOTABLE {
                                if f(Move::from_position_with_promotion(
                                    pos,
                                    dst,
                                    MoveFlags::PROMOTION,
                                    *promo_pt,
                                )) {
                                    return true;
                                }
                            }
                        } else if f(Move::from_position(pos, dst, MoveFlags::empty())) {
                            return true;
                        }
                    }

                    // Double push
                    if usize::from(pos.row) == start_row && !push.is_empty() {
                        let double = geo.pawn_push(push, is_white).andnot(occupied) & move_mask;
                        for pidx in double.iter_ones() {
                            let dst = Position::from_index(pidx, W);
                            if f(Move::from_position(pos, dst, MoveFlags::DOUBLE_PUSH)) {
                                return true;
                            }
                        }
                    }

                    // Captures
                    let attacks = geo.pawn_attacks(idx, is_white);
                    let captures = attacks & enemy & move_mask;
                    for cidx in captures.iter_ones() {
                        let dst = Position::from_index(cidx, W);
                        if usize::from(pos.row) == promo_row {
                            for promo_pt in &PieceType::PROMOTABLE {
                                if f(Move::from_position_with_promotion(
                                    pos,
                                    dst,
                                    MoveFlags::CAPTURE | MoveFlags::PROMOTION,
                                    *promo_pt,
                                )) {
                                    return true;
                                }
                            }
                        } else if f(Move::from_position(pos, dst, MoveFlags::CAPTURE)) {
                            return true;
                        }
                    }

                    // En passant (make/unmake fallback for discovered check)
                    if let Some(ep) = self.en_passant {
                        let ep_bb = Bitboard::single(ep.to_index(W));
                        if !(attacks & ep_bb).is_empty() {
                            let ep_move = Move::from_position(
                                pos,
                                ep,
                                MoveFlags::CAPTURE | MoveFlags::EN_PASSANT,
                            );
                            if self.is_pseudo_legal_move_legal(&ep_move, &piece) && f(ep_move) {
                                return true;
                            }
                        }
                    }
                }
                PieceType::Bishop => {
                    let attacks = geo.diagonal_attacks(idx, occupied);
                    let targets = attacks.andnot(own) & move_mask;
                    for dst_idx in targets.iter_ones() {
                        let dst = Position::from_index(dst_idx, W);
                        let flags = if occupied.get(dst_idx) {
                            MoveFlags::CAPTURE
                        } else {
                            MoveFlags::empty()
                        };
                        if f(Move::from_position(pos, dst, flags)) {
                            return true;
                        }
                    }
                }
                PieceType::Rook => {
                    let attacks = geo.orthogonal_attacks(idx, occupied);
                    let targets = attacks.andnot(own) & move_mask;
                    for dst_idx in targets.iter_ones() {
                        let dst = Position::from_index(dst_idx, W);
                        let flags = if occupied.get(dst_idx) {
                            MoveFlags::CAPTURE
                        } else {
                            MoveFlags::empty()
                        };
                        if f(Move::from_position(pos, dst, flags)) {
                            return true;
                        }
                    }
                }
                PieceType::Queen => {
                    let attacks =
                        geo.orthogonal_attacks(idx, occupied) | geo.diagonal_attacks(idx, occupied);
                    let targets = attacks.andnot(own) & move_mask;
                    for dst_idx in targets.iter_ones() {
                        let dst = Position::from_index(dst_idx, W);
                        let flags = if occupied.get(dst_idx) {
                            MoveFlags::CAPTURE
                        } else {
                            MoveFlags::empty()
                        };
                        if f(Move::from_position(pos, dst, flags)) {
                            return true;
                        }
                    }
                }
                PieceType::King => unreachable!(),
            }
        }

        false
    }

    pub fn pseudo_legal_moves(&self) -> MoveList {
        let mut moves = MoveList::new();

        for (pos, piece) in self.board.pieces_iter(self.turn) {
            self.generate_pseudo_legal_moves_for_piece_into(&pos, &piece, &mut moves);
        }

        moves
    }

    pub fn legal_moves_for_position(&mut self, src: &Position) -> MoveList {
        let mut moves = MoveList::new();
        let mut pseudo_legal = MoveList::new();

        if let Some(piece) = self.board.get_piece(src) {
            if piece.color != self.turn {
                return moves;
            }

            self.generate_pseudo_legal_moves_for_piece_into(src, &piece, &mut pseudo_legal);

            for mv in pseudo_legal.iter() {
                if self.is_pseudo_legal_move_legal(mv, &piece) {
                    moves.push(*mv);
                }
            }

            pseudo_legal.clear();
        }

        moves
    }

    pub(super) fn generate_pseudo_legal_moves_for_piece_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut MoveList,
    ) {
        match piece.piece_type {
            PieceType::Pawn => self.generate_pseudo_legal_pawn_moves_into(src, piece, moves),
            PieceType::Knight => self.generate_pseudo_legal_knight_moves_into(src, piece, moves),
            PieceType::Bishop => self.generate_pseudo_legal_bishop_moves_into(src, piece, moves),
            PieceType::Rook => self.generate_pseudo_legal_rook_moves_into(src, piece, moves),
            PieceType::Queen => self.generate_pseudo_legal_queen_moves_into(src, piece, moves),
            PieceType::King => self.generate_pseudo_legal_king_moves_into(src, piece, moves),
        }
    }

    fn generate_pseudo_legal_pawn_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut MoveList,
    ) {
        let occupied = self.board.occupied();
        let enemy = self.board.color_bb(piece.color.opposite());
        let is_white = piece.color == Color::White;
        let geo = Self::geo();

        let start_row = if is_white { 1 } else { H - 2 };
        let promo_row = if is_white { H - 2 } else { 1 };

        let src_idx = src.to_index(W);
        let src_bb = Bitboard::single(src_idx);

        // Single push: forward one square, blocked by any piece
        let push = geo.pawn_push(src_bb, is_white).andnot(occupied);
        for idx in push.iter_ones() {
            let dst = Position::from_index(idx, W);
            if usize::from(src.row) == promo_row {
                for pt in &PieceType::PROMOTABLE {
                    moves.push(Move::from_position_with_promotion(
                        *src,
                        dst,
                        MoveFlags::PROMOTION,
                        *pt,
                    ));
                }
            } else {
                moves.push(Move::from_position(*src, dst, MoveFlags::empty()));
            }
        }

        // Double push: forward two squares from start row, both squares must be empty
        if usize::from(src.row) == start_row && !push.is_empty() {
            let double = geo.pawn_push(push, is_white).andnot(occupied);
            for idx in double.iter_ones() {
                let dst = Position::from_index(idx, W);
                moves.push(Move::from_position(*src, dst, MoveFlags::DOUBLE_PUSH));
            }
        }

        // Captures: diagonal attacks into enemy pieces
        let attacks = geo.pawn_attacks(src_idx, is_white);
        let captures = attacks & enemy;
        for idx in captures.iter_ones() {
            let dst = Position::from_index(idx, W);
            if usize::from(src.row) == promo_row {
                for pt in &PieceType::PROMOTABLE {
                    moves.push(Move::from_position_with_promotion(
                        *src,
                        dst,
                        MoveFlags::CAPTURE | MoveFlags::PROMOTION,
                        *pt,
                    ));
                }
            } else {
                moves.push(Move::from_position(*src, dst, MoveFlags::CAPTURE));
            }
        }

        // En passant
        if let Some(ep) = self.en_passant {
            debug_assert!(
                ep.row == 2 || usize::from(ep.row) == H - 3,
                "en passant square ({}, {}) is not on a valid row (expected row 2 or {})",
                ep.col,
                ep.row,
                H - 3,
            );
            let ep_bb = Bitboard::single(ep.to_index(W));
            if !(attacks & ep_bb).is_empty() {
                moves.push(Move::from_position(
                    *src,
                    ep,
                    MoveFlags::CAPTURE | MoveFlags::EN_PASSANT,
                ));
            }
        }
    }

    fn generate_pseudo_legal_knight_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut MoveList,
    ) {
        let own_color = self.board.color_bb(piece.color);
        let occupied = self.board.occupied();
        let src_idx = src.to_index(W);
        let attacks = Self::geo().knight_attacks(src_idx).andnot(own_color);

        for idx in attacks.iter_ones() {
            let to = Position::from_index(idx, W);
            let flags = if occupied.get(idx) {
                MoveFlags::CAPTURE
            } else {
                MoveFlags::empty()
            };
            moves.push(Move::from_position(*src, to, flags));
        }
    }

    fn generate_sliding_moves_from_attacks_into(
        &self,
        src: &Position,
        piece: &Piece,
        attacks: Bitboard<{ (W * H).div_ceil(64) }>,
        moves: &mut MoveList,
    ) {
        let occupied = self.board.occupied();
        let own_color = self.board.color_bb(piece.color);
        let targets = attacks.andnot(own_color);

        for idx in targets.iter_ones() {
            let dst = Position::from_index(idx, W);
            let flags = if occupied.get(idx) {
                MoveFlags::CAPTURE
            } else {
                MoveFlags::empty()
            };
            moves.push(Move::from_position(*src, dst, flags));
        }
    }

    fn generate_pseudo_legal_bishop_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut MoveList,
    ) {
        let geo = Self::geo();
        let occupied = self.board.occupied();
        let attacks = geo.diagonal_attacks(src.to_index(W), occupied);
        self.generate_sliding_moves_from_attacks_into(src, piece, attacks, moves)
    }

    fn generate_pseudo_legal_rook_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut MoveList,
    ) {
        let geo = Self::geo();
        let occupied = self.board.occupied();
        let attacks = geo.orthogonal_attacks(src.to_index(W), occupied);
        self.generate_sliding_moves_from_attacks_into(src, piece, attacks, moves)
    }

    fn generate_pseudo_legal_queen_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut MoveList,
    ) {
        let geo = Self::geo();
        let occupied = self.board.occupied();
        let attacks = geo.orthogonal_attacks(src.to_index(W), occupied)
            | geo.diagonal_attacks(src.to_index(W), occupied);
        self.generate_sliding_moves_from_attacks_into(src, piece, attacks, moves)
    }

    fn generate_pseudo_legal_king_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut MoveList,
    ) {
        debug_assert!(
            *src == match piece.color {
                Color::White => self.white_king_pos,
                Color::Black => self.black_king_pos,
            },
            "king move generation src {:?} doesn't match tracked king pos",
            src,
        );
        let own_color = self.board.color_bb(piece.color);
        let occupied = self.board.occupied();

        // Regular moves
        let src_idx = src.to_index(W);
        let attacks = Self::geo().king_attacks(src_idx).andnot(own_color);

        for idx in attacks.iter_ones() {
            let to = Position::from_index(idx, W);
            let flags = if occupied.get(idx) {
                MoveFlags::CAPTURE
            } else {
                MoveFlags::empty()
            };
            moves.push(Move::from_position(*src, to, flags));
        }

        // Castling
        if self.castling_enabled && W >= 5 && !self.is_in_check(piece.color) {
            let row = usize::from(src.row);
            let opponent = piece.color.opposite();

            // Kingside: king to col king+2, rook from last col to king+1
            if self.castling_rights.has_kingside(piece.color) {
                let king_dst = usize::from(src.col) + 2;
                let rook_col = W - 1;
                if king_dst < rook_col {
                    self.try_generate_castle(src, row, king_dst, rook_col, opponent, moves);
                }
            }

            // Queenside: king to col king-2, rook from col 0 to king-1
            if self.castling_rights.has_queenside(piece.color) && src.col >= 2 {
                let king_dst = usize::from(src.col) - 2;
                self.try_generate_castle(src, row, king_dst, 0, opponent, moves);
            }
        }
    }

    /// Try to generate a castling move. `king_dst` is the column the king lands on,
    /// `rook_col` is where the rook currently sits. All squares between king and rook
    /// must be empty, and all squares the king passes through must not be attacked.
    fn try_generate_castle(
        &self,
        king_src: &Position,
        row: usize,
        king_dst: usize,
        rook_col: usize,
        opponent: Color,
        moves: &mut MoveList,
    ) {
        let occupied = self.board.occupied();

        // All squares between king and rook must be empty
        let king_src_col = usize::from(king_src.col);
        let (lo, hi) = if king_src_col < rook_col {
            (king_src_col + 1, rook_col)
        } else {
            (rook_col + 1, king_src_col)
        };
        for col in lo..hi {
            if occupied.get(row * W + col) {
                return;
            }
        }

        let occupied_no_king = occupied.andnot(Bitboard::single(row * W + king_src_col));
        if self
            .try_castle_legal(
                king_src,
                row,
                king_dst,
                rook_col,
                opponent,
                occupied_no_king,
            )
            .is_none()
        {
            return;
        }

        moves.push(Move::from_position(
            *king_src,
            Position::from_usize(king_dst, row),
            MoveFlags::CASTLE,
        ));
    }

    /// Check castling legality against the current board using an occupancy
    /// where the king has already been removed from its source square.
    pub(super) fn try_castle_legal(
        &self,
        king_src: &Position,
        row: usize,
        king_dst: usize,
        rook_col: usize,
        opponent: Color,
        occupied_no_king: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Option<Move> {
        // All squares between king and rook must be empty
        let king_src_col = usize::from(king_src.col);
        let (lo, hi) = if king_src_col < rook_col {
            (king_src_col + 1, rook_col)
        } else {
            (rook_col + 1, king_src_col)
        };
        for col in lo..hi {
            if occupied_no_king.get(row * W + col) {
                return None;
            }
        }

        // All squares the king passes through (and lands on) must not be attacked
        let (path_lo, path_hi) = if king_dst > king_src_col {
            (king_src_col + 1, king_dst + 1)
        } else {
            (king_dst, king_src_col)
        };
        for col in path_lo..path_hi {
            let square_idx = row * W + col;
            let occupied_at_square = occupied_no_king.andnot(Bitboard::single(square_idx));
            if self.is_square_attacked_on(square_idx, opponent, occupied_at_square) {
                return None;
            }
        }

        Some(Move::from_position(
            *king_src,
            Position::from_usize(king_dst, row),
            MoveFlags::CASTLE,
        ))
    }
}
