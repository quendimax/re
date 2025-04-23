use crate::symbl::{Symbl, symbl};
use crate::symrng::SymRng;
use i256::u256;

pub struct SymSet(u256);

impl SymSet {
    /// Creats a new empty symbl set.
    pub fn new(bitmap: [u64; 4]) -> Self {
        SymSet(u256::from_le_u64(bitmap))
    }

    /// Returns an iterator over symbls.
    pub fn symbls(&self) -> SymblIter {
        todo!()
    }

    /// Returns an iterator over inner ranges in ascendent order.
    pub fn ranges(&self) -> RangeIter {
        RangeIter {
            bitmap: self.0,
            already_shifted: 0,
        }
    }

    /// Merges the `other` set into `self` one.
    pub fn merge(&mut self, other: &Self) {
        self.0 |= other.0;
    }
}

pub struct SymblIter<'a> {
    value: &'a u256,
    index: u32,
}

impl std::iter::Iterator for SymblIter<'_> {
    type Item = Symbl;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < u256::BITS {
            let mut mask = u256::from_le_u64([1, 0, 0, 0]);
            mask <<= self.index;
            self.index += 1;
            if (*self.value & mask) != 0u64.into() {
                let symbl_value = self.index - 1;
                return Some(symbl(symbl_value as u8));
            }
        }
        None
    }
}

pub struct RangeIter {
    bitmap: u256,
    already_shifted: u32,
}

impl std::iter::Iterator for RangeIter {
    type Item = SymRng;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bitmap != 0u64.into() {
            let trail_zeros = self.bitmap.trailing_zeros();
            self.bitmap >>= trail_zeros;

            let trail_ones = self.bitmap.trailing_ones();
            self.bitmap = self.bitmap.checked_shr(trail_ones).unwrap_or(0u64.into());

            let start = self.already_shifted + trail_zeros;
            let end = start + (trail_ones - 1);

            self.already_shifted += trail_zeros + trail_ones;
            return Some(SymRng::new_unchecked(start as u8, end as u8));
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
    fn symset_ranges() {
        let map = |a: u64, b: u64, c: u64, d: u64| {
            let map = SymSet::new([a, b, c, d]);
            dbg!(map.0);
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
                vec![rng(0, 255)]
            );
            assert_eq!(map(1, 0, 0, 0), vec![rng(0, 0)]);
            assert_eq!(
                map(0x8000000000000001, 0, 0, 0),
                vec![rng(0, 0), rng(63, 63)]
            );
            assert_eq!(
                map(0x8000000000000001, 0x8000000000000001, 0, 0),
                vec![rng(0, 0), rng(63, 64), rng(127, 127)]
            );
            assert_eq!(
                map(0xC000000000000007, 0x1F000001, 0, 0),
                vec![rng(0, 2), rng(62, 64), rng(88, 92)]
            );
        }
    }
}
