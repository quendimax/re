use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq)]
pub enum Error {}

/// Helper module to facilitate creating new error instances.
pub(crate) mod err {
    use super::{Error, Result};
}
