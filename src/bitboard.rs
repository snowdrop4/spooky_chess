use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

/// Compute the number of u64 words needed for a board of given dimensions.
#[inline]
pub const fn nw_for_board(width: u8, height: u8) -> usize {
    ((width as u16 * height as u16) as usize).div_ceil(64)
}

/// A fixed-size bitboard parameterized by the number of u64 words.
/// `NW` = number of active words = ceil(width*height / 64).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bitboard<const NW: usize> {
    words: [u64; NW],
}

impl<const NW: usize> Bitboard<NW> {
    /// All bits zero.
    #[inline]
    pub const fn empty() -> Self {
        Bitboard { words: [0; NW] }
    }

    /// Single bit set at `index`.
    #[inline]
    #[hotpath::measure]
    pub fn single(index: usize) -> Self {
        debug_assert!(index < NW * 64);
        let mut bb = Self::empty();
        bb.words[index / 64] = 1u64 << (index % 64);
        bb
    }

    /// Construct from raw words.
    #[inline]
    pub const fn from_words(words: [u64; NW]) -> Self {
        Bitboard { words }
    }

    /// Test whether bit `index` is set.
    #[inline]
    #[hotpath::measure]
    pub fn get(&self, index: usize) -> bool {
        debug_assert!(index < NW * 64);
        (self.words[index / 64] >> (index % 64)) & 1 != 0
    }

    /// Return the bit at `index` as a `u64` (0 or 1). Branchless.
    #[inline]
    pub fn bit_at(&self, index: usize) -> u64 {
        debug_assert!(index < NW * 64);
        (self.words[index / 64] >> (index % 64)) & 1
    }

    /// Set bit `index` to 1.
    #[inline]
    #[hotpath::measure]
    pub fn set(&mut self, index: usize) {
        debug_assert!(index < NW * 64);
        self.words[index / 64] |= 1u64 << (index % 64);
    }

    /// Clear bit `index` to 0.
    #[inline]
    #[hotpath::measure]
    pub fn clear(&mut self, index: usize) {
        debug_assert!(index < NW * 64);
        self.words[index / 64] &= !(1u64 << (index % 64));
    }

    /// True if no bits are set.
    #[inline]
    #[hotpath::measure]
    pub fn is_empty(&self) -> bool {
        let mut i = 0;
        while i < NW {
            if self.words[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Population count — number of set bits.
    #[inline]
    #[hotpath::measure]
    pub fn count(&self) -> u32 {
        let mut total = 0u32;
        let mut i = 0;
        while i < NW {
            total += self.words[i].count_ones();
            i += 1;
        }
        total
    }

    /// Index of the lowest set bit, or `None` if empty.
    #[inline]
    #[hotpath::measure]
    pub fn lowest_bit_index(&self) -> Option<usize> {
        let mut i = 0;
        while i < NW {
            let w = self.words[i];
            if w != 0 {
                return Some(i * 64 + w.trailing_zeros() as usize);
            }
            i += 1;
        }
        None
    }

    /// Shift all bits left (toward higher indices) by `n` positions.
    /// Bits shifted beyond NW*64-1 are lost.
    #[inline]
    #[hotpath::measure]
    pub fn shift_left(&self, n: usize) -> Self {
        if n == 0 {
            return *self;
        }
        if n >= NW * 64 {
            return Self::empty();
        }
        let word_shift = n / 64;
        let bit_shift = n % 64;
        let mut out = [0u64; NW];

        if bit_shift == 0 {
            out[word_shift..NW].copy_from_slice(&self.words[..(NW - word_shift)]);
        } else {
            let mut i = word_shift;
            while i < NW {
                out[i] = self.words[i - word_shift] << bit_shift;
                if i > word_shift {
                    out[i] |= self.words[i - word_shift - 1] >> (64 - bit_shift);
                }
                i += 1;
            }
        }
        Bitboard { words: out }
    }

    /// Shift all bits right (toward lower indices) by `n` positions.
    /// Bits shifted below 0 are lost.
    #[inline]
    #[hotpath::measure]
    pub fn shift_right(&self, n: usize) -> Self {
        if n == 0 {
            return *self;
        }
        if n >= NW * 64 {
            return Self::empty();
        }
        let word_shift = n / 64;
        let bit_shift = n % 64;
        let mut out = [0u64; NW];

        if bit_shift == 0 {
            out[..(NW - word_shift)].copy_from_slice(&self.words[word_shift..]);
        } else {
            let mut i = 0;
            while i < NW - word_shift {
                out[i] = self.words[i + word_shift] >> bit_shift;
                if i + word_shift + 1 < NW {
                    out[i] |= self.words[i + word_shift + 1] << (64 - bit_shift);
                }
                i += 1;
            }
        }
        Bitboard { words: out }
    }

    /// `self & !rhs` — bits in self that are not in rhs.
    #[inline]
    #[hotpath::measure]
    pub fn andnot(self, rhs: Bitboard<NW>) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] & !rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }

    /// Iterate over indices of set bits.
    #[inline]
    #[hotpath::measure]
    pub fn iter_ones(&self) -> BitIterator<NW> {
        BitIterator {
            words: self.words,
            word_index: 0,
        }
    }
}

#[hotpath::measure_all]
impl<const NW: usize> BitAnd for Bitboard<NW> {
    type Output = Bitboard<NW>;
    #[inline]
    fn bitand(self, rhs: Bitboard<NW>) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] & rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }
}

