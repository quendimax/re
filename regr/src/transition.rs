use crate::ops::{ContainOp, IntersectOp, MergeOp};
use crate::symbol::Epsilon;
use redt::{Legible, RangeU8, Step};
use std::fmt::Write;

/// Quantity of `u64` values in the `chunks` member for symbols' bits.
const SYM_BITMAP_LEN: usize = 4;

/// Entire quantity of `u64` values in the `chunks` member.
const BITMAP_LEN: usize = SYM_BITMAP_LEN + 1; // + 1 for Epsilon bit

/// Index of `u64`-item that contains Epsilon bit.
const EPSILON_CHUNK: usize = 4;

/// Transition is a struct that contains symbols that connect two nodes. The
/// symbols can be bytes and Epsilon.
///
/// # Implementation
///
/// Symbols are the corresponding bits in `chunks` bitmap from 4x`u64` values.
/// The 256-th bit is for Epsilon.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Transition {
    chunks: [u64; BITMAP_LEN],
}

impl Transition {
    pub const BITS: u32 = 256;

    /// Creates a new transition initialized with the given symbol bitmap.
    pub fn new(chunks: &[u64; SYM_BITMAP_LEN]) -> Self {
        Self {
            chunks: [chunks[0], chunks[1], chunks[2], chunks[3], 0],
        }
    }

    /// Creates a new transition initialized with Epsilon.
    pub fn epsilon() -> Self {
        Self {
            chunks: [0, 0, 0, 0, 1],
        }
    }

    /// Creates a new transition parsing the given byte array, and setting a bit
    /// corresponding to each byte value in the array.
    pub fn from_symbols(bytes: &[u8]) -> Self {
        let mut tr = Transition::default();
        for byte in bytes {
            tr.merge(*byte);
        }
        tr
    }

    /// Returns iterator over all symbols in this trasition instance in
    /// ascendent order.
    pub fn symbols(&self) -> SymbolIter<'_> {
        SymbolIter::new(&self.chunks)
    }

    /// Returns iterator over all symbol ranges in this trasition instance in
    /// ascendent order.
    pub fn ranges(&self) -> RangeIter<'_> {
        RangeIter::new(&self.chunks)
    }

    /// Merges the `other` boject into this transition.
    pub fn merge<T>(&mut self, other: T)
    where
        Self: MergeOp<T>,
    {
        MergeOp::merge(self, other);
    }

    pub fn intersects<T>(&self, other: T) -> bool
    where
        Self: IntersectOp<T>,
    {
        IntersectOp::intersects(self, other)
    }

    pub fn contains<T>(&self, other: T) -> bool
    where
        Self: ContainOp<T>,
    {
        ContainOp::contains(self, other)
    }
}

impl ContainOp<u8> for Transition {
    #[inline]
    fn contains(&self, symbol: u8) -> bool {
        IntersectOp::intersects(self, symbol)
    }
}

impl ContainOp<&u8> for Transition {
    #[inline]
    fn contains(&self, symbol: &u8) -> bool {
        Self::contains(self, *symbol)
    }
}

impl ContainOp<RangeU8> for Transition {
    #[inline]
    fn contains(&self, range: RangeU8) -> bool {
        let mut other_tr = Transition::default();
        other_tr.merge(range);
        ContainOp::contains(self, &other_tr)
    }
}

impl ContainOp<&RangeU8> for Transition {
    #[inline]
    fn contains(&self, range: &RangeU8) -> bool {
        Self::contains(self, *range)
    }
}

impl ContainOp<std::ops::RangeInclusive<u8>> for Transition {
    #[inline]
    fn contains(&self, range: std::ops::RangeInclusive<u8>) -> bool {
        let range = RangeU8::from(range);
        ContainOp::contains(self, range)
    }
}

impl ContainOp<&std::ops::RangeInclusive<u8>> for Transition {
    #[inline]
    fn contains(&self, range: &std::ops::RangeInclusive<u8>) -> bool {
        let range = RangeU8::new(*range.start(), *range.end());
        ContainOp::contains(self, range)
    }
}

impl ContainOp<&Transition> for Transition {
    fn contains(&self, other: &Transition) -> bool {
        self.chunks[0] & other.chunks[0] == other.chunks[0]
            && self.chunks[1] & other.chunks[1] == other.chunks[1]
            && self.chunks[2] & other.chunks[2] == other.chunks[2]
            && self.chunks[3] & other.chunks[3] == other.chunks[3]
            && self.chunks[4] & other.chunks[4] == other.chunks[4]
    }
}

