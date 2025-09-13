mod error;
pub use error::{Error, Result};

mod hir;
pub use hir::{ConcatHir, DisjunctHir, GroupHir, Hir, RepeatHir};

mod lexis;
pub use lexis::{Lexer, Token, TokenKind, tok};

mod syntax;
pub use syntax::Parser;

/// Re-export of the `renc` crate.
pub mod enc {
    pub use renc::*;
}
