use crate::color::Color;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use crate::r#move::{Move, MoveFlags};

use super::{Game, MoveHistoryEntry};

#[hotpath::measure_all]
impl<const NW: usize> Game<NW> {
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
        // Store state for unmake
        let captured = self.board.get_piece(&mv.dst);
        let old_castling = self.castling_rights;
        let old_en_passant = self.en_passant;
        let old_halfmove = self.halfmove_clock;

        // Make the move on the board
        self.board.remove_piece(&mv.src, piece);
        if let Some(ref cap) = captured {
            self.board.remove_piece(&mv.dst, cap);
        }

        // Handle promotion
        let placed_piece = if mv.flags.contains(MoveFlags::PROMOTION) {
            Piece::new(mv.promotion.unwrap_or(PieceType::Queen), piece.color)
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
            let captured_pawn_pos = Position::new(mv.dst.col, mv.src.row);
            let ep_piece = Piece::new(PieceType::Pawn, piece.color.opposite());
            self.board.remove_piece(&captured_pawn_pos, &ep_piece);
        }

        // Handle castling
        if mv.flags.contains(MoveFlags::CASTLE) {
            let rook = Piece::new(PieceType::Rook, piece.color);
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

            self.board.remove_piece(&rook_from, &rook);
            self.board.place_piece(&rook_to, &rook);
        }

        // Update castling rights
        self.update_castling_rights(mv, piece);

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

            // Handle castling
            if mv.flags.contains(MoveFlags::CASTLE) {
                let rook = Piece::new(PieceType::Rook, self.turn);
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

                self.board.remove_piece(&rook_to, &rook);
                self.board.place_piece(&rook_from, &rook);
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
        let width = self.board.width();
        let height = self.board.height();

        // King moves: lose both sides
        if piece.piece_type == PieceType::King {
            self.castling_rights.set_kingside(piece.color, false);
            self.castling_rights.set_queenside(piece.color, false);
        }

        // Rook moves from its starting corner
        if piece.piece_type == PieceType::Rook {
            self.castling_rights.revoke_at(&mv.src, width, height);
        }

        // Captures on a rook's starting corner
        self.castling_rights.revoke_at(&mv.dst, width, height);
    }
}
