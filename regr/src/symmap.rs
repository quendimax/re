use crate::symrng::SymRng;

/// Quantity of `u64` values in the `bitmap` member.
const SYM_BITMAP_LEN: usize = 4;

#[derive(Clone, Debug)]
pub struct Symmap {
    bitmap: [u64; SYM_BITMAP_LEN],
}

impl Symmap {
    pub const BITS: u32 = 256;

    /// Creates a new empty Symmap.
    pub fn new(bitmap: &[u64; SYM_BITMAP_LEN]) -> Self {
        Self { bitmap: *bitmap }
    }

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

impl std::default::Default for Symmap {
    fn default() -> Self {
        Self {
            bitmap: Default::default(),
        }
    }
}

impl std::convert::AsRef<[u64; SYM_BITMAP_LEN]> for Symmap {
    fn as_ref(&self) -> &[u64; SYM_BITMAP_LEN] {
        &self.bitmap
    }
}

pub struct RangeIter<'a> {
    bitmap: &'a [u64; SYM_BITMAP_LEN],
    reg: u64,
    index: usize,
    already_shifted: u32,
}

impl<'a> RangeIter<'a> {
    fn new(bitmap: &'a [u64; SYM_BITMAP_LEN]) -> Self {
        Self {
            bitmap,
            reg: bitmap[0],
            index: 0,
            already_shifted: 0,
        }
    }
}

impl<'a> std::iter::Iterator for RangeIter<'a> {
    type Item = SymRng;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < SYM_BITMAP_LEN {
            if self.reg != 0 {
                let trailing_zeros = self.reg.trailing_zeros();
                self.reg >>= trailing_zeros;

                let trailing_ones = self.reg.trailing_ones();
                self.reg = self.reg.checked_shr(trailing_ones).unwrap_or(0);

                let start = trailing_zeros + self.already_shifted;
                let end = start + (trailing_ones - 1);

                self.already_shifted += trailing_zeros + trailing_ones;
                return Some(SymRng::new_unchecked(start as u8, end as u8));
            }

            self.index += 1;
            if self.index < SYM_BITMAP_LEN {
                self.reg = self.bitmap[self.index];
                self.already_shifted = (self.index as u32) << 6;
            } else {
                break;
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::symrng::rng;
    use pretty_assertions::assert_eq;

    #[test]
    fn symmap_ranges() {
        let map = |a: u64, b: u64, c: u64, d: u64| {
            let map = Symmap::new(&[a, b, c, d]);
            map.ranges().collect::<Vec<SymRng>>()
        };
        for _ in 0..500_000 {
            assert_eq!(map(0, 0, 0, 0), vec![]);
            assert_eq!(map(255, 0, 0, 0), vec![rng(0, 7)]);
            assert_eq!(map(255, 255, 0, 0), vec![rng(0, 7), rng(64, 71)]);
            assert_eq!(map(0, 255, 0, 0), vec![rng(64, 71)]);
            assert_eq!(map(0, 0, 0, 255), vec![rng(192, 199)]);
            assert_eq!(
                map(255, 255, 255, 255),
                vec![rng(0, 7), rng(64, 71), rng(128, 135), rng(192, 199)]
            );
            assert_eq!(map(u64::MAX, 0, 0, 0), vec![rng(0, 63)]);
            assert_eq!(map(0, u64::MAX, 0, 0), vec![rng(64, 127)]);
            assert_eq!(
                map(u64::MAX, 0, 0, u64::MAX),
                vec![rng(0, 63), rng(192, 255)]
            );
            assert_eq!(
                map(u64::MAX, u64::MAX, u64::MAX, u64::MAX),
                vec![rng(0, 63), rng(64, 127), rng(128, 191), rng(192, 255)]
            );
            assert_eq!(map(1, 0, 0, 0), vec![rng(0, 0)]);
            assert_eq!(
                map(0x8000000000000001, 0, 0, 0),
                vec![rng(0, 0), rng(63, 63)]
            );
            assert_eq!(
                map(0x8000000000000001, 0x8000000000000001, 0, 0),
                vec![rng(0, 0), rng(63, 63), rng(64, 64), rng(127, 127)]
            );
            assert_eq!(
                map(0xC000000000000007, 0x1F000001, 0, 0),
                vec![rng(0, 2), rng(62, 63), rng(64, 64), rng(88, 92)]
            );
        }
    }
}
