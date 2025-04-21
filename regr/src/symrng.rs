use crate::error::{Error, err};
use crate::symbl::Symbl;

/// Inclusive range of symbls with invariant `start` is always less or equal to `end`.
///
/// I use this custom structure instead of [`std::ops::RangeInclusive`] because
/// the standrad range implements [`std::iter::Iterator`], and it requires
/// implementing [`std::iter::Step`] that is unstable. Also the standard range
/// uses additional data [`std::iter::Iterator`], but my custom range doesn't.
#[derive(Copy, Clone, PartialEq)]
pub struct SymRng {
    start: Symbl,
    end: Symbl,
}

/// Just a short [`Range`] constructor.
pub fn rng(into_range: impl Into<SymRng>) -> SymRng {
    into_range.into()
}

impl SymRng {
    /// Creates a new range with inclusive bounds from `start` to `end`.
    ///
    /// Panics if `end` is less than `start`.
    pub fn new(start: impl Into<Symbl>, end: impl Into<Symbl>) -> Self {
        let start = start.into();
        let end = end.into();
        assert!(start <= end);
        Self { start, end }
    }

    /// Returns the start position of the range
    #[inline]
    pub fn start(&self) -> Symbl {
        self.start
    }

    /// Returns the end position of the range
    #[inline]
    pub fn end(&self) -> Symbl {
        self.end
    }

    /// Sets a new value of `start` filed.
    ///
    /// Panics if the new `start` is greater than `end`.
    pub fn set_start(&mut self, value: impl Into<Symbl>) {
        let value = value.into();
        assert!(value <= self.end);
        self.start = value
    }

    /// Sets a new value of `end` filed.
    ///
    /// Panics if the new `end` is less than `start`.
    pub fn set_end(&mut self, value: impl Into<Symbl>) {
        let value = value.into();
        assert!(self.start <= value);
        self.end = value
    }

    /// Checks if `self` range is at left of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_left(&self, other: Self) -> bool {
        self.end < other.start
    }

    /// Checks if `self` range is at right of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_right(&self, other: Self) -> bool {
        other.end < self.start
    }

    /// Checks if the two ranges have common elements.
    pub fn intersects(&self, other: Self) -> bool {
        !(self.end < other.start || other.end < self.start)
    }

    /// Checks if the two range have a joint, but not common elements.
    pub fn adjoins(&self, other: Self) -> bool {
        (self.end < other.start && self.end.adjoins(other.start))
            || (other.end < self.start && other.end.adjoins(self.start))
    }

    /// Merges two ranges if they are either intersected or adjoint. Otherwise
    /// returns error message.
    pub fn try_merge(&self, other: Self) -> Result<Self, Error> {
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
    pub fn merge(&self, other: Self) -> Self {
        self.try_merge(other).unwrap_or_else(|e| panic!("{}", e))
    }
}

impl std::convert::From<Symbl> for SymRng {
    fn from(value: Symbl) -> Self {
        Self::new(value, value)
    }
}

impl std::convert::From<u8> for SymRng {
    fn from(value: u8) -> Self {
        Self::new(value, value)
    }
}

impl std::convert::From<std::ops::RangeInclusive<Symbl>> for SymRng {
    fn from(value: std::ops::RangeInclusive<Symbl>) -> Self {
        if value.start() <= value.end() {
            Self::new(*value.start(), *value.end())
        } else {
            Self::new(*value.end(), *value.start())
        }
    }
}

impl std::convert::From<std::ops::RangeInclusive<u8>> for SymRng {
    #[inline]
    fn from(value: std::ops::RangeInclusive<u8>) -> Self {
        let start = value.start().into();
        let end = value.end().into();
        std::convert::From::<std::ops::RangeInclusive::<Symbl>>::from(start..=end)
    }
}

impl std::fmt::Display for SymRng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}", self.start)?;
        if self.start != self.end {
            write!(f, "-{}", self.end)?;
        }
        f.write_str("]")
    }
}

impl std::fmt::Debug for SymRng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}", self.start)?;
        if self.start != self.end {
            write!(f, "-{:?}", self.end)?;
        }
        f.write_str("]")
    }
}
