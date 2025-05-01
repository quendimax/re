use crate::range::Range;
use std::fmt::Write;

/// Quantity of `u64` values in the `bitmap` member.
const SYM_BITMAP_LEN: usize = 4;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Transition {
    chunks: [u64; SYM_BITMAP_LEN],
}

impl Transition {
    pub const BITS: u32 = 256;

    /// Creates a new transition initialized with the given bitmap.
    pub fn new(chunks: &[u64; SYM_BITMAP_LEN]) -> Self {
        Self { chunks: *chunks }
    }

    /// Creates a new transition parsing the given byte array, and setting a bit
    /// corresponding to each byte value in the array.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut tr = Transition::default();
        for byte in bytes {
            tr.merge(*byte);
        }
        tr
    }

    /// Returns iterator over all symbols in this trasition instance in
    /// ascendent order.
    pub fn symbols(&self) -> SymbolIter {
        SymbolIter::new(&self.chunks)
    }

    /// Returns iterator over all symbol ranges in this trasition instance in
    /// ascendent order.
    pub fn ranges(&self) -> RangeIter {
        RangeIter::new(&self.chunks)
    }

    /// Merges the `other` boject into this transition.
    pub fn merge<T>(&mut self, other: T)
    where
        Self: std::ops::BitOrAssign<T>,
    {
        *self |= other;
    }

    pub fn contains(&self, symbol: u8) -> bool {
        let res = self.chunks[symbol as usize >> 6] & 1 << (symbol & (u8::MAX >> 2));
        res != 0
    }
}

impl std::ops::BitOrAssign<&Transition> for Transition {
    #[inline]
    fn bitor_assign(&mut self, other: &Transition) {
        self.chunks[0] |= other.chunks[0];
        self.chunks[1] |= other.chunks[1];
        self.chunks[2] |= other.chunks[2];
        self.chunks[3] |= other.chunks[3];
    }
}

impl std::ops::BitOrAssign<Transition> for Transition {
    #[inline]
    fn bitor_assign(&mut self, other: Transition) {
        self.bitor_assign(&other)
    }
}

impl std::ops::BitOrAssign<Range> for Transition {
    fn bitor_assign(&mut self, range: Range) {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.end() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = range.start() as usize >> 6;
        let ms_index = range.end() as usize >> 6;

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
                    *self.chunks.get_unchecked_mut(ls_index) |= ls_mask;
                    *self.chunks.get_unchecked_mut(ls_index + 1) |= u64::MAX;
                    *self.chunks.get_unchecked_mut(ls_index + 2) |= u64::MAX;
                    *self.chunks.get_unchecked_mut(ls_index + 3) |= ms_mask;
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }
}

impl std::ops::BitOrAssign<u8> for Transition {
    /// Merges a symbol into this transition.
    #[inline]
    fn bitor_assign(&mut self, symbol: u8) {
        self.chunks[symbol as usize >> 6] |= 1 << (symbol & (u8::MAX >> 2));
    }
}

impl std::convert::AsRef<[u64; SYM_BITMAP_LEN]> for Transition {
    fn as_ref(&self) -> &[u64; SYM_BITMAP_LEN] {
        &self.chunks
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Transition {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_char('[')?;
                let mut iter = self.ranges();
                let mut range = iter.next();
                loop {
                    if let Some(range) = range {
                        std::fmt::$trait::fmt(&range, f)?;
                    }
                    if let Some(next_range) = iter.next() {
                        f.write_str(" | ")?;
                        range = Some(next_range);
                    } else {
                        break;
                    }
                }
                f.write_char(']')
            }
        }
    };
}

impl_fmt!(std::fmt::Display);
impl_fmt!(std::fmt::Debug);
impl_fmt!(std::fmt::Binary);
impl_fmt!(std::fmt::Octal);
impl_fmt!(std::fmt::LowerHex);
impl_fmt!(std::fmt::UpperHex);

pub struct SymbolIter<'a> {
    chunks: &'a [u64; SYM_BITMAP_LEN],
    chunk: u64,
    shift: u32,
}

impl<'a> SymbolIter<'a> {
    fn new(chunks: &'a [u64; SYM_BITMAP_LEN]) -> Self {
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
    chunks: &'a [u64; SYM_BITMAP_LEN],
    chunk: u64,
    shift: u32,
}

impl<'a> RangeIter<'a> {
    fn new(bitmap: &'a [u64; SYM_BITMAP_LEN]) -> Self {
        Self {
            chunks: bitmap,
            chunk: bitmap[0],
            shift: 0,
        }
    }
}

impl std::iter::Iterator for RangeIter<'_> {
    type Item = Range;

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

                return Some(Range::new_unchecked(start as u8, end as u8));
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
