/// The `regr` specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// A try to create a Unicode point out of the possible range. `U+0000`
    /// through `U+10FFFF`.
    InvalidCodepoint(u32),

    /// A try to create a symbol out of the possible [`Symbol`] range.
    ///
    /// I use `i64` instead of `u32` to show an invalid values lower than `0`.
    InvalidSymbol(i64),

    /// A try to merge delimited ranges
    MergeDelimitedRanges,

    /// This regex feature is not implemented.
    UnsupportedFeature(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            InvalidCodepoint(codepoint) => {
                write!(
                    f,
                    "can't create a char with invalid codepoint {}",
                    codepoint
                )
            }
            InvalidSymbol(value) => {
                write!(f, "can't create a symbol with invalid value {}", value)
            }
            MergeDelimitedRanges => f.write_str("can't merge delimited ranges"),
            UnsupportedFeature(msg) => f.write_str(msg),
        }
    }
}

/// Inner module to facilitate creating errors and avoid ambiguty with other
/// `Error` types.
pub mod err {
    use super::*;

    pub const fn invalid_codepoint<T>(codepoint: u32) -> Result<T> {
        Err(Error::InvalidCodepoint(codepoint))
    }

    pub const fn invalid_symbol<T>(value: i64) -> Result<T> {
        Err(Error::InvalidSymbol(value))
    }

    pub const fn merge_delimited_ranges<T>() -> Result<T> {
        Err(Error::MergeDelimitedRanges)
    }

    pub fn unsupported_feature<T>(msg: &'static str) -> Result<T> {
        Err(Error::UnsupportedFeature(msg))
    }
}
