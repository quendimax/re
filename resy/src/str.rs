//! Module containing a lexer and a parser that parse RE patterns from Rust
//! (UTF-8) strings.

mod lexis;
pub use lexis::{Lexer, Token, TokenKind, tok};
