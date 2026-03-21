use crate::color::Color;
use crate::r#move::{Move, MoveFlags};
use crate::outcome::{GameOutcome, TurnState};
use crate::pieces::{Piece, PieceType};
use crate::position::Position;

use super::Game;

#[hotpath::measure_all]
impl<const W: usize, const H: usize> Game<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    pub(super) fn is_square_attacked(&self, square: &Position, by_color: Color) -> bool {
        let occupied = self.board.occupied();
        let enemy = self.board.color_bb(by_color);
        let sq_idx = square.to_index(W);
        let geo = Self::geo();

        // 1. Pawn attacks
        let pawns = self.board.piece_type_bb(PieceType::Pawn) & enemy;
        if !pawns.is_empty() {
            let pawn_attackers = geo.pawn_attacks(sq_idx, by_color != Color::White);
            if !(pawn_attackers & pawns).is_empty() {
                return true;
            }
        }

        // 2. Knight attacks
        let knights = self.board.piece_type_bb(PieceType::Knight) & enemy;
        if !knights.is_empty() && !(geo.knight_attacks(sq_idx) & knights).is_empty() {
            return true;
        }

        // 3. King attacks
        let kings = self.board.piece_type_bb(PieceType::King) & enemy;
        if !kings.is_empty() && !(geo.king_attacks(sq_idx) & kings).is_empty() {
            return true;
        }

        // 4. Sliding attacks — rooks/queens along ranks and files
        let queens = self.board.piece_type_bb(PieceType::Queen) & enemy;
        let rooks_queens = (self.board.piece_type_bb(PieceType::Rook) & enemy) | queens;
        if !rooks_queens.is_empty() {
            let ortho = geo.orthogonal_attacks(sq_idx, occupied);
            if !(ortho & rooks_queens).is_empty() {
                return true;
            }
        }

