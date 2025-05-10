use thiserror::Error;

/// The `regr` specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    /// A try to merge delimited ranges
    #[error("can't merge delimited ranges")]
    MergeDelimitedRanges,
}

/// Inner module to facilitate creating errors and avoid ambiguty with other
/// `Error` types.
pub(crate) mod err {
    use super::*;

    pub(crate) const fn merge_delimited_ranges<T>() -> Result<T> {
        Err(Error::MergeDelimitedRanges)
    }
}
