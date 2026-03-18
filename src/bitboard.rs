use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

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
    pub const fn single(index: usize) -> Self {
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
    pub const fn get(&self, index: usize) -> bool {
        debug_assert!(index < NW * 64);
        (self.words[index / 64] >> (index % 64)) & 1 != 0
    }

    /// Return the bit at `index` as a `u64` (0 or 1). Branchless.
    #[inline]
    pub const fn bit_at(&self, index: usize) -> u64 {
        debug_assert!(index < NW * 64);
        (self.words[index / 64] >> (index % 64)) & 1
    }

    /// Set bit `index` to 1.
    #[inline]
    pub const fn set(&mut self, index: usize) {
        debug_assert!(index < NW * 64);
        self.words[index / 64] |= 1u64 << (index % 64);
    }

    /// Clear bit `index` to 0.
    #[inline]
    pub const fn clear(&mut self, index: usize) {
        debug_assert!(index < NW * 64);
        self.words[index / 64] &= !(1u64 << (index % 64));
    }

    /// True if no bits are set.
    #[inline]
    pub const fn is_empty(&self) -> bool {
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
    pub const fn count(&self) -> u32 {
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

    /// Index of the highest set bit, or `None` if empty.
    #[inline]
    #[hotpath::measure]
    pub fn highest_bit_index(&self) -> Option<usize> {
        let mut i = NW;
        while i > 0 {
            i -= 1;
            if self.words[i] != 0 {
                return Some(i * 64 + (63 - self.words[i].leading_zeros() as usize));
            }
        }
        None
    }

    /// Shift all bits left (toward higher indices) by `n` positions.
    /// Bits shifted beyond NW*64-1 are lost.
    #[inline]
    pub const fn shift_left(&self, n: usize) -> Self {
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
            let mut i = word_shift;
            while i < NW {
                out[i] = self.words[i - word_shift];
                i += 1;
            }
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
    pub const fn shift_right(&self, n: usize) -> Self {
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
            let mut i = 0;
            while i < NW - word_shift {
                out[i] = self.words[i + word_shift];
                i += 1;
            }
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
    pub const fn andnot(self, rhs: Bitboard<NW>) -> Bitboard<NW> {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] & !rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }

    /// Const bitwise AND.
    #[inline]
    pub const fn c_and(self, rhs: Self) -> Self {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] & rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }

    /// Const bitwise OR.
    #[inline]
    pub const fn c_or(self, rhs: Self) -> Self {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] | rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }

    /// Const bitwise XOR.
    #[inline]
    pub const fn c_xor(self, rhs: Self) -> Self {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = self.words[i] ^ rhs.words[i];
            i += 1;
        }
        Bitboard { words: out }
    }

    /// Const bitwise NOT.
    #[inline]
    pub const fn c_not(self) -> Self {
        let mut out = [0u64; NW];
        let mut i = 0;
        while i < NW {
            out[i] = !self.words[i];
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
        self.c_and(rhs)
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
        self.c_or(rhs)
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
        self.c_not()
    }
}

#[hotpath::measure_all]
impl<const NW: usize> BitXor for Bitboard<NW> {
    type Output = Bitboard<NW>;
    #[inline]
    fn bitxor(self, rhs: Bitboard<NW>) -> Bitboard<NW> {
        self.c_xor(rhs)
    }
}

#[hotpath::measure_all]
impl<const NW: usize> BitXorAssign for Bitboard<NW> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Bitboard<NW>) {
        let mut i = 0;
        while i < NW {
            self.words[i] ^= rhs.words[i];
            i += 1;
        }
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

/// Precomputed masks and attack tables for a given board geometry.
/// Parameterized by board width W and height H.
/// Access via `BoardGeometry::<W, H>::INSTANCE`.
#[derive(Debug, PartialEq, Eq)]
pub struct BoardGeometry<const W: usize, const H: usize>
where
    [(); (W * H).div_ceil(64)]:,
{
    /// Mask with 1s at all valid board positions (indices 0..W*H).
    pub board_mask: Bitboard<{ (W * H).div_ceil(64) }>,
    /// board_mask minus column 0.
    pub not_col_first: Bitboard<{ (W * H).div_ceil(64) }>,
    /// board_mask minus last column.
    pub not_col_last: Bitboard<{ (W * H).div_ceil(64) }>,
    /// board_mask minus columns 0 and 1.
    pub not_col_first_2: Bitboard<{ (W * H).div_ceil(64) }>,
    /// board_mask minus last two columns.
    pub not_col_last_2: Bitboard<{ (W * H).div_ceil(64) }>,
    /// Orthogonal ray steps: N, S, E, W.
    pub orthogonal_steps: [DirStep<{ (W * H).div_ceil(64) }>; 4],
    /// Diagonal ray steps: NE, NW, SE, SW.
    pub diagonal_steps: [DirStep<{ (W * H).div_ceil(64) }>; 4],
    /// Precomputed attack tables indexed by square index.
    king_attacks_table: [Bitboard<{ (W * H).div_ceil(64) }>; W * H],
    knight_attacks_table: [Bitboard<{ (W * H).div_ceil(64) }>; W * H],
    pawn_attacks_white_table: [Bitboard<{ (W * H).div_ceil(64) }>; W * H],
    pawn_attacks_black_table: [Bitboard<{ (W * H).div_ceil(64) }>; W * H],
    /// Precomputed full unblocked rays for orthogonal directions (N, S, E, W).
    pub(crate) ray_orthogonal: [[Bitboard<{ (W * H).div_ceil(64) }>; W * H]; 4],
    /// Precomputed full unblocked rays for diagonal directions (NE, NW, SE, SW).
    pub(crate) ray_diagonal: [[Bitboard<{ (W * H).div_ceil(64) }>; W * H]; 4],
}

impl<const W: usize, const H: usize> Default for BoardGeometry<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const W: usize, const H: usize> BoardGeometry<W, H>
where
    [(); (W * H).div_ceil(64)]:,
{
    pub const INSTANCE: Self = Self::new();

    pub const fn width() -> usize {
        W
    }

    pub const fn height() -> usize {
        H
    }

    pub const fn area() -> usize {
        W * H
    }

    /// Build geometry for a `W × H` board at compile time.
    pub const fn new() -> Self {
        type Bb<const N: usize> = Bitboard<N>;

        let area = W * H;

        let mut board_mask: Bb<{ (W * H).div_ceil(64) }> = Bb::empty();
        let mut i = 0;
        while i < area {
            board_mask.set(i);
            i += 1;
        }

        let mut not_col_first = board_mask;
        {
            let mut row = 0;
            while row < H {
                not_col_first.clear(row * W); // column 0
                row += 1;
            }
        }

        let mut not_col_last = board_mask;
        {
            let mut row = 0;
            while row < H {
                not_col_last.clear(row * W + W - 1); // last column
                row += 1;
            }
        }

        let mut not_col_first_2 = not_col_first;
        if W >= 2 {
            let mut row = 0;
            while row < H {
                not_col_first_2.clear(row * W + 1); // column 1
                row += 1;
            }
        }

        let mut not_col_last_2 = not_col_last;
        if W >= 2 {
            let mut row = 0;
            while row < H {
                not_col_last_2.clear(row * W + W - 2); // second-to-last column
                row += 1;
            }
        }

        // Orthogonal steps: N, S, E, W
        let orthogonal_steps = [
            DirStep {
                shift: W,
                left: true,
                mask: board_mask,
            }, // N (+row)
            DirStep {
                shift: W,
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
                shift: W + 1,
                left: true,
                mask: not_col_first,
            }, // NE
            DirStep {
                shift: W - 1,
                left: true,
                mask: not_col_last,
            }, // NW
            DirStep {
                shift: W - 1,
                left: false,
                mask: not_col_first,
            }, // SE
            DirStep {
                shift: W + 1,
                left: false,
                mask: not_col_last,
            }, // SW
        ];

        // Compute attack tables
        let mut king_table: [Bb<{ (W * H).div_ceil(64) }>; W * H] = [Bb::empty(); W * H];
        let mut knight_table: [Bb<{ (W * H).div_ceil(64) }>; W * H] = [Bb::empty(); W * H];
        let mut pawn_w_table: [Bb<{ (W * H).div_ceil(64) }>; W * H] = [Bb::empty(); W * H];
        let mut pawn_b_table: [Bb<{ (W * H).div_ceil(64) }>; W * H] = [Bb::empty(); W * H];

        let mut ray_ortho: [[Bb<{ (W * H).div_ceil(64) }>; W * H]; 4] = [[Bb::empty(); W * H]; 4];
        let mut ray_diag: [[Bb<{ (W * H).div_ceil(64) }>; W * H]; 4] = [[Bb::empty(); W * H]; 4];

        let mut idx = 0;
        while idx < area {
            let sq = Bb::single(idx);
            king_table[idx] =
                Self::compute_king_attacks_const(sq, board_mask, not_col_first, not_col_last);
            knight_table[idx] = Self::compute_knight_attacks_const(
                sq,
                board_mask,
                not_col_first,
                not_col_last,
                not_col_first_2,
                not_col_last_2,
            );
            pawn_w_table[idx] =
                Self::compute_pawn_attacks_const(sq, true, board_mask, not_col_first, not_col_last);
            pawn_b_table[idx] = Self::compute_pawn_attacks_const(
                sq,
                false,
                board_mask,
                not_col_first,
                not_col_last,
            );

            // Ray tables for orthogonal directions
            let mut d = 0;
            while d < 4 {
                ray_ortho[d][idx] = Self::compute_ray_const(
                    idx,
                    orthogonal_steps[d].shift,
                    orthogonal_steps[d].left,
                    orthogonal_steps[d].mask,
                    board_mask,
                );
                d += 1;
            }
            // Ray tables for diagonal directions
            d = 0;
            while d < 4 {
                ray_diag[d][idx] = Self::compute_ray_const(
                    idx,
                    diagonal_steps[d].shift,
                    diagonal_steps[d].left,
                    diagonal_steps[d].mask,
                    board_mask,
                );
                d += 1;
            }

            idx += 1;
        }

        BoardGeometry {
            board_mask,
            not_col_first,
            not_col_last,
            not_col_first_2,
            not_col_last_2,
            orthogonal_steps,
            diagonal_steps,
            king_attacks_table: king_table,
            knight_attacks_table: knight_table,
            pawn_attacks_white_table: pawn_w_table,
            pawn_attacks_black_table: pawn_b_table,
            ray_orthogonal: ray_ortho,
            ray_diagonal: ray_diag,
        }
    }

    const fn compute_king_attacks_const(
        src: Bitboard<{ (W * H).div_ceil(64) }>,
        board_mask: Bitboard<{ (W * H).div_ceil(64) }>,
        not_col_first: Bitboard<{ (W * H).div_ceil(64) }>,
        not_col_last: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        let n = src.shift_left(W);
        let s = src.shift_right(W);
        let e = src.shift_left(1).c_and(not_col_first);
        let w_ = src.shift_right(1).c_and(not_col_last);
        let ne = src.shift_left(W + 1).c_and(not_col_first);
        let nw = src.shift_left(W - 1).c_and(not_col_last);
        let se = src.shift_right(W - 1).c_and(not_col_first);
        let sw = src.shift_right(W + 1).c_and(not_col_last);

        n.c_or(s)
            .c_or(e)
            .c_or(w_)
            .c_or(ne)
            .c_or(nw)
            .c_or(se)
            .c_or(sw)
            .c_and(board_mask)
    }

    const fn compute_knight_attacks_const(
        src: Bitboard<{ (W * H).div_ceil(64) }>,
        board_mask: Bitboard<{ (W * H).div_ceil(64) }>,
        not_col_first: Bitboard<{ (W * H).div_ceil(64) }>,
        not_col_last: Bitboard<{ (W * H).div_ceil(64) }>,
        not_col_first_2: Bitboard<{ (W * H).div_ceil(64) }>,
        not_col_last_2: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        let a = src.shift_left(2 * W + 1).c_and(not_col_first);
        let b = src.shift_left(W + 2).c_and(not_col_first_2);
        let c = src.shift_left(2 * W - 1).c_and(not_col_last);
        let d = src.shift_left(W - 2).c_and(not_col_last_2);

        let e = src.shift_right(2 * W - 1).c_and(not_col_first);
        let f = src.shift_right(W - 2).c_and(not_col_first_2);
        let g = src.shift_right(2 * W + 1).c_and(not_col_last);
        let h = src.shift_right(W + 2).c_and(not_col_last_2);

        a.c_or(b)
            .c_or(c)
            .c_or(d)
            .c_or(e)
            .c_or(f)
            .c_or(g)
            .c_or(h)
            .c_and(board_mask)
    }

    const fn compute_pawn_attacks_const(
        src: Bitboard<{ (W * H).div_ceil(64) }>,
        is_white: bool,
        board_mask: Bitboard<{ (W * H).div_ceil(64) }>,
        not_col_first: Bitboard<{ (W * H).div_ceil(64) }>,
        not_col_last: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        if is_white {
            let left = src.shift_left(W + 1).c_and(not_col_first);
            let right = src.shift_left(W - 1).c_and(not_col_last);
            left.c_or(right).c_and(board_mask)
        } else {
            let left = src.shift_right(W - 1).c_and(not_col_first);
            let right = src.shift_right(W + 1).c_and(not_col_last);
            left.c_or(right).c_and(board_mask)
        }
    }

    /// Compute the full unblocked ray from a square in a given direction.
    const fn compute_ray_const(
        sq_idx: usize,
        shift: usize,
        left: bool,
        mask: Bitboard<{ (W * H).div_ceil(64) }>,
        board_mask: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        let mut ray = Bitboard::empty();
        let mut cursor = Bitboard::single(sq_idx);
        let max_steps = if W > H { W } else { H };
        let mut step = 0;
        while step < max_steps {
            cursor = if left {
                cursor.shift_left(shift).c_and(mask).c_and(board_mask)
            } else {
                cursor.shift_right(shift).c_and(mask).c_and(board_mask)
            };
            if cursor.is_empty() {
                break;
            }
            ray = ray.c_or(cursor);
            step += 1;
        }
        ray
    }

    /// Compute sliding attacks for a single ray direction using the ray difference trick.
    /// `is_left` = true for rays toward higher indices (use LSB for first blocker),
    /// false for rays toward lower indices (use MSB).
    #[inline]
    fn sliding_ray_attacks(
        sq_idx: usize,
        dir_idx: usize,
        ray_table: &[[Bitboard<{ (W * H).div_ceil(64) }>; W * H]; 4],
        is_left: bool,
        occupied: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        let full_ray = ray_table[dir_idx][sq_idx];
        let blockers = full_ray & occupied;
        if blockers.is_empty() {
            return full_ray;
        }
        let first_blocker = if is_left {
            blockers.lowest_bit_index().expect("blockers confirmed non-empty")
        } else {
            blockers.highest_bit_index().expect("blockers confirmed non-empty")
        };
        full_ray ^ ray_table[dir_idx][first_blocker]
    }

    /// Compute all orthogonal sliding attacks (N, S, E, W) from a square.
    #[inline]
    pub fn orthogonal_attacks(
        &self,
        sq_idx: usize,
        occupied: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        // N=left, S=right, E=left, W=right
        Self::sliding_ray_attacks(sq_idx, 0, &self.ray_orthogonal, true, occupied)
            | Self::sliding_ray_attacks(sq_idx, 1, &self.ray_orthogonal, false, occupied)
            | Self::sliding_ray_attacks(sq_idx, 2, &self.ray_orthogonal, true, occupied)
            | Self::sliding_ray_attacks(sq_idx, 3, &self.ray_orthogonal, false, occupied)
    }

    /// Compute all diagonal sliding attacks (NE, NW, SE, SW) from a square.
    #[inline]
    pub fn diagonal_attacks(
        &self,
        sq_idx: usize,
        occupied: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        // NE=left, NW=left, SE=right, SW=right
        Self::sliding_ray_attacks(sq_idx, 0, &self.ray_diagonal, true, occupied)
            | Self::sliding_ray_attacks(sq_idx, 1, &self.ray_diagonal, true, occupied)
            | Self::sliding_ray_attacks(sq_idx, 2, &self.ray_diagonal, false, occupied)
            | Self::sliding_ray_attacks(sq_idx, 3, &self.ray_diagonal, false, occupied)
    }

    /// Compute the set of all orthogonal neighbors of every bit in `bb`.
    #[inline]
    pub fn neighbors(
        &self,
        bb: &Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        let right = bb.shift_left(1) & self.not_col_first;
        let left = bb.shift_right(1) & self.not_col_last;
        let down = bb.shift_left(W);
        let up = bb.shift_right(W);

        (right | left | down | up) & self.board_mask
    }

    #[inline]
    pub fn knight_attacks(&self, sq_index: usize) -> Bitboard<{ (W * H).div_ceil(64) }> {
        debug_assert!(
            sq_index < W * H,
            "knight_attacks: sq_index {} out of bounds for {}x{} board",
            sq_index,
            W,
            H,
        );
        self.knight_attacks_table[sq_index]
    }

    /// Single forward push for pawns. White = shift_left(width), Black = shift_right(width).
    /// Does NOT filter by occupancy — caller must `andnot(occupied)`.
    #[inline]
    pub fn pawn_push(
        &self,
        src: Bitboard<{ (W * H).div_ceil(64) }>,
        is_white: bool,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        if is_white {
            src.shift_left(W) & self.board_mask
        } else {
            src.shift_right(W) & self.board_mask
        }
    }

    /// Diagonal attack squares for pawns (both capture directions combined).
    #[inline]
    pub fn pawn_attacks(
        &self,
        sq_index: usize,
        is_white: bool,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        debug_assert!(
            sq_index < W * H,
            "pawn_attacks: sq_index {} out of bounds for {}x{} board",
            sq_index,
            W,
            H,
        );
        if is_white {
            self.pawn_attacks_white_table[sq_index]
        } else {
            self.pawn_attacks_black_table[sq_index]
        }
    }

    #[inline]
    pub fn king_attacks(&self, sq_index: usize) -> Bitboard<{ (W * H).div_ceil(64) }> {
        debug_assert!(
            sq_index < W * H,
            "king_attacks: sq_index {} out of bounds for {}x{} board",
            sq_index,
            W,
            H,
        );
        self.king_attacks_table[sq_index]
    }

    /// Flood-fill from `seed` through `mask`. Returns the connected component
    /// of `seed` within `mask`.
    #[inline]
    pub fn flood_fill(
        &self,
        seed: Bitboard<{ (W * H).div_ceil(64) }>,
        mask: Bitboard<{ (W * H).div_ceil(64) }>,
    ) -> Bitboard<{ (W * H).div_ceil(64) }> {
        debug_assert!(
            seed.andnot(mask).is_empty(),
            "flood_fill: seed has bits outside mask",
        );
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
        let geo = &BoardGeometry::<9, 9>::INSTANCE;
        assert_eq!(BoardGeometry::<9, 9>::area(), 81);
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
        let geo = &BoardGeometry::<9, 9>::INSTANCE;
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
        let geo = &BoardGeometry::<9, 9>::INSTANCE;
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
        let geo = &BoardGeometry::<9, 9>::INSTANCE;
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
        let geo = &BoardGeometry::<9, 9>::INSTANCE;
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
        let geo = &BoardGeometry::<5, 5>::INSTANCE;
        let seed = Bitboard::single(0);
        let mask = seed;
        let result = geo.flood_fill(seed, mask);
        assert_eq!(result, seed);
    }

    #[test]
    fn test_flood_fill_group() {
        let geo = &BoardGeometry::<5, 5>::INSTANCE;
        // Create a group: (0,0), (1,0), (2,0) -> indices 0, 1, 2
        let mask = Bitboard::single(0) | Bitboard::single(1) | Bitboard::single(2);
        let seed = Bitboard::single(0);
        let result = geo.flood_fill(seed, mask);
        assert_eq!(result, mask);
    }

    #[test]
    fn test_flood_fill_disconnected() {
        let geo = &BoardGeometry::<5, 5>::INSTANCE;
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
        let geo = &BoardGeometry::<5, 3>::INSTANCE;
        assert_eq!(BoardGeometry::<5, 3>::area(), 15);
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
        check_all_neighbors::<5, 5>();
        check_all_neighbors::<8, 8>();
        check_all_neighbors::<9, 9>();
        check_all_neighbors::<19, 19>();
    }

    fn check_all_neighbors<const W: usize, const H: usize>()
    where
        [(); (W * H).div_ceil(64)]:,
    {
        let geo = &BoardGeometry::<W, H>::INSTANCE;
        let area = W * H;
        for idx in 0..area {
            let bb = Bitboard::single(idx);
            let nbrs = geo.neighbors(&bb);
            // Verify result is within board
            assert_eq!(
                nbrs & geo.board_mask,
                nbrs,
                "neighbors outside board at {}x{} idx={}",
                W,
                H,
                idx
            );
            // Verify correct neighbor count
            let col = idx % W;
            let row = idx / W;
            let mut expected = 0u32;
            if col > 0 {
                expected += 1;
            }
            if col + 1 < W {
                expected += 1;
            }
            if row > 0 {
                expected += 1;
            }
            if row + 1 < H {
                expected += 1;
            }
            assert_eq!(
                nbrs.count(),
                expected,
                "wrong neighbor count at {}x{} col={} row={}",
                W,
                H,
                col,
                row
            );
        }
    }

    #[test]
    fn test_8x8_word_boundary() {
        // 8x8 = 64 bits = exactly 1 word. shift_left(1) of bit 63 spills beyond.
        let geo = &BoardGeometry::<8, 8>::INSTANCE;

        // bit 63 = col 7, row 7 (bottom-right corner of 8x8)
        let corner = Bitboard::single(63);
        let nbrs = geo.neighbors(&corner);
        // col 7, row 7: left=62, up=55. No right (col 8 invalid), no down (row 8 invalid)
        assert!(nbrs.get(62));
        assert!(nbrs.get(55));
        assert_eq!(nbrs.count(), 2);
    }
}
