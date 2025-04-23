/// This trait adds some functionality needed by [`regr::Range`] type.
///
/// It could be based on [`std::iter::Step`] trait, but it is experimental and
/// is not available within Rust's stable standard library.
pub trait Symbol: Sized + Copy {
    /// Returns the number of steps required to get from `self` to `other` or
    /// vice versa.
    fn steps_between(&self, other: Self) -> usize;

    /// Returns the value that would be obtained by taking the _successor_ of
    /// `self` count times.
    ///
    /// Returns `None` if the value is out of `Self`'s valid range.
    fn forward(&self, count: usize) -> Option<Self>;

    /// Returns the value that would be obtained by taking the _predecessor_ of
    /// `self` count times.
    ///
    /// Returns `None` if the value is out of `Self`'s valid range.
    fn backward(&self, count: usize) -> Option<Self>;

    /// Checks if there is one step between the specified two values.
    fn adjoins(&self, other: Self) -> bool {
        self.steps_between(other) == 1
    }

    /// Prints a symbol using character corresponding to its value if it is
    /// possible.
    fn format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    /// Returns a wrapper for symbol that can be used for more human legible
    /// formatting.
    fn formatted(self) -> SymbolDisplay;
}

impl Symbol for u8 {
    fn steps_between(&self, other: Self) -> usize {
        self.abs_diff(other).into()
    }

    fn forward(&self, count: usize) -> Option<Self> {
        if let Ok(count) = Self::try_from(count) {
            self.checked_add(count)
        } else {
            None
        }
    }

    fn backward(&self, count: usize) -> Option<Self> {
        if let Ok(count) = Self::try_from(count) {
            self.checked_sub(count)
        } else {
            None
        }
    }

    /// Prints a symbol using character corresponding to its value if the value
    /// is graphical ASCII character, i.e. within range `U+0021`..=`U+007E`.
    fn format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_ascii_graphic() {
            std::fmt::Debug::fmt(&char::from(*self), f)
        } else {
            std::fmt::Debug::fmt(self, f)
        }
    }

    fn formatted(self) -> SymbolDisplay {
        SymbolDisplay(self)
    }
}

/// A wrapper around a symbol that can be used within formatting strings for
/// better legibility.
///
/// It doesn't have own constructor and shoud be created by [`Symbol::fmtd`]
/// method.
pub struct SymbolDisplay(u8);

impl std::fmt::Display for SymbolDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_ascii_graphic() {
            std::fmt::Display::fmt(&char::from(self.0), f)
        } else {
            std::fmt::Display::fmt(&self.0, f)
        }
    }
}

impl std::fmt::Debug for SymbolDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

macro_rules! reimpl {
    (std::fmt::$trait:ident for $outer_type:ident) => {
        impl std::fmt::$trait for $outer_type {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::fmt::$trait::fmt(&self.0, f)
            }
        }
    };
}

reimpl!(std::fmt::Binary for SymbolDisplay);
reimpl!(std::fmt::Octal for SymbolDisplay);
reimpl!(std::fmt::LowerHex for SymbolDisplay);
reimpl!(std::fmt::UpperHex for SymbolDisplay);
