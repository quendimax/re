mod arena;

pub mod adt;

pub mod dfa;

mod edge;
pub use edge::Edge;

mod error;
pub use error::{Error, Result, err};

mod graph;
pub use graph::{AutomatonKind, Graph};

pub mod nfa;

pub mod node;
pub use node::{Node, NodeId};

mod ops;

mod private {
    pub trait Sealed {}
}

mod range;
pub use range::{Range, range};

mod symbol;
pub use symbol::{Epsilon, Symbol};

mod transition;
pub use transition::Transition;

mod translate;
pub use translate::Translator;
