use crate::error::{Error, err};
use crate::symbol::Symbol;

/// Inclusive range of symbols (i.e. `start` is always less or equal to `end`).
///
/// I use this custom structure instead of [`std::ops::RangeInclusive`] because
/// the standrad range implements [`std::iter::Iterator`], but it requires
/// implementing [`std::iter::Step`] that is unstable. Also the standard range
/// uses additional data [`std::iter::Iterator`], but my custom range doesn't.
#[derive(Clone, PartialEq)]
pub struct Range<T> {
    start: T,
    end: T,
}

impl<T: PartialOrd> Range<T> {
    /// Creates a new range with inclusive bounds from `start` to `end`.
    ///
    /// Panics if `end` is less than `start`.
    pub fn new(start: T, end: T) -> Self {
        assert!(start <= end);
        Self { start, end }
    }
}

impl<T: Copy> Range<T> {
    /// Returns the start position of the range
    pub fn start(&self) -> T {
        self.start
    }

    /// Returns the end position of the range
    pub fn end(&self) -> T {
        self.end
    }
}

impl<T: PartialOrd> Range<T> {
    /// Sets a new value of `start` filed.
    ///
    /// Panics if the new `start` is greater than `end`.
    pub fn set_start(&mut self, value: T) {
        assert!(value <= self.end);
        self.start = value
    }

    /// Sets a new value of `end` filed.
    ///
    /// Panics if the new `end` is less than `start`.
    pub fn set_end(&mut self, value: T) {
        assert!(self.start <= value);
        self.end = value
    }
}

impl<T: Copy + PartialOrd> std::convert::From<T> for Range<T> {
    fn from(value: T) -> Self {
        Self::new(value, value)
    }
}

impl<T> std::convert::From<std::ops::RangeInclusive<T>> for Range<T>
where
    T: Copy + PartialOrd,
{
    fn from(value: std::ops::RangeInclusive<T>) -> Self {
        if value.start() <= value.end() {
            Self::new(*value.start(), *value.end())
        } else {
            Self::new(*value.end(), *value.start())
        }
    }
}

/// Just a short [`Range`] constructor.
pub fn range<T>(into_range: impl Into<Range<T>>) -> Range<T> {
    into_range.into()
}

impl<T: PartialOrd> Range<T> {
    /// Checks if `self` range is at left of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_left(&self, other: &Self) -> bool {
        self.end < other.start
    }

    /// Checks if `self` range is at right of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_right(&self, other: &Self) -> bool {
        other.end < self.start
    }

    /// Checks if the two ranges have common elements.
    pub fn intersects(&self, other: &Self) -> bool {
        !(self.end < other.start || other.end < self.start)
    }
}

impl<T: Symbol + PartialOrd> Range<T> {
    /// Checks if the two range have a joint, but not common elements.
    pub fn adjoins(&self, other: &Self) -> bool {
        (self.end < other.start && self.end.adjoins(other.start))
            || (other.end < self.start && other.end.adjoins(self.start))
    }
}

impl<T: Symbol + Ord> Range<T> {
    /// Merges two ranges if they are either intersected or adjoint. Otherwise
    /// returns error message.
    pub fn try_merge(&self, other: &Self) -> Result<Self, Error> {
        if self.intersects(other) || self.adjoins(other) {
            Ok(Self {
                start: self.start.min(other.start),
                end: self.end.max(other.end),
            })
        } else {
            err::merge_delimited_ranges()
        }
    }

    /// Merges two ranges if they are either intersected or adjoint. Otherwise
    /// it panics.
    pub fn merge(&self, other: &Self) -> Self {
        self.try_merge(other).unwrap_or_else(|e| panic!("{}", e))
    }
}

impl<T: std::fmt::Debug + PartialEq> std::fmt::Debug for Range<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        self.start.fmt(f)?;
        if self.start != self.end {
            f.write_str("-")?;
            self.end.fmt(f)?;
        }
        f.write_str("]")
    }
}
