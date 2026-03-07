use crate::color::Color;
use crate::pieces::{Piece, PieceType, KNIGHT_DELTAS};
use crate::position::Position;
use crate::r#move::{Move, MoveFlags};

use super::Game;

#[hotpath::measure_all]
impl<const NW: usize> Game<NW> {
    pub fn is_legal_move(&mut self, mv: &Move) -> bool {
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

    /// Test whether a pseudo-legal move is actually legal (doesn't leave own king in check).
    /// Temporarily makes the move on the board, checks, then unmakes.
    pub(super) fn is_pseudo_legal_move_legal(&mut self, mv: &Move, piece: &Piece) -> bool {
        let width = self.board.width();
        let opponent = piece.color.opposite();

        let captured =
            if mv.flags.contains(MoveFlags::CAPTURE) && !mv.flags.contains(MoveFlags::EN_PASSANT) {
                let dst_idx = mv.dst.to_index(width);
                let pt = self.board.piece_type_at(dst_idx).unwrap();
                Some(Piece::new(pt, opponent))
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

        !in_check
    }

    pub fn legal_moves(&mut self) -> Vec<Move> {
        let mut moves = Vec::new();
        let mut pseudo_legal = Vec::new();

        let color = self.turn;
        let width = self.board.width();
        for idx in self.board.color_bb(color).iter_ones() {
            let pt = self.board.piece_type_at(idx).unwrap();
            let pos = Position::from_index(idx, width);
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

        if let Some(piece) = self.board.get_piece(src) {
            if piece.color != self.turn {
                return moves;
            }

            let mut pseudo_legal = Vec::new();
            self.generate_pseudo_legal_moves_for_piece_into(src, &piece, &mut pseudo_legal);

            for mv in pseudo_legal {
                if self.is_pseudo_legal_move_legal(&mv, &piece) {
                    moves.push(mv);
                }
            }
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
        let own_color = self.board.color_bb(piece.color);
        let width = self.board.width();
        let height = self.board.height();

        let direction: i32 = if piece.color == Color::White { 1 } else { -1 };

        let start_row = if piece.color == Color::White {
            1
        } else {
            height - 2
        };

        let promo_row = if piece.color == Color::White {
            height - 2
        } else {
            1
        };

        // Single push
        let dst_row = (src.row as i32 + direction) as usize;

        if dst_row < height {
            let idx = dst_row * width + src.col;

            if !occupied.get(idx) {
                let dst_position = Position::new(src.col, dst_row);

                if src.row == promo_row {
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
            let between_row = (src.row as i32 + direction) as usize;
            let to_idx = to_row * width + src.col;
            let between_idx = between_row * width + src.col;

            if !occupied.get(to_idx) && !occupied.get(between_idx) {
                moves.push(Move::from_position(
                    *src,
                    Position::new(src.col, to_row),
                    MoveFlags::DOUBLE_PUSH,
                ));
            }
        }

        // Captures
        for col_offset in &[-1i32, 1i32] {
            let dst_col = (src.col as i32 + col_offset) as usize;
            let dst_row = (src.row as i32 + direction) as usize;

            if dst_col < width && dst_row < height {
                let idx = dst_row * width + dst_col;

                // Regular capture: occupied by enemy
                if occupied.get(idx) && !own_color.get(idx) {
                    let dst_position = Position::new(dst_col, dst_row);

                    if src.row == promo_row {
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
                        moves.push(Move::from_position(
                            *src,
                            Position::new(dst_col, dst_row),
                            MoveFlags::CAPTURE,
                        ));
                    }
                }

                // En passant
                if let Some(ep) = self.en_passant {
                    let dst_position = Position::new(dst_col, dst_row);

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
    }

    fn generate_pseudo_legal_knight_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
        let own_color = self.board.color_bb(piece.color);
        let occupied = self.board.occupied();
        let width = self.board.width();
        let height = self.board.height();

        for (col_offset, row_offset) in &KNIGHT_DELTAS {
            let dst_col = (src.col as i32 + col_offset) as usize;
            let dst_row = (src.row as i32 + row_offset) as usize;

            if dst_col < width && dst_row < height {
                let idx = dst_row * width + dst_col;

                if own_color.get(idx) {
                    continue;
                }

                let to = Position::new(dst_col, dst_row);

                if occupied.get(idx) {
                    moves.push(Move::from_position(*src, to, MoveFlags::CAPTURE));
                } else {
                    moves.push(Move::from_position(*src, to, MoveFlags::empty()));
                }
            }
        }
    }

    fn generate_sliding_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        directions: &[(i32, i32)],
        moves: &mut Vec<Move>,
    ) {
        let occupied = self.board.occupied();
        let own_color = self.board.color_bb(piece.color);
        let width = self.board.width();
        let height = self.board.height();

        for (col_dir, row_dir) in directions {
            let mut distance = 1;

            loop {
                let dst_col = (src.col as i32 + col_dir * distance) as usize;
                let dst_row = (src.row as i32 + row_dir * distance) as usize;

                if dst_col >= width || dst_row >= height {
                    break;
                }

                let idx = dst_row * width + dst_col;

                if occupied.get(idx) {
                    if !own_color.get(idx) {
                        let dst_position = Position::new(dst_col, dst_row);
                        moves.push(Move::from_position(*src, dst_position, MoveFlags::CAPTURE));
                    }

                    break;
                } else {
                    let dst_position = Position::new(dst_col, dst_row);
                    moves.push(Move::from_position(*src, dst_position, MoveFlags::empty()));
                }

                distance += 1;
            }
        }
    }

    fn generate_pseudo_legal_bishop_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
        let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        self.generate_sliding_moves_into(src, piece, &directions, moves)
    }

    fn generate_pseudo_legal_rook_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        self.generate_sliding_moves_into(src, piece, &directions, moves)
    }

    fn generate_pseudo_legal_queen_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
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
        self.generate_sliding_moves_into(src, piece, &directions, moves)
    }

    fn generate_pseudo_legal_king_moves_into(
        &self,
        src: &Position,
        piece: &Piece,
        moves: &mut Vec<Move>,
    ) {
        let own_color = self.board.color_bb(piece.color);
        let occupied = self.board.occupied();
        let width = self.board.width();
        let height = self.board.height();

        // Regular moves
        for col_offset in -1..=1i32 {
            for row_offset in -1..=1i32 {
                if col_offset == 0 && row_offset == 0 {
                    continue;
                }

                let dst_col = (src.col as i32 + col_offset) as usize;
                let dst_row = (src.row as i32 + row_offset) as usize;

                if dst_col < width && dst_row < height {
                    let idx = dst_row * width + dst_col;

                    if own_color.get(idx) {
                        continue;
                    }

                    let dst_position = Position::new(dst_col, dst_row);

                    if occupied.get(idx) {
                        moves.push(Move::from_position(*src, dst_position, MoveFlags::CAPTURE));
                    } else {
                        moves.push(Move::from_position(*src, dst_position, MoveFlags::empty()));
                    }
                }
            }
        }

        // Castling (only if enabled and for 8x8 boards)
        if self.castling_enabled && width == 8 && height == 8 && !self.is_in_check(piece.color) {
            let row = if piece.color == Color::White { 0 } else { 7 };

            // Kingside
            if ((piece.color == Color::White && self.castling_rights.white_kingside)
                || (piece.color == Color::Black && self.castling_rights.black_kingside))
                && !occupied.get(row * 8 + 5)
                && !occupied.get(row * 8 + 6)
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
                && !occupied.get(row * 8 + 1)
                && !occupied.get(row * 8 + 2)
                && !occupied.get(row * 8 + 3)
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
    }
}
