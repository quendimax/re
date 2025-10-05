use crate::node::Node;
use crate::transition::Transition;
use redt::Set;

/// Recursively visit all nodes in the graph starting from the `start_node`,
/// applying the given `action` to each node.
///
/// The `action` should return `true` if the node's children should be visited,
/// and `false` otherwise. So, `action` is called for `start_node` at least.
pub fn for_each_node<'n, A>(start_node: Node<'n>, action: A)
where
    A: FnMut(Node<'n>) -> bool,
{
    let mut action = action;
    let mut visited = Set::new();
    let mut unvisited = Vec::with_capacity(64);
    unvisited.push(start_node);
    while let Some(node) = unvisited.pop() {
        visited.insert(node.nid());
        if action(node) {
            for (target, _) in node.targets().iter() {
                if !visited.contains(&target.nid()) {
                    unvisited.push(*target);
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
pub fn for_each_transition<'n, A>(start_node: Node<'n>, action: A)
where
    A: FnMut(Node<'n>, Transition<'n>, Node<'n>) -> bool,
{
    let mut action = action;
    let mut visited = Set::new();
    let mut unvisited = Vec::with_capacity(64);
    unvisited.push(start_node);
    while let Some(node) = unvisited.pop() {
        visited.insert(node.nid());
        for (target, tr) in node.targets().iter() {
            if action(node, *tr, *target) && !visited.contains(&target.nid()) {
                unvisited.push(*target);
            }
        }
    }
}
