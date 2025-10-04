use crate::Epsilon;
use crate::graph::Graph;
use crate::node::Node;
use redt::{Set, SetU8, ops::*};

/// Checks if the given graph represents a valid DFA.
#[allow(clippy::mutable_key_type)]
pub fn verify_dfa<'a>(graph: &Graph<'a>) -> bool {
    let mut visited = Set::new();
    let mut unvisited = Vec::new();
    unvisited.push(graph.start_node());
    while let Some(node) = unvisited.pop() {
        if visited.contains(&node.nid()) {
            continue;
        }
        visited.insert(node.nid());
        if !verify_dfa_node(node) {
            return false;
        }
        node.for_each_target(|target, _| {
            if !visited.contains(&target.nid()) {
                unvisited.push(target);
            }
        });
    }
    true
}

fn verify_dfa_node<'a>(node: Node<'a>) -> bool {
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
