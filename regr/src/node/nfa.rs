use super::NodeId;
use crate::adt::{Map, Set};
use crate::edge::Edge;
use crate::symbol::Symbol;
use std::cell::RefCell;

#[derive(Clone, Copy)]
pub struct Node<'a, T>(&'a NodeInner<'a, T>);

pub(super) struct NodeInner<'a, T> {
    id: NodeId,
    targets: RefCell<Map<Node<'a, T>, Edge<T>>>,
    epsilon_targets: RefCell<Set<Node<'a, T>>>,
}

impl<T> NodeInner<'_, T> {
    /// Creates a new NFA Node with the specified id.
    pub(super) fn new(id: NodeId) -> Self {
        Self {
            id,
            targets: Default::default(),
            epsilon_targets: Default::default(),
        }
    }
}

impl<'a, T: PartialOrd + Ord + Symbol> Node<'a, T> {
    /// Connects this node to another node with a specified edge rule.
    /// If a connection to the target node already exists, it merges
    /// the new edge rule with the existing one.
    ///
    /// # Arguments
    ///
    /// * `to` - The target node to connect to
    /// * `with` - The edge rule describing valid transitions to the target
    pub fn connect(&self, to: Node<'a, T>, with: impl Into<Edge<T>>) {
        let with = with.into();
        let mut targets = self.0.targets.borrow_mut();
        if let Some(edge) = targets.get_mut(&to) {
            edge.merge(&with);
        } else {
            targets.insert(to, with);
        }
    }

    pub fn connect_with_epsilon(&self, to: Node<'a, T>) {
        let mut targets = self.0.epsilon_targets.borrow_mut();
        targets.insert(to);
    }

    /// Returns a set of all nodes reachable from this one through epsilon
    /// transitions (including itself).
    ///
    /// Performs a recursive traversal of the node's epsilon transitions to
    /// calculate the epsilon closure. Each node is visited only once.
    #[allow(clippy::mutable_key_type)]
    pub fn eclosure(&'a self) -> Set<Node<'a, T>> {
        fn finder<'a, T: Copy>(node: Node<'a, T>, closure: &mut Set<Node<'a, T>>) {
            if closure.contains(&node) {
                return;
            }
            closure.insert(node);
            for target in node.0.epsilon_targets.borrow().iter() {
                finder(*target, closure);
            }
        }
        let mut closure_set = Set::new();
        finder(*self, &mut closure_set);
        closure_set
    }
}

impl<'a, T: PartialEq + Copy + std::fmt::Debug> Node<'a, T> {
    /// Prints NFA graph treating the current node as the start one.
    #[allow(clippy::mutable_key_type)]
    pub fn print_graph(&self, w: &mut dyn std::fmt::Write, ident: &str) -> std::fmt::Result {
        fn collect_nodes<'a, T: Copy>(node: Node<'a, T>, visited: &mut Set<Node<'a, T>>) {
            if visited.contains(&node) {
                return;
            }
            visited.insert(node);
            for target_node in node.0.targets.borrow().iter() {
                collect_nodes(*target_node.0, visited);
            }
            for target_node in node.0.epsilon_targets.borrow().iter() {
                collect_nodes(*target_node, visited);
            }
        }

        let mut visited = Set::<Node<'a, T>>::new();
        collect_nodes(*self, &mut visited);

        let mut first_iteration = true;
        for node in visited.iter() {
            if !first_iteration {
                w.write_str("\n")?;
            } else {
                first_iteration = false;
            }
            if node.0.id == self.0.id {
                write!(w, "{ident}start {}", node.0.id)?;
            } else {
                write!(w, "{ident}node {}", node.0.id)?;
            }
            let targets = node.0.targets.borrow();
            let epsilon_targets = node.0.epsilon_targets.borrow();
            if !targets.is_empty() || !epsilon_targets.is_empty() {
                write!(w, ":")?;
            }
            for (target, edge) in targets.iter() {
                write!(w, "\n{ident}    {:?} -> {}", edge, target.0.id)?;
            }
            for target in epsilon_targets.iter() {
                write!(w, "\n{ident}    EPSILON -> {}", target.0.id)?;
            }
        }
        Ok(())
    }
}

impl<'a, T> Node<'a, T> {
    pub(super) fn new(inner: &'a NodeInner<'a, T>) -> Self {
        Self(inner)
    }

    pub fn id(&self) -> NodeId {
        self.0.id
    }
}

impl<T> std::cmp::PartialEq for Node<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id.eq(&other.0.id)
    }
}

impl<T> std::cmp::Eq for Node<'_, T> {}

impl<T> std::cmp::PartialOrd for Node<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> std::cmp::Ord for Node<'_, T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Ord::cmp(&self.0.id, &other.0.id)
    }
}

impl<T> std::hash::Hash for Node<'_, T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id.hash(state)
    }
}

impl<T> std::fmt::Debug for Node<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node({})", self.id())
    }
}
