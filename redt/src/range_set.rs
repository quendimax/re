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

    #[inline]
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    #[inline]
    pub fn ranges(&self) -> &[Range<T>] {
        &self.ranges
    }
}

impl<T: Step + Copy + Ord> RangeSet<T> {
    pub fn merge(&mut self, other: Range<T>) {
        let i = match self
            .ranges
            .binary_search_by(|r| r.start().cmp(&other.start()))
        {
            Ok(index) => {
                self.ranges[index] = self.ranges[index].try_merge(&other).unwrap();
                index
            }
            Err(index) if index == self.ranges.len() => {
                if let Some(last_range) = self.ranges.last_mut()
                    && let Some(new_range) = last_range.try_merge(&other)
                {
                    *last_range = new_range;
                } else {
                    self.ranges.push(other);
                }
                return;
            }
            Err(index) if index == 0 => {
                if let Some(next_range) = self.ranges.get_mut(index)
                    && let Some(new_range) = next_range.try_merge(&other)
                {
                    *next_range = new_range;
                    index
                } else {
                    self.ranges.insert(index, other);
                    return;
                }
            }
            Err(index) => {
                if let Some(prev_range) = self.ranges.get_mut(index - 1)
                    && let Some(new_range) = prev_range.try_merge(&other)
                {
                    *prev_range = new_range;
                    index - 1
                } else if let Some(next_range) = self.ranges.get_mut(index)
                    && let Some(new_range) = next_range.try_merge(&other)
                {
                    *next_range = new_range;
                    index
                } else {
                    self.ranges.insert(index, other);
                    return;
                }
            }
        };
        let mut to_remove = 0;
        for j in i + 1..self.ranges.len() {
            if let Some(new_range) = self.ranges[i].try_merge(&self.ranges[j]) {
                self.ranges[i] = new_range;
                to_remove += 1;
            } else {
                break;
            }
        }
        if to_remove > 0 {
            drop(self.ranges.drain(i + 1..i + 1 + to_remove));
        }
    }

    pub fn exclude(&mut self, other: Range<T>) {
        if self.is_empty() {
            return;
        }
        let start = match self
            .ranges
            .binary_search_by(|r| r.last().cmp(&other.start())) // start <= last
        {
            Ok(index) => {
                self.ranges[index] = Range::new_unchecked(
                    self.ranges[index].start(),
                    other.start().backward(1).unwrap(),
                );
                index + 1
            }
            Err(index) if index == self.ranges.len() => return,
            Err(index) => {
                let range = self.ranges[index];
                if other.start() <= range.start() {
                    index
                } else if other.last() < self.ranges[index].last() {
                    self.ranges[index] =
                        Range::new_unchecked(range.start(), other.start().backward(1).unwrap());
                    let new_range =
                        Range::new_unchecked(other.last().forward(1).unwrap(), range.last());
                    self.ranges.insert(index + 1, new_range);
                    return;
                } else {
                    self.ranges[index] =
                        Range::new_unchecked(range.start(), other.start().backward(1).unwrap());
                    index + 1
                }
            }
        };

        let end = match self
            .ranges
            .binary_search_by(|r| r.start().cmp(&other.last()))
        {
            Ok(index) => {
                self.ranges[index] = Range::new_unchecked(
                    other.last().forward(1).unwrap(),
                    self.ranges[index].last(),
                );
                index
            }
            Err(index) => {
                if index == 0 {
                    return;
                }
                let index = index - 1;
                if self.ranges[index].last() <= other.last() {
                    index
                } else {
                    self.ranges[index] = Range::new_unchecked(
                        other.last().forward(1).unwrap(),
                        self.ranges[index].last(),
                    );
                    index
                }
            }
        };

        if start < end {
            drop(self.ranges.drain(start..end));
        }
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
