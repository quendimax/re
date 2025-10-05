mod verify;
pub use verify::verify_dfa;

mod visit;
pub use visit::{for_each_node, for_each_transition};
