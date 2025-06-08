use recdc::CodecError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("codec error: {0}")]
    CodecError(#[from] CodecError),

    #[error("empty escape expression is not supported")]
    EmptyEscape,

    #[error("character `{0}` must be escaped with a prior backslash `\\`")]
    EscapeIt(char),

    #[error("invalid hexadecimal {0}")]
    InvalidHex(String),

    #[error("value {0} doesn't make sense here")]
    NonsenseValue(u32),

    #[error("value `{value}` is out of allowable range {range}")]
    OutOfRange { value: String, range: String },

    #[error("escape expression '\\{0}' is not supported")]
    UnsupportedEscape(char),

    #[error("unexpected close bracket `{0}` encountered without open one")]
    UnexpcetedCloseBracket(char),

    #[error("expected that {expected}")]
    UnexpectedCond { expected: String },

    #[error("unexpected end of file within {aborted_expr} expression")]
    UnexpectedEof { aborted_expr: String },

    #[error("expected {expected}, but got {gotten}")]
    UnexpectedToken { gotten: String, expected: String },
}

/// Helper module to facilitate creating new error instances.
pub(crate) mod err {
    use super::{Error, Result};

    pub(crate) fn escape_it<T>(symbol: char) -> Result<T> {
        Err(Error::EscapeIt(symbol))
    }

    pub(crate) fn nonsense_value<T>(value: u32) -> Result<T> {
        Err(Error::NonsenseValue(value))
    }

    pub(crate) fn unexpected_close_bracket<T>(bracket: char) -> Result<T> {
        Err(Error::UnexpcetedCloseBracket(bracket))
    }

    pub(crate) fn unexpected_cond<T>(expected: impl Into<String>) -> Result<T> {
        Err(Error::UnexpectedCond {
            expected: expected.into(),
        })
    }

    pub(crate) fn unexpected_eof<T>(aborted_expr: impl Into<String>) -> Result<T> {
        Err(Error::UnexpectedEof {
            aborted_expr: aborted_expr.into(),
        })
    }

    pub(crate) fn unexpected_token<T>(
        gotten: impl Into<String>,
        expected: impl Into<String>,
    ) -> Result<T> {
        Err(Error::UnexpectedToken {
            gotten: gotten.into(),
            expected: expected.into(),
        })
    }
}
