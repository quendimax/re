use crate::{Legible, RangeU8, Step};
use std::fmt::Write;
use std::ops::Deref;
use std::ops::RangeInclusive;

type Chunk = u64;

/// Quantity of `Chunk` values in the `chunks` member for symbols' bits.
const BITMAP_LEN: usize = (u8::MAX as usize + 1) / Chunk::BITS as usize;

/// A set of symbols that can be used to represent any byte.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetU8 {
    chunks: [Chunk; BITMAP_LEN],
}

impl SetU8 {
    /// Creates a new empty symbol set.
    #[inline]
    pub fn new() -> Self {
        Self::empty()
    }

    /// Creates a new empty symbol set.
    #[inline]
    pub fn empty() -> Self {
        Self {
            chunks: [0; BITMAP_LEN],
        }
    }
}

impl std::default::Default for SetU8 {
    /// Creates a new empty symbol set.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::convert::From<u8> for SetU8 {
    fn from(value: u8) -> Self {
        let mut set = Self::empty();
        set |= value;
        set
    }
}

impl std::convert::From<std::ops::RangeInclusive<u8>> for SetU8 {
    fn from(value: std::ops::RangeInclusive<u8>) -> Self {
        let mut set = Self::empty();
        set |= RangeU8::from(value);
        set
    }
}

impl std::convert::From<RangeU8> for SetU8 {
    fn from(value: RangeU8) -> Self {
        let mut set = Self::empty();
        set |= value;
        set
    }
}

impl std::convert::From<&RangeU8> for SetU8 {
    fn from(value: &RangeU8) -> Self {
        let mut set = Self::empty();
        set |= value;
        set
    }
}

impl std::convert::From<&[u8]> for SetU8 {
    fn from(value: &[u8]) -> Self {
        let mut set = Self::default();
        for byte in value {
            set |= *byte;
        }
        set
    }
}

impl<const N: usize> std::convert::From<&[u8; N]> for SetU8 {
    fn from(value: &[u8; N]) -> Self {
        std::convert::From::<&[u8]>::from(&value[..])
    }
}

impl std::convert::AsRef<SetU8> for SetU8 {
    #[inline]
    fn as_ref(&self) -> &SetU8 {
        self
    }
}

macro_rules! impl_ops {
    ($op_assign_trait:ident, $op_assign_fn:ident, $op_trait:ident, $op_fn:ident) => {
        impl ::std::ops::$op_assign_trait<&SetU8> for SetU8 {
            #[inline]
            fn $op_assign_fn(&mut self, rhs: &SetU8) {
                self.chunks[0].$op_assign_fn(&rhs.chunks[0]);
                self.chunks[1].$op_assign_fn(&rhs.chunks[1]);
                self.chunks[2].$op_assign_fn(&rhs.chunks[2]);
                self.chunks[3].$op_assign_fn(&rhs.chunks[3]);
            }
        }

        impl ::std::ops::$op_assign_trait<SetU8> for SetU8 {
            #[inline]
            fn $op_assign_fn(&mut self, rhs: SetU8) {
                self.$op_assign_fn(&rhs);
            }
        }

        impl ::std::ops::$op_assign_trait<&RangeU8> for SetU8 {
            #[inline]
            fn $op_assign_fn(&mut self, range: &RangeU8) {
                let (ls_mask, ms_mask, ls_index, ms_index) = find_masks_indices(*range);
                unsafe {
                    match ms_index - ls_index {
                        0 => {
                            let mask = ls_mask & ms_mask;
                            self.chunks.get_unchecked_mut(ls_index).$op_assign_fn(mask);
                        }
                        1 => {
                            self.chunks
                                .get_unchecked_mut(ls_index)
                                .$op_assign_fn(ls_mask);
                            self.chunks
                                .get_unchecked_mut(ls_index + 1)
                                .$op_assign_fn(ms_mask);
                        }
                        2 => {
                            self.chunks
                                .get_unchecked_mut(ls_index)
                                .$op_assign_fn(ls_mask);
                            self.chunks
                                .get_unchecked_mut(ls_index + 1)
                                .$op_assign_fn(Chunk::MAX);
                            self.chunks
                                .get_unchecked_mut(ls_index + 2)
                                .$op_assign_fn(ms_mask);
                        }
                        3 => {
                            self.chunks.get_unchecked_mut(0).$op_assign_fn(ls_mask);
                            self.chunks.get_unchecked_mut(1).$op_assign_fn(Chunk::MAX);
                            self.chunks.get_unchecked_mut(2).$op_assign_fn(Chunk::MAX);
                            self.chunks.get_unchecked_mut(3).$op_assign_fn(ms_mask);
                        }
                        _ => std::hint::unreachable_unchecked(),
                    };
                };
            }
        }

        impl ::std::ops::$op_assign_trait<RangeU8> for SetU8 {
            #[inline]
            fn $op_assign_fn(&mut self, rhs: RangeU8) {
                self.$op_assign_fn(&rhs);
            }
        }

        impl ::std::ops::$op_assign_trait<&u8> for SetU8 {
            #[inline]
            fn $op_assign_fn(&mut self, byte: &u8) {
                self.chunks[*byte as usize >> 6].$op_assign_fn(1 << (*byte & (u8::MAX >> 2)));
            }
        }

        impl ::std::ops::$op_assign_trait<u8> for SetU8 {
            #[inline]
            fn $op_assign_fn(&mut self, byte: u8) {
                self.$op_assign_fn(&byte);
            }
        }

        impl ::std::ops::$op_trait<SetU8> for SetU8 {
            type Output = Self;

            #[inline]
            fn $op_fn(self, rhs: SetU8) -> Self {
                let mut result = self.clone();
                ::std::ops::$op_assign_trait::$op_assign_fn(&mut result, &rhs);
                result
            }
        }

        impl ::std::ops::$op_trait<RangeU8> for SetU8 {
            type Output = Self;

            #[inline]
            fn $op_fn(self, rhs: RangeU8) -> Self {
                let mut result = self.clone();
                ::std::ops::$op_assign_trait::$op_assign_fn(&mut result, &rhs);
                result
            }
        }

        impl ::std::ops::$op_trait<u8> for SetU8 {
            type Output = Self;

            #[inline]
            fn $op_fn(self, rhs: u8) -> Self {
                let mut result = self.clone();
                ::std::ops::$op_assign_trait::$op_assign_fn(&mut result, &rhs);
                result
            }
        }
    };
}