impl ContainOp<Epsilon> for Transition {
    fn contains(&self, _: Epsilon) -> bool {
        self.chunks[EPSILON_CHUNK] & 1 == 1
    }
}

impl IntersectOp<u8> for Transition {
    #[inline]
    fn intersects(&self, symbol: u8) -> bool {
        self.chunks[symbol as usize >> 6] & (1 << (symbol & (u8::MAX >> 2))) != 0
    }
}

impl IntersectOp<&u8> for Transition {
    #[inline]
    fn intersects(&self, symbol: &u8) -> bool {
        Self::intersects(self, *symbol)
    }
}

impl IntersectOp<RangeU8> for Transition {
    #[inline]
    fn intersects(&self, range: RangeU8) -> bool {
        Self::intersects(self, &range)
    }
}

impl IntersectOp<&RangeU8> for Transition {
    fn intersects(&self, range: &RangeU8) -> bool {
        let mut other = Transition::default();
        other.merge(range);
        IntersectOp::intersects(self, &other)
    }
}

impl IntersectOp<std::ops::RangeInclusive<u8>> for Transition {
    fn intersects(&self, range: std::ops::RangeInclusive<u8>) -> bool {
        Self::intersects(self, &range)
    }
}

impl IntersectOp<&std::ops::RangeInclusive<u8>> for Transition {
    fn intersects(&self, range: &std::ops::RangeInclusive<u8>) -> bool {
        let range = RangeU8::new(*range.start(), *range.end());
        IntersectOp::intersects(self, range)
    }
}

impl IntersectOp<&Transition> for Transition {
    fn intersects(&self, other: &Transition) -> bool {
        self.chunks[0] & other.chunks[0] != 0
            || self.chunks[1] & other.chunks[1] != 0
            || self.chunks[2] & other.chunks[2] != 0
            || self.chunks[3] & other.chunks[3] != 0
            || self.chunks[4] & other.chunks[4] != 0
    }
}

impl MergeOp<u8> for Transition {
    /// Merges a symbol into this transition.
    #[inline]
    fn merge(&mut self, symbol: u8) {
        self.chunks[symbol as usize >> 6] |= 1 << (symbol & (u8::MAX >> 2));
    }
}

impl MergeOp<&u8> for Transition {
    #[inline]
    fn merge(&mut self, symbol: &u8) {
        Self::merge(self, *symbol)
    }
}

