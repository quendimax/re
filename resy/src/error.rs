use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("escape expression with character {0:?} is not supported")]
    UnsupportedEscape(char),

    #[error("unexpected end of file: {0}")]
    UnexpectedEof(&'static str),
}

/// Inner module to facilitate creating errors and avoid ambiguty with other
/// `Error` types.
pub(crate) mod err {
    use super::*;

    pub(crate) const fn unsupported_escape<T>(c: char) -> Result<T> {
        Err(Error::UnsupportedEscape(c))
    }

    pub(crate) const fn unexpected_eof<T>(msg: &'static str) -> Result<T> {
        Err(Error::UnexpectedEof(msg))
    }
}