        // 5. Sliding attacks — bishops/queens along diagonals
        let bishops_queens = (self.board.piece_type_bb(PieceType::Bishop) & enemy) | queens;
        if !bishops_queens.is_empty() {
            let diag = geo.diagonal_attacks(sq_idx, occupied);
            if !(diag & bishops_queens).is_empty() {
                return true;
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
        self.for_each_legal_move(|_mv| true)
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
            let ep_idx = ep_square.to_index(W);
            let is_white = self.turn == Color::White;
            let geo = Self::geo();

            // Find our pawns that can attack the ep square using reverse pawn attacks
            let candidates = geo.pawn_attacks(ep_idx, !is_white)
                & self.board.piece_type_bb(PieceType::Pawn)
                & self.board.color_bb(self.turn);

            if candidates.is_empty() {
                return false;
            }

            let pawn = Piece::new(PieceType::Pawn, self.turn);
            let captured_pawn = Piece::new(PieceType::Pawn, self.turn.opposite());

            for pawn_idx in candidates.iter_ones() {
                let pawn_pos = Position::from_index(pawn_idx, W);
                let captured_pawn_pos = Position::new(ep_square.col, pawn_pos.row);

                self.board.remove_piece(&pawn_pos, &pawn);
                self.board.remove_piece(&captured_pawn_pos, &captured_pawn);
                self.board.place_piece(&ep_square, &pawn);

                let in_check = self.is_in_check(self.turn);

                self.board.remove_piece(&ep_square, &pawn);
                self.board.place_piece(&captured_pawn_pos, &captured_pawn);
                self.board.place_piece(&pawn_pos, &pawn);

                if !in_check {
                    return true;
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
            if let Some(ep_square) = self.en_passant
                && *dst == ep_square
            {
                flags |= MoveFlags::CAPTURE | MoveFlags::EN_PASSANT;
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
        let base_move = Move::from_lan(lan, W, H)?;

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

    pub fn move_to_lan(&self, mv: &Move) -> String {
        mv.to_lan()
    }

    pub fn move_to_san(&mut self, mv: &Move) -> String {
        let mut san = String::new();

        // Castling
        if mv.flags.contains(MoveFlags::CASTLE) {
            if mv.dst.col > mv.src.col {
                san.push_str("O-O");
            } else {
                san.push_str("O-O-O");
            }
        } else {
            let piece = self
                .board
                .get_piece(&mv.src)
                .expect("No piece at source square for SAN conversion");

            if piece.piece_type != PieceType::Pawn {
                san.push(piece.piece_type.to_san_char());

                // Disambiguation: find other pieces of same type that can reach same destination
                let legal = self.legal_moves();
                let ambiguous: Vec<&Move> = legal
                    .iter()
                    .filter(|m| {
                        m.src != mv.src
                            && m.dst == mv.dst
                            && self
                                .board
                                .get_piece(&m.src)
                                .map(|p| p.piece_type == piece.piece_type)
                                .unwrap_or(false)
                    })
                    .collect();

                if !ambiguous.is_empty() {
                    let same_file = ambiguous.iter().any(|m| m.src.col == mv.src.col);
                    let same_rank = ambiguous.iter().any(|m| m.src.row == mv.src.row);

                    if !same_file {
                        san.push((b'a' + mv.src.col as u8) as char);
                    } else if !same_rank {
                        san.push_str(&(mv.src.row + 1).to_string());
                    } else {
                        san.push((b'a' + mv.src.col as u8) as char);
                        san.push_str(&(mv.src.row + 1).to_string());
                    }
                }
            } else if mv.flags.contains(MoveFlags::CAPTURE) {
                // Pawn captures: prefix with source file
                san.push((b'a' + mv.src.col as u8) as char);
            }

            if mv.flags.contains(MoveFlags::CAPTURE) {
                san.push('x');
            }

            san.push_str(&mv.dst.to_algebraic());

            if let Some(promo) = mv.promotion {
                san.push('=');
                san.push(promo.to_san_char());
            }
        }

        // Check/checkmate suffix
        self.make_move_unchecked(mv);
        if self.is_in_check(self.turn) {
            if !self.has_any_legal_move() {
                san.push('#');
            } else {
                san.push('+');
            }
        }
        self.unmake_move();

        san
    }

    pub fn move_from_san(&mut self, san: &str) -> Result<Move, String> {
        // Strip check/mate suffixes
        let san = san.trim_end_matches(['+', '#']);

        if san.is_empty() {
            return Err("Empty SAN string".to_string());
        }

        let legal = self.legal_moves();

        // Castling (accept both O and 0)
        if san == "O-O" || san == "0-0" {
            return legal
                .into_iter()
                .find(|m| m.flags.contains(MoveFlags::CASTLE) && m.dst.col > m.src.col)
                .ok_or_else(|| "Kingside castling not available".to_string());
        }
        if san == "O-O-O" || san == "0-0-0" {
            return legal
                .into_iter()
                .find(|m| m.flags.contains(MoveFlags::CASTLE) && m.dst.col < m.src.col)
                .ok_or_else(|| "Queenside castling not available".to_string());
        }

        let chars: Vec<char> = san.chars().collect();
        let mut idx = 0;

        // Piece type
        let piece_type = if idx < chars.len() && chars[idx].is_ascii_uppercase() {
            if let Some(pt) = PieceType::from_san_char(chars[idx]) {
                idx += 1;
                pt
            } else {
                return Err(format!("Invalid piece character: {}", chars[idx]));
            }
        } else {
            PieceType::Pawn
        };

        // Parse the remaining: optional disambiguation, optional 'x', destination, optional '=P'
        // We need to find the destination square, which is the last 2 chars before optional promotion
        let mut promo_type: Option<PieceType> = None;
        let mut end = chars.len();

        // Check for promotion suffix: =Q, =R, =B, =N
        if end >= 2 && chars[end - 2] == '=' {
            promo_type = PieceType::from_san_char(chars[end - 1]);
            if promo_type.is_none() {
                return Err(format!("Invalid promotion piece: {}", chars[end - 1]));
            }
            end -= 2;
        }

        // Last 2 chars before promotion are the destination square
        if end < idx + 2 {
            return Err("SAN too short to contain destination square".to_string());
        }
        let dst_file = chars[end - 2];
        let dst_rank = chars[end - 1];
        if !dst_file.is_ascii_lowercase() || !dst_rank.is_ascii_digit() {
            return Err(format!(
                "Invalid destination square: {}{}",
                dst_file, dst_rank
            ));
        }
        let dst = Position::from_algebraic(&format!("{}{}", dst_file, dst_rank))?;
        let disambig = &chars[idx..end - 2];

        // Parse disambiguation (skip 'x' if present)
        let mut file_hint: Option<usize> = None;
        let mut rank_hint: Option<usize> = None;
        for &c in disambig {
            if c == 'x' {
                continue;
            }
            if c.is_ascii_lowercase() {
                file_hint = Some((c as u8 - b'a') as usize);
            } else if c.is_ascii_digit() {
                rank_hint = Some((c as u8 - b'1') as usize);
            }
        }

        // Filter legal moves
        let candidates: Vec<Move> = legal
            .into_iter()
            .filter(|m| {
                if m.dst != dst {
                    return false;
                }
                let p = match self.board.get_piece(&m.src) {
                    Some(p) => p,
                    None => return false,
                };
                if p.piece_type != piece_type {
                    return false;
                }
                if let Some(f) = file_hint
                    && m.src.col != f
                {
                    return false;
                }
                if let Some(r) = rank_hint
                    && m.src.row != r
                {
                    return false;
                }
                if let Some(pt) = promo_type {
                    if m.promotion != Some(pt) {
                        return false;
                    }
                } else if m.promotion.is_some() {
                    return false;
                }
                true
            })
            .collect();

        match candidates.len() {
            0 => Err(format!("No legal move matches SAN: {}", san)),
            1 => Ok(candidates
                .into_iter()
                .next()
                .expect("move_from_san: candidates vec confirmed to have exactly one element")),
            _ => Err(format!(
                "Ambiguous SAN: {} matches {} moves",
                san,
                candidates.len()
            )),
        }
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

    pub fn turn_state(&mut self) -> TurnState {
        if self.halfmove_clock >= 150 {
            return TurnState::Over(GameOutcome::FiftyMoveRule);
        }

        if self.is_insufficient_material() {
            return TurnState::Over(GameOutcome::InsufficientMaterial);
        }

        let moves = self.legal_moves();
        if !moves.is_empty() {
            return TurnState::Ongoing(moves);
        }

        let outcome = if self.is_check() {
            if self.turn == Color::White {
                GameOutcome::BlackWin
            } else {
                GameOutcome::WhiteWin
            }
        } else {
            GameOutcome::Stalemate
        };

        TurnState::Over(outcome)
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

        // Check square colors of all bishops (both white and black)
        for idx in bishops.iter_ones() {
            // A square is light if (col + row) is even
            let col = idx % W;
            let row = idx / W;
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
