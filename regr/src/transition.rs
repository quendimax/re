use crate::range::Range;
use std::fmt::Write;

/// Quantity of `u64` values in the `bitmap` member.
const SYM_BITMAP_LEN: usize = 4;

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Transition {
    bitmap: [u64; SYM_BITMAP_LEN],
}

impl Transition {
    pub const BITS: u32 = 256;

    /// Creates a new transition initialized with the given bitmap.
    pub fn new(bitmap: &[u64; SYM_BITMAP_LEN]) -> Self {
        Self { bitmap: *bitmap }
    }

    /// Creates a new transition parsing the given byte array, and setting a bit
    /// corresponding to each byte value in the array.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut bitmap = [0u64; SYM_BITMAP_LEN];
        for byte in bytes {
            bitmap[*byte as usize >> 6] |= 1 << (byte & 0x3F);
        }
        Self::new(&bitmap)
    }

    /// Returns iterator over all symbols in this trasition instance in
    /// ascendent order.
    pub fn symbols(&self) -> SymbolIter {
        SymbolIter::new(&self.bitmap)
    }

    /// Returns iterator over all symbol ranges in this trasition instance in
    /// ascendent order.
    pub fn ranges(&self) -> RangeIter {
        RangeIter::new(&self.bitmap)
    }

    /// Merges the `other` symmap into `self` one.
    pub fn merge(&mut self, other: &Self) {
        self.bitmap[0] |= other.bitmap[0];
        self.bitmap[1] |= other.bitmap[1];
        self.bitmap[2] |= other.bitmap[2];
        self.bitmap[3] |= other.bitmap[3];
    }
}

impl std::convert::AsRef<[u64; SYM_BITMAP_LEN]> for Transition {
    fn as_ref(&self) -> &[u64; SYM_BITMAP_LEN] {
        &self.bitmap
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
    bitmap: &'a [u64; SYM_BITMAP_LEN],
    reg: u64,
    shift: u32,
}

impl<'a> SymbolIter<'a> {
    fn new(bitmap: &'a [u64; SYM_BITMAP_LEN]) -> Self {
        Self {
            bitmap,
            reg: bitmap[0],
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
            if self.reg != 0 {
                let trailing_zeros = self.reg.trailing_zeros();
                self.reg &= self.reg.wrapping_sub(1);
                let symbol = trailing_zeros + self.shift;
                return Some(symbol as u8);
            }
            if self.shift < SHIFT_OVERFLOW - 64 {
                self.shift += 64;
                self.reg = self.bitmap[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}

pub struct RangeIter<'a> {
    bitmap: &'a [u64; SYM_BITMAP_LEN],
    reg: u64,
    shift: u32,
}

impl<'a> RangeIter<'a> {
    fn new(bitmap: &'a [u64; SYM_BITMAP_LEN]) -> Self {
        Self {
            bitmap,
            reg: bitmap[0],
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
            if self.reg != 0 {
                let trailing_zeros = self.reg.trailing_zeros();
                self.reg |= self.reg.wrapping_sub(1);

                let trailing_ones = self.reg.trailing_ones();
                self.reg &= self.reg.wrapping_add(1);

                let start = trailing_zeros + self.shift;
                let end = trailing_ones - 1 + self.shift;

                return Some(Range::new_unchecked(start as u8, end as u8));
            }

            if self.shift < SHIFT_OVERFLOW - 64 {
                self.shift += 64;
                self.reg = self.bitmap[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}
