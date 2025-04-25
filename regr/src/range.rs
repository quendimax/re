use crate::error::{Error, err};
use crate::symbol::Symbol;
use std::fmt::Write;

/// Inclusive range of symbls with invariant `start` is always less or equal to `end`.
///
/// I use this custom structure instead of [`std::ops::RangeInclusive`] because
/// the standrad range implements [`std::iter::Iterator`], and it requires
/// implementing [`std::iter::Step`] that is unstable. Also the standard range
/// uses additional data [`std::iter::Iterator`], but my custom range doesn't.
#[derive(Copy, Clone, PartialEq)]
pub struct Range {
    start: u8,
    end: u8,
}

/// Just a short [`Range`] constructor.
#[inline]
pub fn range(range: impl Into<Range>) -> Range {
    range.into()
}

impl Range {
    /// Creates a new range with inclusive bounds from `start` to `end`. If
    /// `start` is greater than `end`, ther are swapped.
    pub fn new(start: u8, end: u8) -> Self {
        if start <= end {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
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
        self.end() < other.start()
    }

    /// Checks if `self` range is at right of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_right(self, other: Self) -> bool {
        other.end() < self.start()
    }

    /// Checks if the two ranges have common elements.
    pub fn intersects(self, other: Self) -> bool {
        !(self.end() < other.start() || other.end() < self.start())
    }

    /// Checks if the two range have a joint, but not common elements.
    pub fn adjoins(self, other: Self) -> bool {
        (self.end() < other.start() && self.end.adjoins(other.start()))
            || (other.end() < self.start() && other.end().adjoins(self.start()))
    }

    /// Merges two ranges if they are either intersected or adjoint. Otherwise
    /// returns an error.
    pub fn try_merge(self, other: Self) -> Result<Self, Error> {
        if self.intersects(other) || self.adjoins(other) {
            Ok(Self {
                start: self.start().min(other.start()),
                end: self.end().max(other.end()),
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

impl std::convert::From<u8> for Range {
    #[inline]
    fn from(value: u8) -> Self {
        Self {
            start: value,
            end: value,
        }
    }
}

impl std::convert::From<std::ops::RangeInclusive<u8>> for Range {
    #[inline]
    fn from(value: std::ops::RangeInclusive<u8>) -> Self {
        Self::new(*value.start(), *value.end())
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}", self.start().formatted())?;
        if self.start() != self.end() {
            write!(f, "-{}", self.end().formatted())?;
        }
        f.write_str("]")
    }
}

impl std::fmt::Debug for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}", self.start().formatted())?;
        if self.start() != self.end() {
            write!(f, "-{:?}", self.end().formatted())?;
        }
        f.write_str("]")
    }
}

macro_rules! reimpl {
    (std::fmt::$trait:ident for $outer_type:ident) => {
        impl std::fmt::$trait for $outer_type {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_char('[')?;
                std::fmt::$trait::fmt(&self.start(), f)?;
                if self.start() != self.end() {
                    f.write_char('-')?;
                    std::fmt::$trait::fmt(&self.end(), f)?;
                }
                f.write_char(']')
            }
        }
    };
}

reimpl!(std::fmt::Binary for Range);
reimpl!(std::fmt::Octal for Range);
reimpl!(std::fmt::LowerHex for Range);
reimpl!(std::fmt::UpperHex for Range);
