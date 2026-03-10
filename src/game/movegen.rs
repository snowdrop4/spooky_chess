use crate::bitboard::{Bitboard, DirStep};
use crate::color::Color;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use crate::r#move::{Move, MoveFlags};

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
        let mut pseudo_legal = Vec::new();
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
        let opponent = piece.color.opposite();

        let captured =
            if mv.flags.contains(MoveFlags::CAPTURE) && !mv.flags.contains(MoveFlags::EN_PASSANT) {
                let dst_idx = mv.dst.to_index(W);
                let pt = self.board.piece_type_at(dst_idx).unwrap();
                Some(Piece::new(pt, opponent))
            } else {
                None
            };

        // Handle castling rook: must move rook before placing king so pieces
        // don't overlap on the same square (which would corrupt bitboards).
        let castle_rook = if mv.flags.contains(MoveFlags::CASTLE) {
            let rook = Piece::new(PieceType::Rook, piece.color);
            let (rook_from, rook_to) = if mv.dst.col > mv.src.col {
                (
                    Position::new(W - 1, mv.src.row),
                    Position::new(mv.dst.col - 1, mv.dst.row),
                )
            } else {
                (
                    Position::new(0, mv.src.row),
                    Position::new(mv.dst.col + 1, mv.dst.row),
                )
            };
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
            Piece::new(mv.promotion.unwrap_or(PieceType::Queen), piece.color)
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

    pub fn legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut pseudo_legal = Vec::new();

        let color = self.turn;
        for idx in self.board.color_bb(color).iter_ones() {
            let pt = self.board.piece_type_at(idx).unwrap();
            let pos = Position::from_index(idx, W);
            let piece = Piece::new(pt, color);

            pseudo_legal.clear();
            self.generate_pseudo_legal_moves_for_piece_into(&pos, &piece, &mut pseudo_legal);

            for mv in pseudo_legal.iter() {
                if self.is_pseudo_legal_move_legal(mv, &piece) {
                    moves.push(*mv);
                }
            }
        }

        moves
    }

    pub fn pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        for (pos, piece) in self.board.pieces_iter(self.turn) {
            self.generate_pseudo_legal_moves_for_piece_into(&pos, &piece, &mut moves);
        }

        moves
    }

    pub fn legal_moves_for_position(&mut self, src: &Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut pseudo_legal = Vec::new();

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
        moves: &mut Vec<Move>,
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
        moves: &mut Vec<Move>,
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
            if src.row == promo_row {
                for pt in &[
                    PieceType::Queen,
                    PieceType::Rook,
                    PieceType::Bishop,
                    PieceType::Knight,
                ] {
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
        if src.row == start_row && !push.is_empty() {
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
            if src.row == promo_row {
                for pt in &[
                    PieceType::Queen,
                    PieceType::Rook,
                    PieceType::Bishop,
                    PieceType::Knight,
                ] {
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
        moves: &mut Vec<Move>,
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

    fn generate_sliding_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        dirs: &[DirStep<{ (W * H).div_ceil(64) }>],
        moves: &mut Vec<Move>,
    ) {
        let occupied = self.board.occupied();
        let own_color = self.board.color_bb(piece.color);
        let src_bb = Bitboard::single(src.to_index(W));
        let geo = Self::geo();

        for dir in dirs {
            let ray = geo.ray_attacks(src_bb, dir, occupied);
            let targets = ray.andnot(own_color);

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
    }

    fn generate_pseudo_legal_bishop_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
        self.generate_sliding_moves_into(src, piece, &Self::geo().diagonal_steps, moves)
    }

    fn generate_pseudo_legal_rook_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
        self.generate_sliding_moves_into(src, piece, &Self::geo().orthogonal_steps, moves)
    }

    fn generate_pseudo_legal_queen_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
        self.generate_sliding_moves_into(src, piece, &Self::geo().orthogonal_steps, moves);
        self.generate_sliding_moves_into(src, piece, &Self::geo().diagonal_steps, moves);
    }

    fn generate_pseudo_legal_king_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
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
            let row = src.row;
            let opponent = piece.color.opposite();

            // Kingside: king to col king+2, rook from last col to king+1
            if self.castling_rights.has_kingside(piece.color) {
                let king_dst = src.col + 2;
                let rook_col = W - 1;
                if king_dst < rook_col {
                    self.try_generate_castle(src, row, king_dst, rook_col, opponent, moves);
                }
            }

            // Queenside: king to col king-2, rook from col 0 to king-1
            if self.castling_rights.has_queenside(piece.color) && src.col >= 2 {
                let king_dst = src.col - 2;
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
        moves: &mut Vec<Move>,
    ) {
        let occupied = self.board.occupied();

        // All squares between king and rook must be empty
        let (lo, hi) = if king_src.col < rook_col {
            (king_src.col + 1, rook_col)
        } else {
            (rook_col + 1, king_src.col)
        };
        for col in lo..hi {
            if occupied.get(row * W + col) {
                return;
            }
        }

        // All squares the king passes through (and lands on) must not be attacked
        let (path_lo, path_hi) = if king_dst > king_src.col {
            (king_src.col + 1, king_dst + 1)
        } else {
            (king_dst, king_src.col)
        };
        for col in path_lo..path_hi {
            if self.is_square_attacked(&Position::new(col, row), opponent) {
                return;
            }
        }

        moves.push(Move::from_position(
            *king_src,
            Position::new(king_dst, row),
            MoveFlags::CASTLE,
        ));
    }
}
