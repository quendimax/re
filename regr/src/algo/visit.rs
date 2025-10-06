use crate::node::Node;
use crate::transition::Transition;
use redt::Set;

pub enum VisitResult {
    Stop,
    Continue,
    Recurse,
}

use VisitResult::*;

/// Recursively visit all nodes in the graph starting from the `start_node`,
/// applying the given `action` to each node.
///
/// The `action` should return `true` if the node's children should be visited,
/// and `false` otherwise. So, `action` is called for `start_node` at least.
pub fn visit_nodes<'n, A>(start_node: Node<'n>, action: A)
where
    A: FnMut(Node<'n>) -> VisitResult,
{
    let mut action = action;
    let mut visited = Set::new();
    let mut unvisited = Vec::with_capacity(512);
    unvisited.push(start_node);
    while let Some(node) = unvisited.pop() {
        visited.insert(node.nid());
        match action(node) {
            Stop => break,
            Continue => continue,
            Recurse => {
                for (target, _) in node.targets().iter() {
                    if !visited.contains(&target.nid()) {
                        unvisited.push(*target);
                    }
                }
            }
        }
    }
}

/// Recursively visit all transitions in the graph starting from the
/// `start_node`, applying the given `action` to each transition.
///
/// The `action` should return `true` if you want to visit transitions of the
/// current target node. Otherwise, the transitions will be skipped.
pub fn visit_transitions<'n, A>(start_node: Node<'n>, action: A)
where
    A: FnMut(Node<'n>, Transition<'n>, Node<'n>) -> VisitResult,
{
    let mut action = action;
    let mut visited = Set::new();
    let mut unvisited = Vec::with_capacity(512);
    unvisited.push(start_node);
    while let Some(node) = unvisited.pop() {
        visited.insert(node.nid());
        for (target, tr) in node.targets().iter() {
            match action(node, *tr, *target) {
                Stop => return,
                Continue => continue,
                Recurse => {
                    if !visited.contains(&target.nid()) {
                        unvisited.push(*target);
                    }
                }
            }
        }
    }
}
