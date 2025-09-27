use crate::range::Range;
use crate::step::Step;

/// A set of non-overlapping inclusive ranges, stored in increasing order.
#[derive(PartialEq, Eq)]
pub struct RangeList<T> {
    ranges: Vec<Range<T>>,
}

impl<T: PartialOrd> RangeList<T> {
    /// Creates a new range list from the given start and last values.
    #[inline]
    pub fn new(start: T, last: T) -> Self {
        Self {
            ranges: vec![Range::new(start, last)],
        }
    }
}

impl<T> RangeList<T> {
    /// Returns the number of ranges in this range list.
    #[inline]
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Returns `true` if the range list contains no ranges.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Returns a slice of all ranges in this range list.
    ///
    /// The ranges are guaranteed to be non-overlapping and in increasing order.
    #[inline]
    pub fn ranges(&self) -> &[Range<T>] {
        &self.ranges
    }
}

impl<T: Step + Ord> RangeList<T> {
    /// Merges the given range into this range list.
    ///
    /// If the new range overlaps with or is adjacent to existing ranges, they
    /// will be combined into a single range. The range list maintains its
    /// invariant of non-overlapping ranges in increasing order.
    ///
    /// # Arguments
    ///
    /// * `other` - The range to merge into this range list
    pub fn merge(&mut self, other: impl AsRef<Range<T>>) {
        let other = other.as_ref();
        let i = match self
            .ranges
            .binary_search_by(|r| r.start().cmp(&other.start()))
        {
            Ok(index) => {
                self.ranges[index] = self.ranges[index].try_merge(other).unwrap();
                index
            }
            Err(index) if index == self.ranges.len() => {
                if let Some(last_range) = self.ranges.last_mut()
                    && let Some(new_range) = last_range.try_merge(other)
                {
                    *last_range = new_range;
                } else {
                    self.ranges.push(*other);
                }
                return;
            }
            Err(index) if index == 0 => {
                if let Some(next_range) = self.ranges.get_mut(index)
                    && let Some(new_range) = next_range.try_merge(other)
                {
                    *next_range = new_range;
                    index
                } else {
                    self.ranges.insert(index, *other);
                    return;
                }
            }
            Err(index) => {
                if let Some(prev_range) = self.ranges.get_mut(index - 1)
                    && let Some(new_range) = prev_range.try_merge(other)
                {
                    *prev_range = new_range;
                    index - 1
                } else if let Some(next_range) = self.ranges.get_mut(index)
                    && let Some(new_range) = next_range.try_merge(other)
                {
                    *next_range = new_range;
                    index
                } else {
                    self.ranges.insert(index, *other);
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

    /// Removes the given range from this range list.
    ///
    /// Any existing ranges that overlap with the given range will be modified
    /// or removed to exclude the overlapping portions. This may result in
    /// ranges being split into multiple ranges.
    ///
    /// # Arguments
    ///
    /// * `other` - The range to exclude from this range list
    pub fn exclude(&mut self, other: impl AsRef<Range<T>>) {
        if self.is_empty() {
            return;
        }
        let other = other.as_ref();
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
                    index + 1
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

impl<T> std::default::Default for RangeList<T> {
    /// Creates an empty range list.
    #[inline]
    fn default() -> Self {
        Self { ranges: Vec::new() }
    }
}

impl<T> std::convert::From<Range<T>> for RangeList<T> {
    /// Creates a range list containing a single range.
    fn from(range: Range<T>) -> Self {
        Self {
            ranges: vec![range],
        }
    }
}

impl<T, R, I> std::convert::From<I> for RangeList<T>
where
    T: Step + Ord,
    R: AsRef<Range<T>>,
    I: IntoIterator<Item = R>,
{
    /// Creates a range list from an iterator of ranges.
    ///
    /// The ranges will be merged together to maintain the invariant of
    /// non-overlapping ranges in increasing order.
    fn from(iter: I) -> Self {
        let mut set = RangeList::default();
        for range in iter.into_iter() {
            set.merge(range.as_ref());
        }
        set
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl<T: Copy + PartialEq + std::fmt::$trait> std::fmt::$trait for RangeList<T> {
            /// Formats the range list by displaying each range separated by " | ".
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                for (i, range) in self.ranges.iter().enumerate() {
                    std::fmt::$trait::fmt(&range, f)?;
                    if i < self.ranges.len() - 1 {
                        f.write_str(" | ")?;
                    }
                }
                Ok(())
            }
        }
    };
}

impl_fmt!(std::fmt::Debug);
impl_fmt!(std::fmt::Binary);
impl_fmt!(std::fmt::Octal);
impl_fmt!(std::fmt::LowerHex);
impl_fmt!(std::fmt::UpperHex);

impl std::fmt::Display for RangeList<u8> {
    /// Formats the range list for display by showing each range separated by
    /// ` | `.
    ///
    /// This implementation is specialized for `u8` ranges to provide more
    /// readable output for byte ranges.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, range) in self.ranges.iter().enumerate() {
            std::fmt::Display::fmt(&range, f)?;
            if i < self.ranges.len() - 1 {
                f.write_str(" | ")?;
            }
        }
        Ok(())
    }
}