impl MergeOp<RangeU8> for Transition {
    fn merge(&mut self, range: RangeU8) {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        unsafe {
            match ms_index - ls_index {
                0 => {
                    *self.chunks.get_unchecked_mut(ls_index) |= ls_mask & ms_mask;
                }
                1 => {
                    *self.chunks.get_unchecked_mut(ls_index) |= ls_mask;
                    *self.chunks.get_unchecked_mut(ls_index + 1) |= ms_mask;
                }
                2 => {
                    *self.chunks.get_unchecked_mut(ls_index) |= ls_mask;
                    *self.chunks.get_unchecked_mut(ls_index + 1) |= u64::MAX;
                    *self.chunks.get_unchecked_mut(ls_index + 2) |= ms_mask;
                }
                3 => {
                    *self.chunks.get_unchecked_mut(0) |= ls_mask;
                    *self.chunks.get_unchecked_mut(1) |= u64::MAX;
                    *self.chunks.get_unchecked_mut(2) |= u64::MAX;
                    *self.chunks.get_unchecked_mut(3) |= ms_mask;
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }
}

impl MergeOp<&RangeU8> for Transition {
    #[inline]
    fn merge(&mut self, range: &RangeU8) {
        Self::merge(self, *range)
    }
}

impl MergeOp<std::ops::RangeInclusive<u8>> for Transition {
    #[inline]
    fn merge(&mut self, range: std::ops::RangeInclusive<u8>) {
        let range = RangeU8::from(range);
        MergeOp::merge(self, range)
    }
}

impl MergeOp<&std::ops::RangeInclusive<u8>> for Transition {
    #[inline]
    fn merge(&mut self, range: &std::ops::RangeInclusive<u8>) {
        let range = RangeU8::new(*range.start(), *range.end());
        MergeOp::merge(self, range)
    }
}

impl MergeOp<&Transition> for Transition {
    #[inline]
    fn merge(&mut self, other: &Transition) {
        self.chunks[0] |= other.chunks[0];
        self.chunks[1] |= other.chunks[1];
        self.chunks[2] |= other.chunks[2];
        self.chunks[3] |= other.chunks[3];
        self.chunks[4] |= other.chunks[4];
    }
}

impl MergeOp<Epsilon> for Transition {
    #[inline]
    fn merge(&mut self, _: Epsilon) {
        self.chunks[EPSILON_CHUNK] |= 1;
    }
}

impl std::convert::From<u8> for Transition {
    fn from(value: u8) -> Self {
        let mut tr = Self::default();
        tr.merge(value);
        tr
    }
}

impl std::convert::From<Epsilon> for Transition {
    #[inline]
    fn from(_: Epsilon) -> Self {
        Self::epsilon()
    }
}

impl std::fmt::Display for Transition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        let mut iter = self.ranges();
        let mut range = iter.next();
        let mut has_symbols = false;
        while let Some(cur_range) = range {
            has_symbols = true;
            if let Some(next_range) = iter.next() {
                if cur_range.last().steps_between(next_range.start()) == 1 {
                    range = Some(RangeU8::new(cur_range.start(), next_range.last()));
                    continue;
                } else {
                    std::fmt::Display::fmt(&cur_range.display(), f)?;
                    f.write_str(" | ")?;
                    range = Some(next_range);
                }
            } else {
                std::fmt::Display::fmt(&cur_range.display(), f)?;
                break;
            }
        }
        if self.contains(crate::symbol::Epsilon) {
            if has_symbols {
                f.write_str(" | ")?;
            }
            f.write_str("Epsilon")?;
        }
        f.write_char(']')
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Transition {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_char('[')?;
                let mut first_iter = true;
                for range in self.ranges() {
                    if first_iter {
                        first_iter = false;
                    } else {
                        f.write_str(" | ")?;
                    }
                    ::std::fmt::$trait::fmt(&range, f)?;
                }

                if self.contains(crate::symbol::Epsilon) {
                    if !first_iter {
                        f.write_str(" | ")?;
                    }
                    f.write_str("Epsilon")?;
                }
                f.write_char(']')
            }
        }
    };
}

impl_fmt!(std::fmt::Debug);
impl_fmt!(std::fmt::Binary);
impl_fmt!(std::fmt::Octal);
impl_fmt!(std::fmt::LowerHex);
impl_fmt!(std::fmt::UpperHex);

pub struct SymbolIter<'a> {
    chunks: &'a [u64; BITMAP_LEN],
    chunk: u64,
    shift: u32,
}

impl<'a> SymbolIter<'a> {
    fn new(chunks: &'a [u64; BITMAP_LEN]) -> Self {
        Self {
            chunks,
            chunk: chunks[0],
            shift: 0,
        }
    }
}

impl std::iter::Iterator for SymbolIter<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const SHIFT_OVERFLOW: u32 = (SYM_BITMAP_LEN << 6) as u32;
        while self.shift < SHIFT_OVERFLOW {
            if self.chunk != 0 {
                let trailing_zeros = self.chunk.trailing_zeros();
                self.chunk &= self.chunk.wrapping_sub(1);
                let symbol = trailing_zeros + self.shift;
                return Some(symbol as u8);
            }
            if self.shift < SHIFT_OVERFLOW - 64 {
                self.shift += 64;
                self.chunk = self.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}

pub struct RangeIter<'a> {
    chunks: &'a [u64; BITMAP_LEN],
    chunk: u64,
    shift: u32,
}

impl<'a> RangeIter<'a> {
    fn new(bitmap: &'a [u64; BITMAP_LEN]) -> Self {
        Self {
            chunks: bitmap,
            chunk: bitmap[0],
            shift: 0,
        }
    }
}

impl std::iter::Iterator for RangeIter<'_> {
    type Item = RangeU8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const SHIFT_OVERFLOW: u32 = (SYM_BITMAP_LEN << 6) as u32;
        while self.shift < SHIFT_OVERFLOW {
            if self.chunk != 0 {
                let trailing_zeros = self.chunk.trailing_zeros();
                self.chunk |= self.chunk.wrapping_sub(1);

                let trailing_ones = self.chunk.trailing_ones();
                self.chunk &= self.chunk.wrapping_add(1);

                let start = trailing_zeros + self.shift;
                let end = trailing_ones - 1 + self.shift;

                return Some(RangeU8::new_unchecked(start as u8, end as u8));
            }

            if self.shift < SHIFT_OVERFLOW - 64 {
                self.shift += 64;
                self.chunk = self.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}
