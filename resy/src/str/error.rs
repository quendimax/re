use crate::str::lexis::Token;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<ErrorKind>;

#[derive(Error, Debug, PartialEq)]
pub enum ErrorKind {
    #[error("encoder error: {cause}")]
    EncoderError {
        #[source]
        cause: renc::Error,
        bad_token: Token,
    },

    #[error("expected {expected}, but found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        found_token: Token,
    },

    #[error("integer `{found_int}` is out of range of `u32`")]
    IntOverflow {
        found_int: String,
        span: std::ops::Range<usize>,
    },
}

impl ErrorKind {
    pub fn error_span(&self) -> std::ops::Range<usize> {
        use ErrorKind::*;
        match self {
            EncoderError { bad_token, .. } => bad_token.span(),
            UnexpectedToken { found_token, .. } => found_token.span(),
            IntOverflow { span, .. } => span.clone(),
        }
    }
}

/// Helper module to facilitate creating new error instances.
pub(crate) mod err {
    use super::*;

    pub(crate) fn encoder_error<T>(cause: renc::Error, bad_token: Token) -> Result<T> {
        Err(Box::new(ErrorKind::EncoderError { cause, bad_token }))
    }

    pub(crate) fn unexpected_token<T>(
        expected: String,
        found_spell: String,
        found_token: Token,
    ) -> Result<T> {
        Err(Box::new(ErrorKind::UnexpectedToken {
            expected,
            found: found_spell,
            found_token,
        }))
    }

    pub(crate) fn int_overflow<T>(found_int: String, span: std::ops::Range<usize>) -> Result<T> {
        Err(Box::new(ErrorKind::IntOverflow { found_int, span }))
    }
}
