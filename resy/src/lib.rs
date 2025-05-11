mod codec;
pub use codec::Codec;

mod error;
pub use error::Error;

mod parse;
pub use parse::Parser;

use regr::{Graph, Node};

/// Parse regular expression from `pattern` parameter, and builds corresponding
/// NFA graph. Returns a pair of nodes: the first one and the last one.
pub fn parse<'a>(_pattern: &str, _nfa: &'a Graph) -> (Node<'a>, Node<'a>) {
    todo!("add check for graph is NFA");
}