impl_ops!(BitAndAssign, bitand_assign, BitAnd, bitand);
impl_ops!(BitOrAssign, bitor_assign, BitOr, bitor);
impl_ops!(BitXorAssign, bitxor_assign, BitXor, bitxor);

impl std::ops::Not for SetU8 {
    type Output = Self;

    #[inline]
    fn not(self) -> Self {
        Self {
            chunks: [
                !self.chunks[0],
                !self.chunks[1],
                !self.chunks[2],
                !self.chunks[3],
            ],
        }
    }
}

impl crate::ops::Containable<u8> for SetU8 {
    #[inline]
    fn contains(&self, byte: u8) -> bool {
        self.chunks[byte as usize >> 6] & (1 << (byte & (u8::MAX >> 2))) != 0
    }
}

impl crate::ops::Containable<RangeU8> for SetU8 {
    fn contains(&self, range: RangeU8) -> bool {
        let (ls_mask, ms_mask, ls_index, ms_index) = find_masks_indices(range);
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
}

impl crate::ops::Containable<&SetU8> for SetU8 {
    #[inline]
    fn contains(&self, rhs: &SetU8) -> bool {
        self.chunks[0] & rhs.chunks[0] == rhs.chunks[0]
            && self.chunks[1] & rhs.chunks[1] == rhs.chunks[1]
            && self.chunks[2] & rhs.chunks[2] == rhs.chunks[2]
            && self.chunks[3] & rhs.chunks[3] == rhs.chunks[3]
    }
}

impl crate::ops::Containable for SetU8 {
    #[inline]
    fn contains(&self, rhs: Self) -> bool {
        self.contains(&rhs)
    }
}

impl crate::ops::Intersectable<u8> for SetU8 {
    #[inline]
    fn intersects(&self, byte: u8) -> bool {
        self.chunks[byte as usize >> 6] & (1 << (byte & (u8::MAX >> 2))) != 0
    }
}

impl crate::ops::Intersectable<RangeU8> for SetU8 {
    fn intersects(&self, range: RangeU8) -> bool {
        let (ls_mask, ms_mask, ls_index, ms_index) = find_masks_indices(range);
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
}

impl crate::ops::Intersectable<&SetU8> for SetU8 {
    #[inline]
    fn intersects(&self, rhs: &SetU8) -> bool {
        self.chunks[0] & rhs.chunks[0] != 0
            || self.chunks[1] & rhs.chunks[1] != 0
            || self.chunks[2] & rhs.chunks[2] != 0
            || self.chunks[3] & rhs.chunks[3] != 0
    }
}

impl crate::ops::Intersectable for SetU8 {
    #[inline]
    fn intersects(&self, rhs: Self) -> bool {
        self.intersects(&rhs)
    }
}

impl crate::ops::Includable<u8> for SetU8 {
    #[inline]
    fn include(&mut self, byte: u8) -> &mut Self {
        *self |= byte;
        self
    }
}

impl crate::ops::Includable<RangeU8> for SetU8 {
    #[inline]
    fn include(&mut self, range: RangeU8) -> &mut Self {
        *self |= range;
        self
    }
}

impl crate::ops::Includable<RangeInclusive<u8>> for SetU8 {
    #[inline]
    fn include(&mut self, range: RangeInclusive<u8>) -> &mut Self {
        *self |= RangeU8::from(range);
        self
    }
}

impl crate::ops::Includable<&SetU8> for SetU8 {
    #[inline]
    fn include(&mut self, rhs: &SetU8) -> &mut Self {
        *self |= rhs;
        self
    }
}

impl crate::ops::Includable for SetU8 {
    #[inline]
    fn include(&mut self, rhs: SetU8) -> &mut Self {
        *self |= rhs;
        self
    }
}

impl crate::ops::Excludable<u8> for SetU8 {
    #[inline]
    fn exclude(&mut self, byte: u8) -> &mut Self {
        self.chunks[byte as usize >> 6] &= !(1 << (byte & (u8::MAX >> 2)));
        self
    }
}

impl crate::ops::Excludable<RangeU8> for SetU8 {
    #[inline]
    fn exclude(&mut self, range: RangeU8) -> &mut Self {
        let (ls_mask, ms_mask, ls_index, ms_index) = find_masks_indices(range);
        unsafe {
            match ms_index - ls_index {
                0 => {
                    *self.chunks.get_unchecked_mut(ls_index) &= !(ls_mask & ms_mask);
                }
                1 => {
                    *self.chunks.get_unchecked_mut(ls_index) &= !ls_mask;
                    *self.chunks.get_unchecked_mut(ls_index + 1) &= !ms_mask;
                }
                2 => {
                    *self.chunks.get_unchecked_mut(ls_index) &= !ls_mask;
                    *self.chunks.get_unchecked_mut(ls_index + 1) &= !Chunk::MAX;
                    *self.chunks.get_unchecked_mut(ls_index + 2) &= !ms_mask;
                }
                3 => {
                    *self.chunks.get_unchecked_mut(0) &= !ls_mask;
                    *self.chunks.get_unchecked_mut(1) &= !Chunk::MAX;
                    *self.chunks.get_unchecked_mut(2) &= !Chunk::MAX;
                    *self.chunks.get_unchecked_mut(3) &= !ms_mask;
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
        self
    }
}

impl crate::ops::Excludable<RangeInclusive<u8>> for SetU8 {
    fn exclude(&mut self, rhs: RangeInclusive<u8>) -> &mut Self {
        self.exclude(RangeU8::from(rhs));
        self
    }
}

impl crate::ops::Excludable<&SetU8> for SetU8 {
    #[inline]
    fn exclude(&mut self, rhs: &SetU8) -> &mut Self {
        *self &= !rhs.clone();
        self
    }
}

impl crate::ops::Excludable for SetU8 {
    #[inline]
    fn exclude(&mut self, rhs: SetU8) -> &mut Self {
        *self &= !rhs;
        self
    }
}

impl SetU8 {
    /// Checks if the set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.chunks.iter().all(|&chunk| chunk == 0)
    }

    /// Returns an iterator over the bytes in the set.
    pub fn bytes(&self) -> impl Iterator<Item = u8> {
        ByteIter::new(self)
    }

    /// Returns an iterator over the inclusive byte ranges in the set.
    pub fn ranges(&self) -> impl Iterator<Item = RangeU8> {
        RangeIter::new(self)
    }
}

impl std::fmt::Display for SetU8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        let mut iter = self.ranges();
        let mut range = iter.next();
        while let Some(cur_range) = range {
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
        f.write_char(']')
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

fn find_masks_indices(range: RangeU8) -> (u64, u64, usize, usize) {
    let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
    ls_mask = !(ls_mask - 1);

    let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
    ms_mask |= ms_mask - 1;

    let ls_index = (range.start() >> 6) as usize;
    let ms_index = (range.last() >> 6) as usize;

    (ls_mask, ms_mask, ls_index, ms_index)
}
