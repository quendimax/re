use crate::str::lexis::Token;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type Error = Box<ErrorKind>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ErrorKind {
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
            UnexpectedTokenError { found_token, .. } => found_token.span(),
        }
    }
}

/// Helper module to facilitate creating new error instances.
pub(crate) mod err {
    use super::*;

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
