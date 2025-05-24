/// This trait adds some functionality needed by [`regr::Range`] type.
///
/// It could be based on [`std::iter::Step`] trait, but it is experimental and
/// is not available within Rust's stable standard library.
pub trait Symbol: Sized + Copy + crate::private::Sealed {
    /// Returns the number of steps required to get from `self` to `other` or
    /// vice versa.
    fn steps_between(self, other: Self) -> usize;

    /// Returns the value that would be obtained by taking the _successor_ of
    /// `self` count times.
    ///
    /// Returns `None` if the value is out of `Self`'s valid range.
    fn forward(self, count: usize) -> Option<Self>;

    /// Returns the value that would be obtained by taking the _predecessor_ of
    /// `self` count times.
    ///
    /// Returns `None` if the value is out of `Self`'s valid range.
    fn backward(self, count: usize) -> Option<Self>;

    /// Checks if there is one step between the specified two values.
    fn adjoins(self, other: Self) -> bool {
        self.steps_between(other) == 1
    }

    /// Returns a wrapper for symbol that can be used for more human legible
    /// formatting.
    fn display(self) -> SymbolDisplay;
}

impl crate::private::Sealed for u8 {}

impl Symbol for u8 {
    fn steps_between(self, other: Self) -> usize {
        self.abs_diff(other).into()
    }

    fn forward(self, count: usize) -> Option<Self> {
        if let Ok(count) = Self::try_from(count) {
            self.checked_add(count)
        } else {
            None
        }
    }

    fn backward(self, count: usize) -> Option<Self> {
        if let Ok(count) = Self::try_from(count) {
            self.checked_sub(count)
        } else {
            None
        }
    }

    /// Returns a wrapper for symbol that can be used for more human legible
    /// formatting.
    fn display(self) -> SymbolDisplay {
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
        if 0x20 <= self.0 && self.0 <= 0x7e {
            write!(f, "'{}'", char::from(self.0))
        } else {
            write!(f, "{:02X}h", self.0)
        }
    }
}

/// A special symbol out of the language alphabet that represents automaton's
/// transition with no real symbol.
pub struct Epsilon;

macro_rules! impl_fmt {
    (std::fmt::$trait:ident for $type:ident) => {
        impl std::fmt::$trait for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("Epsilon")
            }
        }
    };
}

impl_fmt!(std::fmt::Display for Epsilon);
impl_fmt!(std::fmt::Debug for Epsilon);
impl_fmt!(std::fmt::Binary for Epsilon);
impl_fmt!(std::fmt::Octal for Epsilon);
impl_fmt!(std::fmt::LowerHex for Epsilon);
impl_fmt!(std::fmt::UpperHex for Epsilon);
