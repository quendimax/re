use crate::range::Range;
use crate::symbol::Symbol;
use smallvec::{SmallVec, smallvec};

/// Edge is a sorted set of nonintersecting symbol ranges, that represents a
/// connection between two states.
#[derive(Clone, Default)]
pub struct Edge<T> {
    ranges: SmallVec<[Range<T>; 2]>,
}

impl<T> Edge<T> {
    /// Creates a new empty Edge.
    pub fn new() -> Self {
        Self {
            ranges: Default::default(),
        }
    }

    /// Returns an iterator over inner range vector in ascendent order.
    pub fn ranges(&self) -> std::slice::Iter<'_, Range<T>> {
        self.ranges.iter()
    }
}

impl<T> std::convert::From<Range<T>> for Edge<T> {
    fn from(range: Range<T>) -> Self {
        Self {
            ranges: smallvec![range],
        }
    }
}

impl<T: PartialOrd + Copy + std::fmt::Debug> Edge<T> {
    /// Push `range` to the end of `self` edge. If `range` is not at the right
    /// of last edge's element, it panics.
    #[inline]
    pub fn push(&mut self, range: impl Into<Range<T>>) {
        let range = range.into();
        assert!(
            self.ranges.last().is_none_or(|v| v.is_at_left(&range)),
            "range {:?} must be at the right of range {:?}",
            range,
            self.ranges.last().unwrap()
        );
        self.ranges.push(range);
    }
}

/// Creates an edge from list of symbols and symbol ranges. If there are
/// intersected ranges, it panics.
#[macro_export]
macro_rules! edge {
    () => (
        $crate::Edge::new()
    );
    ($($into_range:expr),+ $(,)?) => ({
        let mut new_edge = $crate::Edge::new();
        $(
            new_edge.push($into_range);
        )+
        new_edge
    });
}

impl<T> PartialEq for Edge<T>
where
    T: PartialEq + PartialOrd + Copy + Symbol,
{
    fn eq(&self, other: &Self) -> bool {
        let mut self_it = self.ranges.iter();
        let mut other_it = other.ranges.iter();
        loop {
            match (self_it.next(), other_it.next()) {
                (None, None) => return true,
                (Some(_), None) | (None, Some(_)) => return false,
                (Some(mut left), Some(mut right)) => {
                    if left.start() != right.start() {
                        return false;
                    }
                    // check for possible adjoint ranges
                    while left.end() != right.end() {
                        if left.end() > right.end() {
                            let Some(next_right) = other_it.next() else {
                                return false;
                            };
                            if right.end().adjoins(next_right.start()) {
                                right = next_right;
                            } else {
                                return false;
                            }
                        } else {
                            let Some(next_left) = self_it.next() else {
                                return false;
                            };
                            if left.end().adjoins(next_left.start()) {
                                left = next_left;
                            } else {
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<T: PartialEq + PartialOrd + Symbol> Eq for Edge<T> {}

impl<T: std::fmt::Debug + Copy + PartialEq> std::fmt::Debug for Edge<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        for (i, range) in self.ranges.iter().enumerate() {
            if i > 0 {
                f.write_str("|")?;
            }
            range.start().fmt(f)?;
            if range.start() != range.end() {
                f.write_str("-")?;
                range.end().fmt(f)?;
            }
        }
        f.write_str("]")
    }
}

impl<T: PartialOrd + Ord + Symbol> Edge<T> {
    /// Merges a symbol range with this edge.
    ///
    /// If the range cannot be merged with any existing range, it is inserted
    /// into the appropriate position to maintain the edge's sorted order.
    /// Adjacent ranges are merged where possible.
    ///
    /// # Implementation Details
    ///
    /// Range merging is done by finding existing ranges that intersect or
    /// adjoin with the input range and merging them together. For overlapping
    /// ranges, the larger range is split into two parts at their midpoint.
    pub fn merge_range(&mut self, other: &Range<T>) {
        let mut iter = self.ranges.iter_mut().enumerate();
        while let Some((i, mut range)) = iter.next() {
            if range.is_at_left(other) && !range.adjoins(other) {
                continue;
            }
            if let Ok(new_range) = range.try_merge(other) {
                *range = new_range;
                for (_, next_range) in iter.by_ref() {
                    if range.intersects(next_range) {
                        range.set_start(range.start().min(next_range.start()));
                        next_range.set_end(range.end().max(next_range.end()));
                        let min = range.start();
                        let max = next_range.end();
                        let len = min.steps_between(max);
                        let range_end = min.forward(len.div_ceil(2)).unwrap();

                        range.set_end(range_end);
                        next_range.set_start(range_end.forward(1).unwrap());

                        range = next_range;
                    } else {
                        break;
                    }
                }
                return;
            } else {
                self.ranges.insert(i, other.clone());
                return;
            }
        }
        self.ranges.push(other.clone());
    }

    /// Merge the `other` edge with the `self` one.
    ///
    /// **NOTE**: This is not an optimal implemenation. Its complexity is
    /// _O(N*M)_ where _N_ and _M_ are lengths of the specified edges. But it is
    /// possible to get _O(M+N)_.
    pub fn merge(&mut self, other: &Self) {
        for other_range in other.ranges.iter().rev() {
            self.merge_range(other_range);
        }
    }

    /// Folds adjacent symbol ranges in the edge into single ranges where
    /// possible.
    ///
    /// This method iterates through the ranges in reverse order and merges any
    /// adjacent ranges that can be combined. This reduces the total number of
    /// ranges while preserving the logical representation of the edge.
    pub fn fold(&mut self) {
        for i in (1..self.ranges.len()).rev() {
            if let Ok(new_range) = self.ranges[i - 1].try_merge(&self.ranges[i]) {
                self.ranges[i - 1] = new_range;
                self.ranges.remove(i);
            }
        }
    }
}
