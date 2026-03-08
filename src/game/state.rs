use crate::bitboard::Bitboard;
use crate::color::Color;
use crate::directions::KNIGHT_DELTAS;
use crate::outcome::GameOutcome;
use crate::pieces::{Piece, PieceType};
use crate::position::Position;
use crate::r#move::{Move, MoveFlags};

use super::Game;

#[hotpath::measure_all]
impl<const NW: usize> Game<NW> {
    pub(super) fn is_square_attacked(&self, square: &Position, by_color: Color) -> bool {
        let width = self.board.width();
        let height = self.board.height();
        let occupied = self.board.occupied();
        let enemy = self.board.color_bb(by_color);
        let sq_col = square.col as i32;
        let sq_row = square.row as i32;

        // 1. Pawn attacks
        let pawn_dir: i32 = if by_color == Color::White { -1 } else { 1 };
        let pawn_row = (sq_row + pawn_dir) as usize;
        if pawn_row < height {
            let pawns = self.board.piece_type_bb(PieceType::Pawn) & enemy;
            for col_offset in [-1i32, 1i32] {
                let pawn_col = (sq_col + col_offset) as usize;
                if pawn_col < width {
                    let idx = pawn_row * width + pawn_col;
                    if pawns.get(idx) {
                        return true;
                    }
                }
            }
        }

        // 2. Knight attacks
        let knights = self.board.piece_type_bb(PieceType::Knight) & enemy;
        if !knights.is_empty() {
            for (col_off, row_off) in KNIGHT_DELTAS {
                let nc = (sq_col + col_off) as usize;
                let nr = (sq_row + row_off) as usize;
                if nc < width && nr < height {
                    let idx = nr * width + nc;
                    if knights.get(idx) {
                        return true;
                    }
                }
            }
        }

        // 3. King attacks
        let kings = self.board.piece_type_bb(PieceType::King) & enemy;
        if !kings.is_empty() {
            for col_off in -1..=1i32 {
                for row_off in -1..=1i32 {
                    if col_off == 0 && row_off == 0 {
                        continue;
                    }
                    let kc = (sq_col + col_off) as usize;
                    let kr = (sq_row + row_off) as usize;
                    if kc < width && kr < height {
                        let idx = kr * width + kc;
                        if kings.get(idx) {
                            return true;
                        }
                    }
                }
            }
        }

        // 4. Sliding attacks — rooks/queens along ranks and files
        let rooks_queens = (self.board.piece_type_bb(PieceType::Rook)
            | self.board.piece_type_bb(PieceType::Queen))
            & enemy;
        if !rooks_queens.is_empty() {
            let src_bb = Bitboard::single(square.to_index(width));
            for dir in &self.geometry.orthogonal_steps {
                let ray = self.geometry.ray_attacks(src_bb, dir, occupied);
                if !(ray & rooks_queens).is_empty() {
                    return true;
                }
            }
        }

        // 5. Sliding attacks — bishops/queens along diagonals
        let bishops_queens = (self.board.piece_type_bb(PieceType::Bishop)
            | self.board.piece_type_bb(PieceType::Queen))
            & enemy;
        if !bishops_queens.is_empty() {
            let src_bb = Bitboard::single(square.to_index(width));
            for dir in &self.geometry.diagonal_steps {
                let ray = self.geometry.ray_attacks(src_bb, dir, occupied);
                if !(ray & bishops_queens).is_empty() {
                    return true;
                }
            }
        }

        false
    }

    pub(super) fn is_in_check(&self, color: Color) -> bool {
        let king_pos = match color {
            Color::White => self.white_king_pos,
            Color::Black => self.black_king_pos,
        };
        self.is_square_attacked(&king_pos, color.opposite())
    }

    pub fn is_check(&self) -> bool {
        self.is_in_check(self.turn)
    }

