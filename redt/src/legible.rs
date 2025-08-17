use std::fmt::Write;

/// A trait for types interpreted as symbols in a finite automaton. Just has an
/// additional method for formatting the symbol more human friendly.
pub trait Legible {
    /// Returns a wrapper for symbol that can be used for more human legible
    /// formatting.
    fn display(&self) -> impl std::fmt::Display;
}

/// A wrapper around an integer that can be used within formatting strings for
/// better legibility.
///
/// It doesn't have own constructor and shoud be created by `<u8 as
/// Legible>::display` method.
pub struct ByteLegible(u8);

impl std::fmt::Display for ByteLegible {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if 0x20 <= self.0 && self.0 <= 0x7e {
            write!(f, "'{}'", char::from(self.0))
        } else {
            write!(f, "{:02X}h", self.0)
        }
    }
}

impl Legible for u8 {
    /// Returns a wrapper for symbol that can be used for more human legible
    /// formatting.
    fn display(&self) -> impl std::fmt::Display {
        ByteLegible(*self)
    }
}

/// A wrapper around an byte array that can be used within formatting strings
/// for better legibility.
///
/// It doesn't have own constructor and shoud be created by `<&[u8] as
/// Legible>::display` method.
pub struct ByteArrayLegible<'a>(&'a [u8]);

impl std::fmt::Display for ByteArrayLegible<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('"')?;
        for byte in self.0 {
            if 0x20 <= *byte && *byte <= 0x7e {
                write!(f, "{}", char::from(*byte))?;
            } else {
                write!(f, "\\x{byte:02X}")?;
            }
        }
        f.write_char('"')
    }
}

impl Legible for [u8] {
    /// Returns a wrapper for symbol that can be used for more human legible
    /// formatting.
    fn display(&self) -> impl std::fmt::Display {
        ByteArrayLegible(self)
    }
}
