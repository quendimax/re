use redt::RangeU8;
use std::cell::Ref;

pub(crate) type Chunk = u64;

/// Quantity of `Chunk` values in the `chunks` member for symbols' bits.
const BITMAP_LEN: usize = (u8::MAX as usize + 1) / Chunk::BITS as usize;

/// A set of symbols that can be used to represent a language alphabet + Epsilon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolSet {
    chunks: [Chunk; BITMAP_LEN],
    epsilon: bool,
}

impl SymbolSet {
    /// Creates a new empty symbol set.
    #[inline]
    pub fn new() -> Self {
        Self {
            chunks: [0; BITMAP_LEN],
            epsilon: false,
        }
    }
}

impl std::default::Default for SymbolSet {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolSet {
    pub fn contains_epsilon(&self, _: Epsilon) -> bool {
        self.epsilon
    }

    pub fn contains_symbol(&self, symbol: u8) -> bool {
        self.chunks[symbol as usize >> 6] & (1 << (symbol & (u8::MAX >> 2))) != 0
    }

    pub fn contains_range(&self, range: RangeU8) -> bool {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        unsafe {
            match ms_index - ls_index {
                0 => {
                    let mask = ls_mask & ms_mask;
                    *self.chunks.get_unchecked(ls_index) & mask == mask
                }
                1 => {
                    *self.chunks.get_unchecked(ls_index) & ls_mask == ls_mask
                        && *self.chunks.get_unchecked(ls_index + 1) & ms_mask == ms_mask
                }
                2 => {
                    *self.chunks.get_unchecked(ls_index) & ls_mask == ls_mask
                        && *self.chunks.get_unchecked(ls_index + 1) == Chunk::MAX
                        && *self.chunks.get_unchecked(ls_index + 2) & ms_mask == ms_mask
                }
                3 => {
                    *self.chunks.get_unchecked(0) & ls_mask == ls_mask
                        && *self.chunks.get_unchecked(1) == Chunk::MAX
                        && *self.chunks.get_unchecked(2) == Chunk::MAX
                        && *self.chunks.get_unchecked(3) & ms_mask == ms_mask
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }

    pub fn contains_symset(&self, other: &SymbolSet) -> bool {
        self.epsilon & other.epsilon == other.epsilon
            && self.chunks[0] & other.chunks[0] == other.chunks[0]
            && self.chunks[1] & other.chunks[1] == other.chunks[1]
            && self.chunks[2] & other.chunks[2] == other.chunks[2]
            && self.chunks[3] & other.chunks[3] == other.chunks[3]
    }

    pub fn intersects_epsilon(&self, _: Epsilon) -> bool {
        self.epsilon
    }

    pub fn intersects_symbol(&self, symbol: u8) -> bool {
        self.chunks[symbol as usize >> 6] & (1 << (symbol & (u8::MAX >> 2))) != 0
    }

    pub fn intersects_range(&self, range: RangeU8) -> bool {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        unsafe {
            match ms_index - ls_index {
                0 => {
                    let mask = ls_mask & ms_mask;
                    *self.chunks.get_unchecked(ls_index) & mask != 0
                }
                1 => {
                    *self.chunks.get_unchecked(ls_index) & ls_mask != 0
                        || *self.chunks.get_unchecked(ls_index + 1) & ms_mask != 0
                }
                2 => {
                    *self.chunks.get_unchecked(ls_index) & ls_mask != 0
                        || *self.chunks.get_unchecked(ls_index + 1) != 0
                        || *self.chunks.get_unchecked(ls_index + 2) & ms_mask != 0
                }
                3 => {
                    *self.chunks.get_unchecked(0) & ls_mask != 0
                        || *self.chunks.get_unchecked(1) != 0
                        || *self.chunks.get_unchecked(2) != 0
                        || *self.chunks.get_unchecked(3) & ms_mask != 0
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }

    pub fn intersects_symset(&self, other: &SymbolSet) -> bool {
        (self.epsilon && other.epsilon)
            || self.chunks[0] & other.chunks[0] != 0
            || self.chunks[1] & other.chunks[1] != 0
            || self.chunks[2] & other.chunks[2] != 0
            || self.chunks[3] & other.chunks[3] != 0
    }

    pub fn merge_epsilon(&mut self, _: Epsilon) {
        self.epsilon = true;
    }

    pub fn merge_symbol(&mut self, symbol: u8) {
        self.chunks[symbol as usize >> 6] |= 1 << (symbol & (u8::MAX >> 2));
    }

    pub fn merge_range(&mut self, range: RangeU8) {
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
                    *self.chunks.get_unchecked_mut(ls_index + 1) |= Chunk::MAX;
                    *self.chunks.get_unchecked_mut(ls_index + 2) |= ms_mask;
                }
                3 => {
                    *self.chunks.get_unchecked_mut(0) |= ls_mask;
                    *self.chunks.get_unchecked_mut(1) |= Chunk::MAX;
                    *self.chunks.get_unchecked_mut(2) |= Chunk::MAX;
                    *self.chunks.get_unchecked_mut(3) |= ms_mask;
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }

    pub fn merge_symset(&mut self, other: &SymbolSet) {
        self.chunks[0] |= other.chunks[0];
        self.chunks[1] |= other.chunks[1];
        self.chunks[2] |= other.chunks[2];
        self.chunks[3] |= other.chunks[3];
        self.epsilon |= other.epsilon;
    }
}

pub(crate) struct SymbolIter<'a> {
    symset: Ref<'a, SymbolSet>,
    chunk: Chunk,
    shift: u32,
}

impl<'a> SymbolIter<'a> {
    pub(crate) fn new(symset: Ref<'a, SymbolSet>) -> Self {
        let chunk = symset.chunks[0];
        Self {
            symset,
            chunk,
            shift: 0,
        }
    }
}

impl std::iter::Iterator for SymbolIter<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const SHIFT_OVERFLOW: u32 = (BITMAP_LEN << 6) as u32;
        while self.shift < SHIFT_OVERFLOW {
            if self.chunk != 0 {
                let trailing_zeros = self.chunk.trailing_zeros();
                self.chunk &= self.chunk.wrapping_sub(1);
                let symbol = trailing_zeros + self.shift;
                return Some(symbol as u8);
            }
            if self.shift < SHIFT_OVERFLOW - 64 {
                self.shift += 64;
                self.chunk = self.symset.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}

pub(crate) struct RangeIter<'a> {
    symset: Ref<'a, SymbolSet>,
    chunk: Chunk,
    shift: u32,
}

impl<'a> RangeIter<'a> {
    pub(crate) fn new(symset: Ref<'a, SymbolSet>) -> Self {
        let chunk = symset.chunks[0];
        Self {
            symset,
            chunk,
            shift: 0,
        }
    }
}

impl std::iter::Iterator for RangeIter<'_> {
    type Item = RangeU8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const SHIFT_OVERFLOW: u32 = (BITMAP_LEN << 6) as u32;
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
                self.chunk = self.symset.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}

/// A special symbol out of the language alphabet that represents automaton's
/// transition with no real symbol.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Epsilon;

macro_rules! impl_fmt {
    (std::fmt::$trait:ident for $type:ident) => {
        impl std::fmt::$trait for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("Epsilon")
            }
        }
    };
}

impl_fmt!(std::fmt::Display for Epsilon);
impl_fmt!(std::fmt::Debug for Epsilon);
impl_fmt!(std::fmt::Binary for Epsilon);
impl_fmt!(std::fmt::Octal for Epsilon);
impl_fmt!(std::fmt::LowerHex for Epsilon);
impl_fmt!(std::fmt::UpperHex for Epsilon);
