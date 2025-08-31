use crate::range::Range;
use crate::step::Step;

pub struct RangeSet<T> {
    ranges: Vec<Range<T>>,
}

impl<T> RangeSet<T> {
    #[inline]
    pub fn new() -> Self {
        RangeSet { ranges: Vec::new() }
    }
}

impl<T: Step + Copy + Ord> RangeSet<T> {
    pub fn merge(&mut self, other: Range<T>) {
        let index = match self
            .ranges
            .binary_search_by(|r| r.start().cmp(&other.start()))
        {
            Ok(index) => {
                if let Some(new_range) = self.ranges[index].try_merge(&other) {
                    self.ranges[index] = new_range;
                }
                index
            }
            Err(index) => {
                self.ranges.insert(index, other);
                index
            }
        };
        let mut drain_range = index + 1..index + 1;
        for j in index + 1..self.ranges.len() {
            if let Some(new_range) = self.ranges[index].try_merge(&self.ranges[j]) {
                self.ranges[index] = new_range;
                drain_range = index + 1..j + 1;
            } else {
                break;
            }
        }
        drop(self.ranges.drain(drain_range));
    }
}

impl<T> std::default::Default for RangeSet<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::convert::From<Range<T>> for RangeSet<T> {
    fn from(range: Range<T>) -> Self {
        Self {
            ranges: vec![range],
        }
    }
}
