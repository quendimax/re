use crate::legible::Legible;
use crate::step::Step;
use std::fmt::Write;

/// Inclusive range of symbols with invariant `start` is always less or equal to `end`.
///
/// I use this custom structure instead of [`std::ops::RangeInclusive`] because
/// the standrad range implements [`std::iter::Iterator`], and it requires
/// implementing [`std::iter::Step`] that is unstable. Also the standard range
/// uses additional data [`std::iter::Iterator`], but my custom range doesn't.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Range<T> {
    start: T,
    last: T,
}

/// Just a short [`Range`] constructor.
#[inline]
pub fn range<T: PartialOrd>(start: impl Into<T>, last: impl Into<T>) -> Range<T> {
    let start = start.into();
    let last = last.into();
    Range::new(start, last)
}

impl<T: PartialOrd> Range<T> {
    /// Creates a new range with inclusive bounds from `start` to `end`. If
    /// `start` is greater than `end`, they are swapped.
    pub fn new(start: T, last: T) -> Self {
        if start <= last {
            Self { start, last }
        } else {
            Self {
                start: last,
                last: start,
            }
        }
    }
}

impl<T: PartialOrd> Range<T> {
    /// Creates a new range with inclusive bounds from `start` to `end`.
    ///
    /// It expects that `start <= end`. Otherwise it breaks `Span`'s invariant.
    /// Although it is still safe in terms of Rust.
    #[inline]
    pub fn new_unchecked(start: T, last: T) -> Self {
        debug_assert!(start <= last);
        Self { start, last }
    }

    /// Creats a new range with inclusive bounds from `start` and `last`.
    ///
    /// If `start` >= `end`, returns `None`.
    #[inline]
    pub fn new_checked(start: T, last: T) -> Option<Self> {
        if start <= last {
            Some(Self { start, last })
        } else {
            None
        }
    }
}

impl<T> Range<T> {
    /// Creates a new range with inclusive bounds from `start` to `end`.
    ///
    /// It expects that `start <= end`. Otherwise it breaks `Span`'s invariant.
    /// Although it is still safe in terms of Rust.
    #[inline]
    pub const fn new_unchecked_const(start: T, last: T) -> Self {
        Self { start, last }
    }
}

impl<T: Copy> Range<T> {
    /// Returns the start position of the range.
    #[inline]
    pub fn start(&self) -> T {
        self.start
    }

    /// Returns the end position of the range.
    #[inline]
    pub fn last(&self) -> T {
        self.last
    }
}

impl<T: Step> Range<T> {
    /// Returns the width of the range. If width is greater than `T::MAX`, it
    /// returns `None`.
    #[inline]
    pub fn width(&self) -> Option<T> {
        self.last.steps_between(self.start).forward(1)
    }
}

impl<T: Copy + PartialOrd + std::fmt::Debug> Range<T> {
    /// Sets a new value of the `start` field.
    ///
    /// Panics if the new `start` is greater than `end`.
    pub fn set_start(&mut self, new_start: T) {
        let last = self.last;
        assert!(
            new_start <= last,
            "new `start` {new_start:?} can't be greater than `end` {last:?}"
        );
        self.start = new_start
    }

    /// Sets a new value of `end` filed.
    ///
    /// Panics if the new `end` is lesser than `start`.
    pub fn set_end(&mut self, new_end: T) {
        let start = self.start;
        assert!(
            start <= new_end,
            "new `end` {new_end:?} can't be lesser than `start` {start:?}"
        );
        self.last = new_end
    }
}

impl<T: Copy + PartialOrd> Range<T> {
    /// Checks if `self` range is at left of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_left(&self, other: &Self) -> bool {
        self.last() < other.start()
    }

    /// Checks if `self` range is at right of `other`, and they don't have
    /// intersections (but can be joint).
    #[inline]
    pub fn is_at_right(&self, other: &Self) -> bool {
        other.last() < self.start()
    }

    /// Checks if the two ranges have common elements.
    pub fn intersects(&self, other: &Self) -> bool {
        !(self.last() < other.start() || other.last() < self.start())
    }

    /// Checks if `self` range contains `other` range.
    pub fn contains(&self, other: &Self) -> bool {
        self.start() <= other.start() && other.last() <= self.last()
    }
}

impl<T: Step + PartialOrd> Range<T> {
    /// Checks if the two ranges have a joint, but not common elements.
    pub fn adjoins(&self, other: &Self) -> bool {
        (self.last() < other.start() && self.last.adjoins(other.start()))
            || (other.last() < self.start() && other.last().adjoins(self.start()))
    }
}

impl<T: Step + Ord> Range<T> {
    /// Merges two ranges if they are either intersected or adjoint. Otherwise
    /// returns an error.
    pub fn try_merge(&self, other: &Self) -> Option<Self> {
        if self.intersects(other) || self.adjoins(other) {
            Some(Self {
                start: self.start().min(other.start()),
                last: self.last().max(other.last()),
            })
        } else {
            None
        }
    }
}

impl<T: Step + Ord + std::fmt::Debug> Range<T> {
    /// Merges two ranges if they are either intersected or adjoint. Otherwise it
    /// panics.
    pub fn merge(&self, other: &Self) -> Self {
        self.try_merge(other)
            .unwrap_or_else(|| panic!("can't merge ranges {self:?} and {other:?}"))
    }
}

impl<T> AsRef<Range<T>> for Range<T> {
    fn as_ref(&self) -> &Range<T> {
        self
    }
}

impl<T: Copy> std::convert::From<T> for Range<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new_unchecked_const(value, value)
    }
}

impl<T: Copy + PartialOrd> std::convert::From<std::ops::RangeInclusive<T>> for Range<T> {
    #[inline]
    fn from(value: std::ops::RangeInclusive<T>) -> Self {
        Self::new(*value.start(), *value.end())
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl<T: Copy + PartialEq + std::fmt::$trait> std::fmt::$trait for Range<T> {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::$trait::fmt(&self.start(), f)?;
                if self.start() != self.last() {
                    f.write_char('-')?;
                    std::fmt::$trait::fmt(&self.last(), f)?;
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

impl Legible for Range<u8> {
    fn display(&self) -> impl std::fmt::Display {
        ByteRangeLegible(*self)
    }
}

pub struct ByteRangeLegible(Range<u8>);

impl std::fmt::Display for ByteRangeLegible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.start().display())?;
        if self.0.start() != self.0.last() {
            write!(f, "-{}", self.0.last().display())?;
        }
        Ok(())
    }
}
