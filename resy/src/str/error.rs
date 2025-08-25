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
    UnexpectedTokenError {
        expected: String,
        found: String,
        found_token: Token,
    },
}

impl ErrorKind {
    pub fn error_span(&self) -> std::ops::Range<usize> {
        use ErrorKind::*;
        match self {
            EncoderError { bad_token, .. } => bad_token.span(),
            UnexpectedTokenError { found_token, .. } => found_token.span(),
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
        Err(Box::new(ErrorKind::UnexpectedTokenError {
            expected,
            found: found_spell,
            found_token,
        }))
    }
}