    fn has_any_legal_move(&mut self) -> bool {
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
                    return true;
                }
            }
        }
        false
    }

    pub fn is_checkmate(&mut self) -> bool {
        self.is_check() && !self.has_any_legal_move()
    }

    pub fn is_stalemate(&mut self) -> bool {
        !self.is_check() && !self.has_any_legal_move()
    }

    pub fn is_over(&mut self) -> bool {
        self.halfmove_clock >= 150 || self.is_insufficient_material() || !self.has_any_legal_move()
    }

    pub fn en_passant_square(&self) -> Option<Position> {
        self.en_passant
    }

    pub fn has_legal_en_passant(&mut self) -> bool {
        if let Some(ep_square) = self.en_passant {
            // Check if any pawn can legally capture en passant
            // Look for pawns of the current player that can attack the en passant square
            let direction: i32 = if self.turn == Color::White { 1 } else { -1 };
            let pawn_row = (ep_square.row as i32 - direction) as usize;

            // Check squares to the left and right of the en passant square
            for col_offset in [-1i32, 1i32] {
                let pawn_col = ep_square.col as i32 + col_offset;
                if pawn_col >= 0 && pawn_col < self.board.width() as i32 {
                    let pawn_pos = Position::new(pawn_col as usize, pawn_row);
                    if let Some(piece) = self.board.get_piece(&pawn_pos) {
                        if piece.piece_type == PieceType::Pawn && piece.color == self.turn {
                            // Test in-place: apply ep capture, check, then undo
                            let captured_pawn_pos = Position::new(ep_square.col, pawn_pos.row);
                            let captured_pawn = Piece::new(PieceType::Pawn, self.turn.opposite());

                            self.board.remove_piece(&pawn_pos, &piece);
                            self.board.remove_piece(&captured_pawn_pos, &captured_pawn);
                            self.board.place_piece(&ep_square, &piece);

                            let in_check = self.is_in_check(self.turn);

                            // Restore
                            self.board.remove_piece(&ep_square, &piece);
                            self.board.place_piece(&captured_pawn_pos, &captured_pawn);
                            self.board.place_piece(&pawn_pos, &piece);

                            if !in_check {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Infer move flags (capture, castle, en passant, double push) from the current board state.
    pub fn infer_move_flags(&self, src: &Position, dst: &Position, piece: &Piece) -> MoveFlags {
        let mut flags = MoveFlags::empty();

        if self.board.get_piece(dst).is_some() {
            flags |= MoveFlags::CAPTURE;
        }

        if piece.piece_type == PieceType::King && (dst.col as i32 - src.col as i32).abs() == 2 {
            flags |= MoveFlags::CASTLE;
        }

        if piece.piece_type == PieceType::Pawn {
            if let Some(ep_square) = self.en_passant {
                if *dst == ep_square {
                    flags |= MoveFlags::CAPTURE | MoveFlags::EN_PASSANT;
                }
            }
            if (dst.row as i32 - src.row as i32).abs() == 2 {
                flags |= MoveFlags::DOUBLE_PUSH;
            }
        }

        flags
    }

    /// Parse a LAN move string, with game context to set proper flags (castling, en passant, etc.)
    /// The `from_lan()` method on Move itself lacks game context.
    pub fn move_from_lan(&self, lan: &str) -> Result<Move, String> {
        let base_move = Move::from_lan(lan, self.board.width(), self.board.height())?;

        let piece = self
            .board
            .get_piece(&base_move.src)
            .ok_or_else(|| "No piece at source square".to_string())?;

        let flags = base_move.flags | self.infer_move_flags(&base_move.src, &base_move.dst, &piece);

        Ok(Move {
            src: base_move.src,
            dst: base_move.dst,
            flags,
            promotion: base_move.promotion,
        })
    }

    pub fn outcome(&mut self) -> Option<GameOutcome> {
        if self.halfmove_clock >= 150 {
            return Some(GameOutcome::FiftyMoveRule);
        }

        if self.is_insufficient_material() {
            return Some(GameOutcome::InsufficientMaterial);
        }

        if self.has_any_legal_move() {
            return None;
        }

        // No legal moves: checkmate or stalemate
        if self.is_check() {
            if self.turn == Color::White {
                Some(GameOutcome::BlackWin)
            } else {
                Some(GameOutcome::WhiteWin)
            }
        } else {
            Some(GameOutcome::Stalemate)
        }
    }

    pub fn is_insufficient_material(&self) -> bool {
        let white = self.board.color_bb(Color::White);
        let black = self.board.color_bb(Color::Black);

        let pawns = self.board.piece_type_bb(PieceType::Pawn);
        let rooks = self.board.piece_type_bb(PieceType::Rook);
        let queens = self.board.piece_type_bb(PieceType::Queen);
        let bishops = self.board.piece_type_bb(PieceType::Bishop);
        let knights = self.board.piece_type_bb(PieceType::Knight);

        // If either side has pawns, queens, or rooks, there's sufficient material
        if !(pawns | rooks | queens).is_empty() {
            return false;
        }

        // Now we only have kings, bishops, and knights
        let white_bishops = (bishops & white).count();
        let white_knights = (knights & white).count();
        let black_bishops = (bishops & black).count();
        let black_knights = (knights & black).count();

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

    pub fn to_fen(&mut self) -> String {
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
