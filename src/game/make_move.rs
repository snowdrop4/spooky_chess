use crate::color::Color;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use crate::r#move::{Move, MoveFlags};

use super::{Game, MoveHistoryEntry};

#[hotpath::measure_all]
impl<const W: usize, const H: usize> Game<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
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

        self.apply_move(mv, &piece);
        true
    }

    /// Apply a move that is already known to be legal. Skips legality checking.
    /// Caller must guarantee the move came from `legal_moves()` or equivalent.
    pub fn make_move_unchecked(&mut self, mv: &Move) {
        let piece = self
            .board
            .get_piece(&mv.src)
            .expect("no piece at move source");
        self.apply_move(mv, &piece);
    }

    pub(super) fn apply_move(&mut self, mv: &Move, piece: &Piece) {
        debug_assert!(
            piece.color == self.turn,
            "apply_move: piece color {:?} doesn't match turn {:?}",
            piece.color,
            self.turn,
        );
        debug_assert!(
            self.board.get_piece(&mv.src) == Some(*piece),
            "apply_move: expected {:?} at ({}, {}), found {:?}",
            piece,
            mv.src.col,
            mv.src.row,
            self.board.get_piece(&mv.src),
        );

        // Store state for unmake.
        // Castle moves never capture — the destination may overlap with the
        // castling rook on small boards, but that rook is moved, not captured.
        let captured = if mv.flags.contains(MoveFlags::CASTLE) {
            None
        } else {
            self.board.get_piece(&mv.dst)
        };
        let old_castling = self.castling_rights;
        let old_en_passant = self.en_passant;
        let old_halfmove = self.halfmove_clock;

        // Handle castling rook first: move rook before placing king so pieces
        // don't overlap on the same square (which would corrupt bitboards on
        // small boards where king destination == rook source).
        if mv.flags.contains(MoveFlags::CASTLE) {
            debug_assert!(
                piece.piece_type == PieceType::King,
                "castle move but piece is {:?}",
                piece.piece_type,
            );
            let rook = Piece::new(PieceType::Rook, piece.color);
            let (rook_from, rook_to) = mv.castling_rook_positions(W);
            debug_assert!(
                self.board.get_piece(&rook_from) == Some(rook),
                "castling: expected rook at ({}, {}), found {:?}",
                rook_from.col,
                rook_from.row,
                self.board.get_piece(&rook_from),
            );
            self.board.remove_piece(&rook_from, &rook);
            self.board.place_piece(&rook_to, &rook);
        }

        // Make the move on the board
        self.board.remove_piece(&mv.src, piece);
        if let Some(ref cap) = captured {
            self.board.remove_piece(&mv.dst, cap);
        }

        // Handle promotion
        let placed_piece = if mv.flags.contains(MoveFlags::PROMOTION) {
            debug_assert!(
                mv.promotion.is_some(),
                "PROMOTION flag set but no promotion piece type specified",
            );
            Piece::new(mv.promotion.unwrap_or(PieceType::DEFAULT_PROMOTION), piece.color)
        } else {
            *piece
        };
        self.board.place_piece(&mv.dst, &placed_piece);

        // Update king position if a king moved
        if piece.piece_type == PieceType::King {
            match piece.color {
                Color::White => self.white_king_pos = mv.dst,
                Color::Black => self.black_king_pos = mv.dst,
            }
        }

        // Handle en passant capture
        if mv.flags.contains(MoveFlags::EN_PASSANT) {
            debug_assert!(
                captured.is_none(),
                "en passant move has a captured piece on destination square",
            );
            debug_assert!(
                self.en_passant.is_some(),
                "EN_PASSANT flag set but no en passant square",
            );
            let captured_pawn_pos = Position::new(mv.dst.col, mv.src.row);
            let ep_piece = Piece::new(PieceType::Pawn, piece.color.opposite());
            debug_assert!(
                self.board.get_piece(&captured_pawn_pos) == Some(ep_piece),
                "en passant: expected opponent pawn at ({}, {}), found {:?}",
                captured_pawn_pos.col,
                captured_pawn_pos.row,
                self.board.get_piece(&captured_pawn_pos),
            );
            self.board.remove_piece(&captured_pawn_pos, &ep_piece);
        }

        // Update castling rights
        self.update_castling_rights(mv, piece);

        // Update en passant square
        debug_assert!(
            piece.piece_type != PieceType::Pawn
                || (mv.dst.row as i32 - mv.src.row as i32).abs() <= 2,
            "pawn moved more than 2 rows: from row {} to row {}",
            mv.src.row,
            mv.dst.row,
        );
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

        // Verify king position cache consistency
        debug_assert!(
            self.board.get_piece(&self.white_king_pos)
                == Some(Piece::new(PieceType::King, Color::White)),
            "white_king_pos ({}, {}) desynced after apply_move",
            self.white_king_pos.col,
            self.white_king_pos.row,
        );
        debug_assert!(
            self.board.get_piece(&self.black_king_pos)
                == Some(Piece::new(PieceType::King, Color::Black)),
            "black_king_pos ({}, {}) desynced after apply_move",
            self.black_king_pos.col,
            self.black_king_pos.row,
        );

        // Switch turns (always, even if the game is over)
        self.turn = self.turn.opposite();
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

            // Remove the piece from its destination
            let dst_piece = self
                .board
                .get_piece(&mv.dst)
                .expect("no piece at move dst during unmake");
            debug_assert!(
                dst_piece.color == self.turn,
                "unmake: piece at destination has color {:?} but expected {:?}",
                dst_piece.color,
                self.turn,
            );
            self.board.remove_piece(&mv.dst, &dst_piece);

            // Restore original piece to source
            let original_piece = if mv.flags.contains(MoveFlags::PROMOTION) {
                Piece::new(PieceType::Pawn, self.turn)
            } else {
                dst_piece
            };
            self.board.place_piece(&mv.src, &original_piece);

            // Restore king position if a king moved
            if original_piece.piece_type == PieceType::King {
                match original_piece.color {
                    Color::White => self.white_king_pos = mv.src,
                    Color::Black => self.black_king_pos = mv.src,
                }
            }

            // Restore captured piece
            if let Some(cap) = captured {
                self.board.place_piece(&mv.dst, &cap);
            }

            // Handle en passant
            if mv.flags.contains(MoveFlags::EN_PASSANT) {
                let captured_pawn_pos = Position::new(mv.dst.col, mv.src.row);
                let ep_piece = Piece::new(PieceType::Pawn, self.turn.opposite());
                self.board.place_piece(&captured_pawn_pos, &ep_piece);
            }

            // Restore castling rook: must happen after king is removed from
            // dst, so rook doesn't overlap with king on small boards.
            if mv.flags.contains(MoveFlags::CASTLE) {
                let rook = Piece::new(PieceType::Rook, self.turn);
                let (rook_from, rook_to) = mv.castling_rook_positions(W);
                debug_assert!(
                    self.board.get_piece(&rook_to) == Some(rook),
                    "unmake castling: expected rook at ({}, {}), found {:?}",
                    rook_to.col,
                    rook_to.row,
                    self.board.get_piece(&rook_to),
                );
                self.board.remove_piece(&rook_to, &rook);
                self.board.place_piece(&rook_from, &rook);
            }

            // Restore state
            self.castling_rights = old_castling;
            self.en_passant = old_en_passant;
            self.halfmove_clock = old_halfmove;

            if self.turn == Color::Black {
                debug_assert!(
                    self.fullmove_number >= 2,
                    "fullmove_number would underflow in unmake_move",
                );
                self.fullmove_number -= 1;
            }

            // Verify king position cache consistency after unmake
            debug_assert!(
                self.board.get_piece(&self.white_king_pos)
                    == Some(Piece::new(PieceType::King, Color::White)),
                "white_king_pos ({}, {}) desynced after unmake_move",
                self.white_king_pos.col,
                self.white_king_pos.row,
            );
            debug_assert!(
                self.board.get_piece(&self.black_king_pos)
                    == Some(Piece::new(PieceType::King, Color::Black)),
                "black_king_pos ({}, {}) desynced after unmake_move",
                self.black_king_pos.col,
                self.black_king_pos.row,
            );

            true
        } else {
            false
        }
    }

    fn update_castling_rights(&mut self, mv: &Move, piece: &Piece) {
        // King moves: lose both sides
        if piece.piece_type == PieceType::King {
            self.castling_rights.set_kingside(piece.color, false);
            self.castling_rights.set_queenside(piece.color, false);
        }

        // Rook moves from its starting corner
        if piece.piece_type == PieceType::Rook {
            self.castling_rights.revoke_at(&mv.src, W, H);
        }

        // Captures on a rook's starting corner
        self.castling_rights.revoke_at(&mv.dst, W, H);
    }
}