#[hotpath::measure_all]
impl<const NW: usize> BitAndAssign for Bitboard<NW> {
    #[inline]
    fn bitand_assign(&mut self, rhs: Bitboard<NW>) {
        let mut i = 0;
        while i < NW {
            self.words[i] &= rhs.words[i];
            i += 1;
        }
    }
}

#[hotpath::measure_all]
impl<const NW: usize> BitOr for Bitboard<NW> {
    type Output = Bitboard<NW>;
    #[inline]
    fn bitor(self, rhs: Bitboard<NW>) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] | rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }
}

#[hotpath::measure_all]
impl<const NW: usize> BitOrAssign for Bitboard<NW> {
    #[inline]
    fn bitor_assign(&mut self, rhs: Bitboard<NW>) {
        let mut i = 0;
        while i < NW {
            self.words[i] |= rhs.words[i];
            i += 1;
        }
    }
}

#[hotpath::measure_all]
impl<const NW: usize> Not for Bitboard<NW> {
    type Output = Bitboard<NW>;
    #[inline]
    fn not(self) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = !self.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }
}

/// Iterator over set-bit indices in a `Bitboard`.
pub struct BitIterator<const NW: usize> {
    words: [u64; NW],
    word_index: u8,
}

#[hotpath::measure_all]
impl<const NW: usize> Iterator for BitIterator<NW> {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<usize> {
        while (self.word_index as usize) < NW {
            let wi = self.word_index as usize;
            let w = self.words[wi];
            if w != 0 {
                let bit = w.trailing_zeros() as usize;
                // Clear lowest set bit
                self.words[wi] = w & (w - 1);
                return Some(wi * 64 + bit);
            }
            self.word_index += 1;
        }
        None
    }
}

/// A single directional step for ray-based sliding move generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirStep<const NW: usize> {
    pub shift: usize,
    pub left: bool,         // true = shift_left, false = shift_right
    pub mask: Bitboard<NW>, // column mask to prevent wrapping (ANDed after each step)
}

impl<const NW: usize> DirStep<NW> {
    #[inline]
    pub fn step(&self, bb: Bitboard<NW>) -> Bitboard<NW> {
        if self.left {
            bb.shift_left(self.shift) & self.mask
        } else {
            bb.shift_right(self.shift) & self.mask
        }
    }
}

/// Precomputed masks for a given board geometry. Created once per Game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardGeometry<const NW: usize> {
    pub width: u8,
    pub height: u8,
    pub area: u16,
    /// Mask with 1s at all valid board positions (indices 0..area).
    pub board_mask: Bitboard<NW>,
    /// board_mask minus column 0.
    pub not_col_first: Bitboard<NW>,
    /// board_mask minus last column.
    pub not_col_last: Bitboard<NW>,
    /// board_mask minus columns 0 and 1.
    pub not_col_first_2: Bitboard<NW>,
    /// board_mask minus last two columns.
    pub not_col_last_2: Bitboard<NW>,
    /// Orthogonal ray steps: N, S, E, W.
    pub orthogonal_steps: [DirStep<NW>; 4],
    /// Diagonal ray steps: NE, NW, SE, SW.
    pub diagonal_steps: [DirStep<NW>; 4],
}

