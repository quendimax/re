use crate::error::{Error, err};
use crate::symbol::Symbol;
use std::num::NonZeroU8;

/// Inclusive range of symbls with invariant `start` is always less or equal to `end`.
///
/// I use this custom structure instead of [`std::ops::RangeInclusive`] because
/// the standrad range implements [`std::iter::Iterator`], and it requires
/// implementing [`std::iter::Step`] that is unstable. Also the standard range
/// uses additional data [`std::iter::Iterator`], but my custom range doesn't.
#[derive(Copy, Clone, PartialEq)]
pub struct SymRng {
    start: u8,
    end: u8,
}

/// Just a short [`Range`] constructor.
#[inline]
pub fn rng(start: u8, end: u8) -> SymRng {
    SymRng::new(start, end)
}

impl SymRng {
    /// Creates a new range with inclusive bounds from `start` to `end`.
    ///
    /// Panics if `end` is less than `start`.
    pub fn new(start: u8, end: u8) -> Self {
        assert!(
            start <= end,
            "`start` {start} can't be greater than `end` {end}"
        );
        Self { start, end }
    }

    /// Creates a new range with inclusive bounds from `start` to `end`.
    ///
    /// It expects that `start <= end`.
    #[inline]
    pub fn new_unchecked(start: u8, end: u8) -> Self {
        Self { start, end }
    }

    /// Returns the start position of the range.
    #[inline]
    pub fn start(self) -> u8 {
        self.start
    }

    /// Returns the end position of the range.
    #[inline]
    pub fn end(self) -> u8 {
        self.end
    }

    /// Returns the range's length.
    #[inline]
    pub fn len(self) -> u8 {
        self.end - self.start + 1
    }

    /// Sets a new value of the `start` field.
    ///
    /// Panics if the new `start` is greater than `end`.
    pub fn set_start(&mut self, new_start: u8) {
        let end = self.end;
        assert!(
            new_start <= end,
            "new `start` {new_start} can't be greater than `end` {end}"
        );
        self.start = new_start
    }

    /// Sets a new value of `end` filed.
    ///
    /// Panics if the new `end` is lesser than `start`.
    pub fn set_end(&mut self, new_end: u8) {
        let start = self.start;
        assert!(
            start <= new_end,
            "new `end` {new_end} can't be lesser than `start` {start}"
        );
        self.end = new_end
    }

    /// Checks if `self` range is at left of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_left(self, other: Self) -> bool {
        self.end() < other.start
    }

    /// Checks if `self` range is at right of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_right(self, other: Self) -> bool {
        other.end() < self.start
    }

    /// Checks if the two ranges have common elements.
    pub fn intersects(self, other: Self) -> bool {
        !(self.end < other.start || other.end < self.start)
    }

    /// Checks if the two range have a joint, but not common elements.
    pub fn adjoins(self, other: Self) -> bool {
        (self.end < other.start && self.end.adjoins(other.start))
            || (other.end < self.start && other.end.adjoins(self.start))
    }

    /// Merges two ranges if they are either intersected or adjoint. Otherwise
    /// returns an error.
    pub fn try_merge(self, other: Self) -> Result<Self, Error> {
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
    pub fn merge(self, other: Self) -> Self {
        self.try_merge(other).unwrap_or_else(|e| panic!("{}", e))
    }
}

impl std::convert::From<u8> for SymRng {
    #[inline]
    fn from(value: u8) -> Self {
        Self {
            start: value,
            end: value,
        }
    }
}

impl std::convert::From<std::ops::RangeInclusive<u8>> for SymRng {
    fn from(value: std::ops::RangeInclusive<u8>) -> Self {
        if value.start() <= value.end() {
            Self {
                start: *value.start(),
                end: *value.end(),
            }
        } else {
            Self {
                start: *value.end(),
                end: *value.start(),
            }
        }
    }
}

impl std::fmt::Display for SymRng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}", self.start.formatted())?;
        if self.start != self.end {
            write!(f, "-{}", self.end.formatted())?;
        }
        f.write_str("]")
    }
}

impl std::fmt::Debug for SymRng {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}", self.start.formatted())?;
        if self.start != self.end {
            write!(f, "-{:?}", self.end.formatted())?;
        }
        f.write_str("]")
    }
}
