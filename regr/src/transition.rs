use crate::ops::{ContainOp, IntersectOp, MergeOp};
use crate::span::Span;
use crate::symbol::Epsilon;
use crate::symbol::Symbol;
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

    /// Returns iterator over all symbol spans in this trasition instance in
    /// ascendent order.
    pub fn spans(&self) -> SpanIter<'_> {
        SpanIter::new(&self.chunks)
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

impl ContainOp<Span> for Transition {
    #[inline]
    fn contains(&self, span: Span) -> bool {
        let mut other_tr = Transition::default();
        other_tr.merge(span);
        ContainOp::contains(self, &other_tr)
    }
}

impl ContainOp<&Span> for Transition {
    #[inline]
    fn contains(&self, span: &Span) -> bool {
        Self::contains(self, *span)
    }
}

impl ContainOp<std::ops::RangeInclusive<u8>> for Transition {
    #[inline]
    fn contains(&self, range: std::ops::RangeInclusive<u8>) -> bool {
        let span = Span::from(range);
        ContainOp::contains(self, span)
    }
}

impl ContainOp<&std::ops::RangeInclusive<u8>> for Transition {
    #[inline]
    fn contains(&self, range: &std::ops::RangeInclusive<u8>) -> bool {
        let span = Span::from(range);
        ContainOp::contains(self, span)
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

impl IntersectOp<Span> for Transition {
    fn intersects(&self, span: Span) -> bool {
        let mut other = Transition::default();
        other.merge(span);
        IntersectOp::intersects(self, &other)
    }
}

impl IntersectOp<std::ops::RangeInclusive<u8>> for Transition {
    fn intersects(&self, range: std::ops::RangeInclusive<u8>) -> bool {
        let range = Span::from(range);
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

impl MergeOp<Span> for Transition {
    fn merge(&mut self, span: Span) {
        let mut ls_mask = 1 << (span.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (span.end() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = span.start() as usize >> 6;
        let ms_index = span.end() as usize >> 6;

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

impl MergeOp<&Span> for Transition {
    #[inline]
    fn merge(&mut self, span: &Span) {
        Self::merge(self, *span)
    }
}

impl MergeOp<std::ops::RangeInclusive<u8>> for Transition {
    #[inline]
    fn merge(&mut self, range: std::ops::RangeInclusive<u8>) {
        let span = Span::from(range);
        MergeOp::merge(self, span)
    }
}

impl MergeOp<&std::ops::RangeInclusive<u8>> for Transition {
    #[inline]
    fn merge(&mut self, range: &std::ops::RangeInclusive<u8>) {
        let span = Span::from(range);
        MergeOp::merge(self, span)
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

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Transition {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_char('[')?;
                let mut iter = self.spans();
                let mut span = iter.next();
                let mut has_symbols = false;
                while let Some(cur_span) = span {
                    has_symbols = true;
                    if let Some(next_range) = iter.next() {
                        if cur_span.end().steps_between(next_range.start()) == 1 {
                            span = Some(Span::new(cur_span.start(), next_range.end()));
                            continue;
                        } else {
                            std::fmt::$trait::fmt(&cur_span, f)?;
                            f.write_str(" | ")?;
                            span = Some(next_range);
                        }
                    } else {
                        std::fmt::$trait::fmt(&cur_span, f)?;
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
    };
}

impl_fmt!(std::fmt::Display);
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

pub struct SpanIter<'a> {
    chunks: &'a [u64; BITMAP_LEN],
    chunk: u64,
    shift: u32,
}

impl<'a> SpanIter<'a> {
    fn new(bitmap: &'a [u64; BITMAP_LEN]) -> Self {
        Self {
            chunks: bitmap,
            chunk: bitmap[0],
            shift: 0,
        }
    }
}

impl std::iter::Iterator for SpanIter<'_> {
    type Item = Span;

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

                return Some(Span::new_unchecked(start as u8, end as u8));
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
