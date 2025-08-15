use std::ops::Deref;

use crate::RangeU8;

type Chunk = u64;

/// Quantity of `Chunk` values in the `chunks` member for symbols' bits.
const BITMAP_LEN: usize = (u8::MAX as usize + 1) / Chunk::BITS as usize;

/// A set of symbols that can be used to represent a language alphabet + Epsilon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetU8 {
    chunks: [Chunk; BITMAP_LEN],
}

impl SetU8 {
    /// Creates a new empty symbol set.
    #[inline]
    pub fn new() -> Self {
        Self {
            chunks: [0; BITMAP_LEN],
        }
    }
}

impl std::default::Default for SetU8 {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SetU8 {
    pub fn contains_byte(&self, byte: u8) -> bool {
        self.chunks[byte as usize >> 6] & (1 << (byte & (u8::MAX >> 2))) != 0
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

    pub fn contains_set(&self, other: &Self) -> bool {
        self.chunks[0] & other.chunks[0] == other.chunks[0]
            && self.chunks[1] & other.chunks[1] == other.chunks[1]
            && self.chunks[2] & other.chunks[2] == other.chunks[2]
            && self.chunks[3] & other.chunks[3] == other.chunks[3]
    }

    pub fn intersects_byte(&self, byte: u8) -> bool {
        self.chunks[byte as usize >> 6] & (1 << (byte & (u8::MAX >> 2))) != 0
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

    pub fn intersects_set(&self, other: &SetU8) -> bool {
        self.chunks[0] & other.chunks[0] != 0
            || self.chunks[1] & other.chunks[1] != 0
            || self.chunks[2] & other.chunks[2] != 0
            || self.chunks[3] & other.chunks[3] != 0
    }

    pub fn merge_byte(&mut self, byte: u8) {
        self.chunks[byte as usize >> 6] |= 1 << (byte & (u8::MAX >> 2));
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

    pub fn merge_set(&mut self, other: &SetU8) {
        self.chunks[0] |= other.chunks[0];
        self.chunks[1] |= other.chunks[1];
        self.chunks[2] |= other.chunks[2];
        self.chunks[3] |= other.chunks[3];
    }

    pub fn bytes(&self) -> impl Iterator<Item = u8> {
        ByteIter::new(self)
    }

    pub fn ranges(&self) -> impl Iterator<Item = RangeU8> {
        RangeIter::new(self)
    }
}

pub struct ByteIter<T> {
    set: T,
    chunk: Chunk,
    shift: u32,
}

impl<T> ByteIter<T>
where
    T: Deref<Target = SetU8>,
{
    pub fn new(set: T) -> Self {
        let chunk = set.chunks[0];
        Self {
            set,
            chunk,
            shift: 0,
        }
    }
}

impl<T> std::iter::Iterator for ByteIter<T>
where
    T: Deref<Target = SetU8>,
{
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
                self.chunk = self.set.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}

pub struct RangeIter<T> {
    set: T,
    chunk: Chunk,
    shift: u32,
}

impl<T> RangeIter<T>
where
    T: Deref<Target = SetU8>,
{
    pub fn new(set: T) -> Self {
        let chunk = set.chunks[0];
        Self {
            set,
            chunk,
            shift: 0,
        }
    }
}

impl<T> std::iter::Iterator for RangeIter<T>
where
    T: Deref<Target = SetU8>,
{
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
                self.chunk = self.set.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}
