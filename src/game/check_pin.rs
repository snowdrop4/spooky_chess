use crate::bitboard::Bitboard;
use crate::color::Color;
use crate::pieces::PieceType;

use super::Game;

pub(super) struct CheckPinInfo<const NW: usize, const N: usize> {
    pub num_checkers: u8,
    pub check_mask: Bitboard<NW>,
    pub pinned: Bitboard<NW>,
    pin_masks: [Bitboard<NW>; N],
    pub king_danger_squares: Bitboard<NW>,
}

impl<const NW: usize, const N: usize> CheckPinInfo<NW, N> {
    /// Returns the pin ray mask for a pinned piece (squares it may move to).
    /// Only call for pieces known to be pinned.
    pub fn get_pin_mask(&self, piece_idx: usize) -> Bitboard<NW> {
        debug_assert!(
            !self.pin_masks[piece_idx].is_empty(),
            "get_pin_mask called for non-pinned piece at index {}",
            piece_idx
        );
        self.pin_masks[piece_idx]
    }
}

#[hotpath::measure_all]
impl<const W: usize, const H: usize> Game<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    pub(super) fn compute_check_pin_info(
        &self,
    ) -> CheckPinInfo<{ (W * H).div_ceil(64) }, { W * H }> {
        let geo = Self::geo();
        let color = self.turn;
        let opponent = color.opposite();
        let king_pos = match color {
            Color::White => self.white_king_pos,
            Color::Black => self.black_king_pos,
        };
        let king_idx = king_pos.to_index(W);

        let occupied = self.board.occupied();
        let own = self.board.color_bb(color);
        let enemy = self.board.color_bb(opponent);

        let mut num_checkers: u8 = 0;
        let mut check_mask = Bitboard::empty();
        let mut pinned = Bitboard::empty();
        let mut pin_masks = [Bitboard::empty(); W * H];

        // Orthogonal: [0]=N(left), [1]=S(right), [2]=E(left), [3]=W(right)
        const ORTHO_IS_LEFT: [bool; 4] = [true, false, true, false];
        let rooks_queens = (self.board.piece_type_bb(PieceType::Rook)
            | self.board.piece_type_bb(PieceType::Queen))
            & enemy;
        if !rooks_queens.is_empty() {
            for (dir, &is_left) in ORTHO_IS_LEFT.iter().enumerate() {
                Self::scan_ray_for_check_pin(
                    &geo.ray_orthogonal,
                    dir,
                    is_left,
                    king_idx,
                    occupied,
                    own,
                    rooks_queens,
                    &mut num_checkers,
                    &mut check_mask,
                    &mut pinned,
                    &mut pin_masks,
                );
            }
        }

        // Diagonal: [0]=NE(left), [1]=NW(left), [2]=SE(right), [3]=SW(right)
        const DIAG_IS_LEFT: [bool; 4] = [true, true, false, false];
        let bishops_queens = (self.board.piece_type_bb(PieceType::Bishop)
            | self.board.piece_type_bb(PieceType::Queen))
            & enemy;
        if !bishops_queens.is_empty() {
            for (dir, &is_left) in DIAG_IS_LEFT.iter().enumerate() {
                Self::scan_ray_for_check_pin(
                    &geo.ray_diagonal,
                    dir,
                    is_left,
                    king_idx,
                    occupied,
                    own,
                    bishops_queens,
                    &mut num_checkers,
                    &mut check_mask,
                    &mut pinned,
                    &mut pin_masks,
                );
            }
        }

        // Knight checks
        let knights = self.board.piece_type_bb(PieceType::Knight) & enemy;
        if !knights.is_empty() {
            let knight_checks = geo.knight_attacks(king_idx) & knights;
            if !knight_checks.is_empty() {
                num_checkers += knight_checks.count() as u8;
                check_mask |= knight_checks;
            }
        }

        // Pawn checks
        let pawns = self.board.piece_type_bb(PieceType::Pawn) & enemy;
        if !pawns.is_empty() {
            let is_white = color == Color::White;
            let pawn_checks = geo.pawn_attacks(king_idx, is_white) & pawns;
            if !pawn_checks.is_empty() {
                num_checkers += pawn_checks.count() as u8;
                check_mask |= pawn_checks;
            }
        }

        // If not in check, check_mask = all ones (no restriction on non-king moves)
        if num_checkers == 0 {
            check_mask = !Bitboard::empty();
        }

        // Compute king danger squares (all enemy attacks with our king removed from occupancy)
        let occupied_no_king = occupied.andnot(Bitboard::single(king_idx));
        let king_danger_squares = self.compute_enemy_attacks(opponent, occupied_no_king);

        CheckPinInfo {
            num_checkers,
            check_mask,
            pinned,
            pin_masks,
            king_danger_squares,
        }
    }

    /// Scan a single ray direction from the king for checks and pins.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn scan_ray_for_check_pin(
        ray_table: &[[Bitboard<{ (W * H).div_ceil(64) }>; W * H]; 4],
        dir: usize,
        is_left: bool,
        king_idx: usize,
        occupied: Bitboard<{ (W * H).div_ceil(64) }>,
        own: Bitboard<{ (W * H).div_ceil(64) }>,
        enemy_sliders: Bitboard<{ (W * H).div_ceil(64) }>,
        num_checkers: &mut u8,
        check_mask: &mut Bitboard<{ (W * H).div_ceil(64) }>,
        pinned: &mut Bitboard<{ (W * H).div_ceil(64) }>,
        pin_masks: &mut [Bitboard<{ (W * H).div_ceil(64) }>; W * H],
    ) {
        let ray = ray_table[dir][king_idx];
        let blockers = ray & occupied;
        if blockers.is_empty() {
            return;
        }

        let first_idx = if is_left {
            blockers
                .lowest_bit_index()
                .expect("blockers confirmed non-empty")
        } else {
            blockers
                .highest_bit_index()
                .expect("blockers confirmed non-empty")
        };
        let first_bb = Bitboard::single(first_idx);

        if !(first_bb & enemy_sliders).is_empty() {
            // Direct check from enemy slider
            *num_checkers += 1;
            // Squares between king and checker (inclusive of checker, exclusive of king)
            *check_mask |= ray ^ ray_table[dir][first_idx];
        } else if !(first_bb & own).is_empty() {
            // First piece is friendly — check for pin (enemy slider behind it)
            let beyond = ray_table[dir][first_idx];
            let beyond_blockers = beyond & occupied;
            if !beyond_blockers.is_empty() {
                let second_idx = if is_left {
                    beyond_blockers
                        .lowest_bit_index()
                        .expect("beyond_blockers confirmed non-empty")
                } else {
                    beyond_blockers
                        .highest_bit_index()
                        .expect("beyond_blockers confirmed non-empty")
                };
                if !(Bitboard::single(second_idx) & enemy_sliders).is_empty() {
                    // Pin detected
                    *pinned |= first_bb;
                    // Pin mask: squares between king and pinner (inclusive of both endpoints
                    // on the ray, exclusive of king)
                    pin_masks[first_idx] = ray ^ ray_table[dir][second_idx];
                }
            }
        }
    }

    /// Compute the union of all squares attacked by pieces of `enemy_color`,
    /// using the given `occupied` bitboard (typically with our king removed).
    fn compute_enemy_attacks(
        &self,
        enemy_color: Color,
        occupied: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        let geo = Self::geo();
        let enemy = self.board.color_bb(enemy_color);
        let mut attacks = Bitboard::empty();

        // Pawns
        let pawns = self.board.piece_type_bb(PieceType::Pawn) & enemy;
        let is_white = enemy_color == Color::White;
        for idx in pawns.iter_ones() {
            attacks |= geo.pawn_attacks(idx, is_white);
        }

        // Knights
        let knights = self.board.piece_type_bb(PieceType::Knight) & enemy;
        for idx in knights.iter_ones() {
            attacks |= geo.knight_attacks(idx);
        }

        // King
        let enemy_king_idx = match enemy_color {
            Color::White => self.white_king_pos.to_index(W),
            Color::Black => self.black_king_pos.to_index(W),
        };
        attacks |= geo.king_attacks(enemy_king_idx);

        // Rooks + Queens (orthogonal)
        let queens = self.board.piece_type_bb(PieceType::Queen) & enemy;
        let rooks_queens = (self.board.piece_type_bb(PieceType::Rook) & enemy) | queens;
        for idx in rooks_queens.iter_ones() {
            attacks |= geo.orthogonal_attacks(idx, occupied);
        }

        // Bishops + Queens (diagonal)
        let bishops_queens = (self.board.piece_type_bb(PieceType::Bishop) & enemy) | queens;
        for idx in bishops_queens.iter_ones() {
            attacks |= geo.diagonal_attacks(idx, occupied);
        }

        attacks
    }
}
