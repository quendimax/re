use crate::range::Range;

/// Quantity of `u64` values in the `bitmap` member.
const SYM_BITMAP_LEN: usize = 4;

#[derive(Clone, Debug)]
pub struct Transition {
    bitmap: [u64; SYM_BITMAP_LEN],
}

impl Transition {
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

impl std::default::Default for Transition {
    fn default() -> Self {
        Self {
            bitmap: Default::default(),
        }
    }
}

impl std::convert::AsRef<[u64; SYM_BITMAP_LEN]> for Transition {
    fn as_ref(&self) -> &[u64; SYM_BITMAP_LEN] {
        &self.bitmap
    }
}

pub struct RangeIter<'a> {
    bitmap: &'a [u64; SYM_BITMAP_LEN],
    reg: u64,
    index: u32,
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
    type Item = Range;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < SYM_BITMAP_LEN as u32 {
            if self.reg != 0 {
                let trailing_zeros = self.reg.trailing_zeros();
                self.reg >>= trailing_zeros;

                let trailing_ones = self.reg.trailing_ones();
                self.reg = self.reg.checked_shr(trailing_ones).unwrap_or(0);

                let start = trailing_zeros + self.already_shifted;
                let end = start + (trailing_ones - 1);

                self.already_shifted += trailing_zeros + trailing_ones;
                return Some(Range::new_unchecked(start as u8, end as u8));
            }

            self.index += 1;
            if self.index < SYM_BITMAP_LEN as u32 {
                self.reg = self.bitmap[self.index as usize];
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
    use crate::range::range;
    use pretty_assertions::assert_eq;

    #[test]
    fn symmap_ranges() {
        fn sm(a: u64, b: u64, c: u64, d: u64) -> Vec<Range> {
            let smap = Transition::new(&[a, b, c, d]);
            smap.ranges().collect::<Vec<Range>>()
        }
        assert_eq!(sm(0, 0, 0, 0), vec![]);
        assert_eq!(sm(255, 0, 0, 0), vec![range(0..=7)]);
        assert_eq!(sm(255, 255, 0, 0), vec![range(0..=7), range(64..=71)]);
        assert_eq!(sm(0, 255, 0, 0), vec![range(64..=71)]);
        assert_eq!(sm(0, 0, 0, 255), vec![range(192..=199)]);
        assert_eq!(
            sm(255, 255, 255, 255),
            vec![range(0..=7), range(64..=71), range(128..=135), range(192..=199)]
        );
        assert_eq!(sm(u64::MAX, 0, 0, 0), vec![range(0..=63)]);
        assert_eq!(sm(0, u64::MAX, 0, 0), vec![range(64..=127)]);
        assert_eq!(
            sm(u64::MAX, 0, 0, u64::MAX),
            vec![range(0..=63), range(192..=255)]
        );
        assert_eq!(
            sm(u64::MAX, u64::MAX, u64::MAX, u64::MAX),
            vec![range(0..=63), range(64..=127), range(128..=191), range(192..=255)]
        );
        assert_eq!(sm(1, 0, 0, 0), vec![range(0)]);
        assert_eq!(
            sm(0x8000000000000001, 0, 0, 0),
            vec![range(0), range(63)]
        );
        assert_eq!(
            sm(0x8000000000000001, 0x8000000000000001, 0, 0),
            vec![range(0), range(63), range(64), range(127)]
        );
        assert_eq!(
            sm(0xC000000000000007, 0x1F000001, 0, 0),
            vec![range(0..=2), range(62..=63), range(64), range(88..=92)]
        );
    }
}