#[hotpath::measure_all]
impl<const NW: usize> BoardGeometry<NW> {
    /// Build geometry for a `width × height` board.
    pub fn new(width: u8, height: u8) -> Self {
        debug_assert!((1..=32).contains(&width));
        debug_assert!((1..=32).contains(&height));

        let area = width as u16 * height as u16;

        assert!(
            NW == (area as usize).div_ceil(64),
            "NW={} does not match board {}x{} (need {})",
            NW,
            width,
            height,
            (area as usize).div_ceil(64)
        );

        let w = width as usize;
        let h = height as usize;

        let mut board_mask = Bitboard::empty();
        for i in 0..area as usize {
            board_mask.set(i);
        }

        let mut not_col_first = board_mask;
        for row in 0..h {
            not_col_first.clear(row * w); // column 0
        }

        let mut not_col_last = board_mask;
        for row in 0..h {
            not_col_last.clear(row * w + w - 1); // last column
        }

        let mut not_col_first_2 = not_col_first;
        if w >= 2 {
            for row in 0..h {
                not_col_first_2.clear(row * w + 1); // column 1
            }
        }

        let mut not_col_last_2 = not_col_last;
        if w >= 2 {
            for row in 0..h {
                not_col_last_2.clear(row * w + w - 2); // second-to-last column
            }
        }

        // Orthogonal steps: N, S, E, W
        let orthogonal_steps = [
            DirStep {
                shift: w,
                left: true,
                mask: board_mask,
            }, // N (+row)
            DirStep {
                shift: w,
                left: false,
                mask: board_mask,
            }, // S (-row)
            DirStep {
                shift: 1,
                left: true,
                mask: not_col_first,
            }, // E (+col)
            DirStep {
                shift: 1,
                left: false,
                mask: not_col_last,
            }, // W (-col)
        ];

        // Diagonal steps: NE, NW, SE, SW
        let diagonal_steps = [
            DirStep {
                shift: w + 1,
                left: true,
                mask: not_col_first,
            }, // NE
            DirStep {
                shift: w - 1,
                left: true,
                mask: not_col_last,
            }, // NW
            DirStep {
                shift: w - 1,
                left: false,
                mask: not_col_first,
            }, // SE
            DirStep {
                shift: w + 1,
                left: false,
                mask: not_col_last,
            }, // SW
        ];

        BoardGeometry {
            width,
            height,
            area,
            board_mask,
            not_col_first,
            not_col_last,
            not_col_first_2,
            not_col_last_2,
            orthogonal_steps,
            diagonal_steps,
        }
    }

    /// Cast a ray from `src` in direction `dir`, stopping at blockers in `occupied`.
    /// The ray includes blocker squares (for captures) but does not pass through them.
    #[inline]
    pub fn ray_attacks(
        &self,
        src: Bitboard<NW>,
        dir: &DirStep<NW>,
        occupied: Bitboard<NW>,
    ) -> Bitboard<NW> {
        let mut attacks = Bitboard::empty();
        let mut cursor = dir.step(src);
        let max_steps = self.width.max(self.height) as usize;
        for _ in 0..max_steps {
            if cursor.is_empty() {
                break;
            }
            attacks |= cursor;
            cursor = dir.step(cursor.andnot(occupied));
        }
        attacks
    }

    /// Compute the set of all orthogonal neighbors of every bit in `bb`.
    #[inline]
    pub fn neighbors(&self, bb: &Bitboard<NW>) -> Bitboard<NW> {
        let w = self.width as usize;

        // right: col+1 = shift left by 1. A bit at col=w-1 wraps to col=0 of next row,
        // so mask off col-0 positions in the result.
        let right = bb.shift_left(1) & self.not_col_first;
        // left: col-1 = shift right by 1. A bit at col=0 wraps to col=w-1 of previous row,
        // so mask off last-column positions in the result.
        let left = bb.shift_right(1) & self.not_col_last;
        // down: row+1 = shift left by width
        let down = bb.shift_left(w);
        // up: row-1 = shift right by width
        let up = bb.shift_right(w);

        (right | left | down | up) & self.board_mask
    }

