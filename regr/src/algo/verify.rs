use crate::Epsilon;
use crate::algo::{self, VisitResult::*};
use crate::graph::Graph;
use crate::node::Node;
use redt::{SetU8, ops::*};

/// Checks if the given graph represents a valid DFA.
#[allow(clippy::mutable_key_type)]
pub fn verify_dfa(graph: &Graph<'_>) -> bool {
    let mut is_dfa = true;
    algo::visit_nodes(graph.start_node(), |node| {
        if !verify_dfa_node(node) {
            is_dfa = false;
            return Stop;
        }
        Recurse
    });
    is_dfa
}

/// Checks if the given node meets the requirements of a DFA.
pub fn verify_dfa_node<'a>(node: Node<'a>) -> bool {
    let mut has_epsilon = false;
    let mut sym_mask = SetU8::empty();
    for (_, tr) in node.targets().iter() {
        if tr.contains(Epsilon) {
            if !has_epsilon {
                has_epsilon = true;
            } else {
                return false;
            }
        } else {
            if sym_mask.contains(tr.as_set().as_ref()) {
                return false;
            }
            sym_mask.include(tr.as_set().as_ref());
        }
    }
    true
}
