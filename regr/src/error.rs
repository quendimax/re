use resy::enc;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("encoder error: {0}")]
    EncoderError(#[from] enc::Error),

    #[error("empty class expression `[]` is not supported")]
    EmptyClass,

    #[error("empty escape expression is not supported")]
    EmptyEscape,

    #[error("character `{0}` must be escaped with a prior backslash `\\`")]
    EscapeIt(char),

    #[error("invalid hexadecimal `{0}`")]
    InvalidHex(Box<str>),

    #[error("value {0} doesn't make sense here")]
    NonsenseValue(u32),

    #[error("value `{value}` is out of allowable range {range}")]
    OutOfRange { value: Box<str>, range: Box<str> },

    #[error("escape expression '\\{0}' is not supported")]
    UnsupportedEscape(char),

    #[error("unexpected close bracket `{0}` encountered without open one")]
    UnexpcetedCloseBracket(char),

    #[error("expected that {condition}")]
    UnexpectedCond { condition: Box<str> },

    #[error("unexpected end of file within {aborted_expr} expression")]
    UnexpectedEof { aborted_expr: Box<str> },

    #[error("expected {expected}, but got '{gotten}'")]
    UnexpectedToken {
        gotten: Box<str>,
        expected: Box<str>,
    },
}

/// Helper module to facilitate creating new error instances.
pub(crate) mod err {
    use super::{Error, Result};

    pub(crate) fn escape_it<T>(symbol: char) -> Result<T> {
        Err(Error::EscapeIt(symbol))
    }

    #[inline(never)]
    pub(crate) fn invalid_hex<T>(got: impl Into<String>) -> Result<T> {
        Err(Error::InvalidHex(got.into().into_boxed_str()))
    }

    pub(crate) fn nonsense_value<T>(value: u32) -> Result<T> {
        Err(Error::NonsenseValue(value))
    }

    #[inline(never)]
    pub(crate) fn out_of_range<T>(
        value: impl Into<Box<str>>,
        range: impl Into<Box<str>>,
    ) -> Result<T> {
        Err(Error::OutOfRange {
            value: value.into(),
            range: range.into(),
        })
    }

    pub(crate) fn unexpected_close_bracket<T>(bracket: char) -> Result<T> {
        Err(Error::UnexpcetedCloseBracket(bracket))
    }

    #[inline(never)]
    pub(crate) fn unexpected_cond<T>(expected: impl Into<Box<str>>) -> Result<T> {
        Err(Error::UnexpectedCond {
            condition: expected.into(),
        })
    }

    #[inline(never)]
    pub(crate) fn unexpected_eof<T>(aborted_expr: impl Into<Box<str>>) -> Result<T> {
        Err(Error::UnexpectedEof {
            aborted_expr: aborted_expr.into(),
        })
    }

    #[inline(never)]
    pub(crate) fn unexpected_token<T>(
        gotten: impl Into<String>,
        expected: impl Into<String>,
    ) -> Result<T> {
        Err(Error::UnexpectedToken {
            gotten: gotten.into().into_boxed_str(),
            expected: expected.into().into_boxed_str(),
        })
    }
}