    #[inline]
    pub fn knight_attacks(&self, src: Bitboard<NW>) -> Bitboard<NW> {
        let w = self.width as usize;

        // +1 col, +2 rows (shift left by 2w+1, mask not_col_last)
        let a = src.shift_left(2 * w + 1) & self.not_col_first;
        // +2 col, +1 row (shift left by w+2, mask not_col_last_2)
        let b = src.shift_left(w + 2) & self.not_col_first_2;
        // -1 col, +2 rows (shift left by 2w-1, mask not_col_first)
        let c = src.shift_left(2 * w - 1) & self.not_col_last;
        // -2 col, +1 row (shift left by w-2, mask not_col_first_2)
        let d = src.shift_left(w - 2) & self.not_col_last_2;

        // +1 col, -2 rows (shift right by 2w-1, mask not_col_last)
        let e = src.shift_right(2 * w - 1) & self.not_col_first;
        // +2 col, -1 row (shift right by w-2, mask not_col_last_2)
        let f = src.shift_right(w - 2) & self.not_col_first_2;
        // -1 col, -2 rows (shift right by 2w+1, mask not_col_first)
        let g = src.shift_right(2 * w + 1) & self.not_col_last;
        // -2 col, -1 row (shift right by w+2, mask not_col_first_2)
        let h = src.shift_right(w + 2) & self.not_col_last_2;

        (a | b | c | d | e | f | g | h) & self.board_mask
    }

    /// Single forward push for pawns. White = shift_left(width), Black = shift_right(width).
    /// Does NOT filter by occupancy — caller must `andnot(occupied)`.
    #[inline]
    pub fn pawn_push(&self, src: Bitboard<NW>, is_white: bool) -> Bitboard<NW> {
        let w = self.width as usize;
        if is_white {
            src.shift_left(w) & self.board_mask
        } else {
            src.shift_right(w) & self.board_mask
        }
    }

    /// Diagonal attack squares for pawns (both capture directions combined).
    #[inline]
    pub fn pawn_attacks(&self, src: Bitboard<NW>, is_white: bool) -> Bitboard<NW> {
        let w = self.width as usize;
        if is_white {
            let left = src.shift_left(w + 1) & self.not_col_first;
            let right = src.shift_left(w - 1) & self.not_col_last;
            (left | right) & self.board_mask
        } else {
            let left = src.shift_right(w - 1) & self.not_col_first;
            let right = src.shift_right(w + 1) & self.not_col_last;
            (left | right) & self.board_mask
        }
    }

    #[inline]
    pub fn king_attacks(&self, src: Bitboard<NW>) -> Bitboard<NW> {
        let w = self.width as usize;
        let n = src.shift_left(w);
        let s = src.shift_right(w);
        let e = src.shift_left(1) & self.not_col_first;
        let w_ = src.shift_right(1) & self.not_col_last;
        let ne = src.shift_left(w + 1) & self.not_col_first;
        let nw = src.shift_left(w - 1) & self.not_col_last;
        let se = src.shift_right(w - 1) & self.not_col_first;
        let sw = src.shift_right(w + 1) & self.not_col_last;

        (n | s | e | w_ | ne | nw | se | sw) & self.board_mask
    }

