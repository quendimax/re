mod arena;

pub mod adt;

mod graph;
pub use graph::{AutomatonKind, Graph};

pub mod node;
pub use node::{Node, NodeId};

mod ops;

mod private {
    pub trait Sealed {}
}

mod span;
pub use span::{Span, span};

mod symbol;
pub use symbol::{Epsilon, Symbol};

mod transition;
pub use transition::Transition;