    /// Flood-fill from `seed` through `mask`. Returns the connected component
    /// of `seed` within `mask`.
    #[inline]
    pub fn flood_fill(&self, seed: Bitboard<NW>, mask: Bitboard<NW>) -> Bitboard<NW> {
        let mut filled = seed & mask;
        loop {
            let nbrs = self.neighbors(&filled);
            let expanded = (filled | nbrs) & mask;
            if expanded == filled {
                return filled;
            }
            filled = expanded;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let bb = Bitboard::<2>::empty();
        assert!(bb.is_empty());
        assert_eq!(bb.count(), 0);
        assert!(bb.lowest_bit_index().is_none());
    }

    #[test]
    fn test_single() {
        let bb = Bitboard::<16>::single(0);
        assert!(bb.get(0));
        assert!(!bb.get(1));
        assert_eq!(bb.count(), 1);
        assert_eq!(bb.lowest_bit_index(), Some(0));

        let bb2 = Bitboard::<16>::single(63);
        assert!(bb2.get(63));
        assert!(!bb2.get(62));
        assert!(!bb2.get(64));

        let bb3 = Bitboard::<16>::single(64);
        assert!(bb3.get(64));
        assert!(!bb3.get(63));

        let bb4 = Bitboard::<16>::single(1023);
        assert!(bb4.get(1023));
        assert_eq!(bb4.count(), 1);
    }

    #[test]
    fn test_set_clear() {
        let mut bb = Bitboard::<2>::empty();
        bb.set(100);
        assert!(bb.get(100));
        assert_eq!(bb.count(), 1);
        bb.clear(100);
        assert!(!bb.get(100));
        assert!(bb.is_empty());
    }

    #[test]
    fn test_bitwise_ops() {
        let a = Bitboard::<1>::single(5) | Bitboard::<1>::single(10);
        let b = Bitboard::<1>::single(10) | Bitboard::<1>::single(20);

        let and = a & b;
        assert!(and.get(10));
        assert!(!and.get(5));
        assert!(!and.get(20));

        let or = a | b;
        assert!(or.get(5));
        assert!(or.get(10));
        assert!(or.get(20));
    }

    #[test]
    fn test_shift_left() {
        let bb = Bitboard::<16>::single(0);
        let shifted = bb.shift_left(1);
        assert!(shifted.get(1));
        assert!(!shifted.get(0));

        // Cross word boundary: 63 -> 64
        let bb2 = Bitboard::<16>::single(63);
        let shifted2 = bb2.shift_left(1);
        assert!(shifted2.get(64));
        assert!(!shifted2.get(63));

        // Cross word boundary: 127 -> 128
        let bb3 = Bitboard::<16>::single(127);
        let shifted3 = bb3.shift_left(1);
        assert!(shifted3.get(128));
        assert!(!shifted3.get(127));
    }

    #[test]
    fn test_shift_right() {
        let bb = Bitboard::<16>::single(1);
        let shifted = bb.shift_right(1);
        assert!(shifted.get(0));
        assert!(!shifted.get(1));

        // Cross word boundary: 64 -> 63
        let bb2 = Bitboard::<16>::single(64);
        let shifted2 = bb2.shift_right(1);
        assert!(shifted2.get(63));
        assert!(!shifted2.get(64));

        // Shift from 0 -> lost
        let bb3 = Bitboard::<16>::single(0);
        let shifted3 = bb3.shift_right(1);
        assert!(shifted3.is_empty());
    }

    #[test]
    fn test_shift_by_width() {
        // Simulate shift by width=9 (row shift on 9x9 board)
        let bb = Bitboard::<2>::single(4); // col=4, row=0
        let shifted = bb.shift_left(9);
        assert!(shifted.get(13)); // col=4, row=1
        assert!(!shifted.get(4));
    }

    #[test]
    fn test_iter_ones() {
        let bb = Bitboard::<4>::single(3) | Bitboard::<4>::single(64) | Bitboard::<4>::single(200);
        let indices: Vec<usize> = bb.iter_ones().collect();
        assert_eq!(indices, vec![3, 64, 200]);
    }

    #[test]
    fn test_iter_ones_empty() {
        let bb = Bitboard::<2>::empty();
        let indices: Vec<usize> = bb.iter_ones().collect();
        assert!(indices.is_empty());
    }

    #[test]
    fn test_geometry_9x9() {
        let geo = BoardGeometry::<{ nw_for_board(9, 9) }>::new(9, 9);
        assert_eq!(geo.area, 81u16);
        assert_eq!(geo.board_mask.count(), 81);

        // Column 0 positions: 0, 9, 18, 27, ...
        for row in 0..9 {
            assert!(!geo.not_col_first.get(row * 9));
            assert!(geo.not_col_first.get(row * 9 + 1));
        }

        // Last column positions: 8, 17, 26, ...
        for row in 0..9 {
            assert!(!geo.not_col_last.get(row * 9 + 8));
            assert!(geo.not_col_last.get(row * 9 + 7));
        }
    }

    #[test]
    fn test_neighbors_center() {
        let geo = BoardGeometry::<{ nw_for_board(9, 9) }>::new(9, 9);
        // Center of 9x9: col=4, row=4 -> index = 4*9+4 = 40
        let center = Bitboard::single(40);
        let nbrs = geo.neighbors(&center);

        // Expected: right=41, left=39, up=31, down=49
        assert!(nbrs.get(41));
        assert!(nbrs.get(39));
        assert!(nbrs.get(31));
        assert!(nbrs.get(49));
        assert_eq!(nbrs.count(), 4);
    }

    #[test]
    fn test_neighbors_corner() {
        let geo = BoardGeometry::<{ nw_for_board(9, 9) }>::new(9, 9);
        // Top-left corner: col=0, row=0 -> index = 0
        let corner = Bitboard::single(0);
        let nbrs = geo.neighbors(&corner);

        // Expected: right=1, down=9 (no left, no up)
        assert!(nbrs.get(1));
        assert!(nbrs.get(9));
        assert_eq!(nbrs.count(), 2);
    }

    #[test]
    fn test_neighbors_no_wrap() {
        let geo = BoardGeometry::<{ nw_for_board(9, 9) }>::new(9, 9);
        // Right edge: col=8, row=1 -> index = 1*9+8 = 17
        let edge = Bitboard::single(17);
        let nbrs = geo.neighbors(&edge);

        // Expected: left=16, up=8, down=26 (no right — must not wrap to col=0 of next row)
        assert!(nbrs.get(16)); // left
        assert!(nbrs.get(8)); // up
        assert!(nbrs.get(26)); // down
        assert!(!nbrs.get(18)); // must NOT wrap
        assert_eq!(nbrs.count(), 3);
    }

    #[test]
    fn test_neighbors_left_edge() {
        let geo = BoardGeometry::<{ nw_for_board(9, 9) }>::new(9, 9);
        // Left edge: col=0, row=2 -> index = 2*9+0 = 18
        let edge = Bitboard::single(18);
        let nbrs = geo.neighbors(&edge);

        // Expected: right=19, up=9, down=27 (no left — must not wrap to col=8 of previous row)
        assert!(nbrs.get(19)); // right
        assert!(nbrs.get(9)); // up
        assert!(nbrs.get(27)); // down
        assert!(!nbrs.get(17)); // must NOT wrap
        assert_eq!(nbrs.count(), 3);
    }

    #[test]
    fn test_flood_fill_single() {
        let geo = BoardGeometry::<{ nw_for_board(5, 5) }>::new(5, 5);
        let seed = Bitboard::single(0);
        let mask = seed;
        let result = geo.flood_fill(seed, mask);
        assert_eq!(result, seed);
    }

    #[test]
    fn test_flood_fill_group() {
        let geo = BoardGeometry::<{ nw_for_board(5, 5) }>::new(5, 5);
        // Create a group: (0,0), (1,0), (2,0) -> indices 0, 1, 2
        let mask = Bitboard::single(0) | Bitboard::single(1) | Bitboard::single(2);
        let seed = Bitboard::single(0);
        let result = geo.flood_fill(seed, mask);
        assert_eq!(result, mask);
    }

    #[test]
    fn test_flood_fill_disconnected() {
        let geo = BoardGeometry::<{ nw_for_board(5, 5) }>::new(5, 5);
        // Two disconnected stones: (0,0) and (3,3) -> indices 0 and 18
        let mask = Bitboard::single(0) | Bitboard::single(18);
        let seed = Bitboard::single(0);
        let result = geo.flood_fill(seed, mask);
        // Should only reach the seed's connected component
        assert!(result.get(0));
        assert!(!result.get(18));
        assert_eq!(result.count(), 1);
    }

    #[test]
    fn test_not() {
        let bb = Bitboard::<1>::single(5);
        let notbb = !bb;
        assert!(!notbb.get(5));
        assert!(notbb.get(0));
        assert!(notbb.get(6));
    }

    #[test]
    fn test_andnot() {
        let a = Bitboard::<1>::single(0) | Bitboard::single(5) | Bitboard::single(10);
        let b = Bitboard::<1>::single(5) | Bitboard::single(20);
        let result = a.andnot(b);
        assert!(result.get(0));
        assert!(!result.get(5));
        assert!(result.get(10));
        assert!(!result.get(20));
    }

    #[test]
    fn test_non_square_board() {
        let geo = BoardGeometry::<{ nw_for_board(5, 3) }>::new(5, 3);
        assert_eq!(geo.area, 15);
        assert_eq!(geo.board_mask.count(), 15);

        // Corner (4, 2) -> index = 2*5+4 = 14
        let corner = Bitboard::single(14);
        let nbrs = geo.neighbors(&corner);
        // Expected: left=13, up=9
        assert!(nbrs.get(13));
        assert!(nbrs.get(9));
        assert_eq!(nbrs.count(), 2);
    }

    #[test]
    fn test_assign_ops() {
        let mut bb = Bitboard::<1>::single(1);
        bb |= Bitboard::single(2);
        assert!(bb.get(1));
        assert!(bb.get(2));

        bb &= Bitboard::single(2);
        assert!(!bb.get(1));
        assert!(bb.get(2));
    }

    #[test]
    fn test_neighbors_all_boards() {
        check_all_neighbors::<{ nw_for_board(5, 5) }>(5, 5);
        check_all_neighbors::<{ nw_for_board(8, 8) }>(8, 8);
        check_all_neighbors::<{ nw_for_board(9, 9) }>(9, 9);
        check_all_neighbors::<{ nw_for_board(19, 19) }>(19, 19);
    }

    fn check_all_neighbors<const NW: usize>(w: u8, h: u8) {
        let geo = BoardGeometry::<NW>::new(w, h);
        let area = geo.area as usize;
        let w = w as usize;
        let h = h as usize;
        for idx in 0..area {
            let bb = Bitboard::single(idx);
            let nbrs = geo.neighbors(&bb);
            // Verify result is within board
            assert_eq!(
                nbrs & geo.board_mask,
                nbrs,
                "neighbors outside board at {}x{} idx={}",
                w,
                h,
                idx
            );
            // Verify correct neighbor count
            let col = idx % w;
            let row = idx / w;
            let mut expected = 0u32;
            if col > 0 {
                expected += 1;
            }
            if col + 1 < w {
                expected += 1;
            }
            if row > 0 {
                expected += 1;
            }
            if row + 1 < h {
                expected += 1;
            }
            assert_eq!(
                nbrs.count(),
                expected,
                "wrong neighbor count at {}x{} col={} row={}",
                w,
                h,
                col,
                row
            );
        }
    }

    #[test]
    fn test_nw_values() {
        assert_eq!(nw_for_board(2, 2), 1); // 4 bits
        assert_eq!(nw_for_board(5, 5), 1); // 25 bits
        assert_eq!(nw_for_board(8, 8), 1); // 64 bits
        assert_eq!(nw_for_board(9, 9), 2); // 81 bits
        assert_eq!(nw_for_board(19, 19), 6); // 361 bits
        assert_eq!(nw_for_board(32, 32), 16); // 1024 bits
    }

    #[test]
    fn test_8x8_word_boundary() {
        // 8x8 = 64 bits = exactly 1 word. shift_left(1) of bit 63 spills beyond.
        let geo = BoardGeometry::<{ nw_for_board(8, 8) }>::new(8, 8);

        // bit 63 = col 7, row 7 (bottom-right corner of 8x8)
        let corner = Bitboard::single(63);
        let nbrs = geo.neighbors(&corner);
        // col 7, row 7: left=62, up=55. No right (col 8 invalid), no down (row 8 invalid)
        assert!(nbrs.get(62));
        assert!(nbrs.get(55));
        assert_eq!(nbrs.count(), 2);
    }
}
