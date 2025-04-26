use crate::arena::Arena;
use crate::node::{Nodal, NodeId};
use std::cell::RefCell;
use std::ptr::NonNull;
use std::fmt::Write;

#[allow(private_bounds)]
pub struct Graph<'a, N: Nodal<'a>> {
    arena: Arena<N::InnerNode>,
    next_id: RefCell<NodeId>,
    start_node: RefCell<Option<NonNull<N::InnerNode>>>,
}

#[allow(private_bounds)]
impl<'a, N: Nodal<'a>> Graph<'a, N> {
    /// Creates a new NFA graph.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates a new NFA graph with preallocated memory for at least `capacity`
    /// nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        let arena = Arena::with_capacity(capacity);
        Self {
            arena,
            next_id: RefCell::new(0),
            start_node: RefCell::new(None),
        }
    }

    /// Creates a new NFA node.
    pub fn node(&'a self) -> N {
        let new_id = self
            .next_id
            .replace_with(|v| v.checked_add(1).expect("node id overflow"));
        let node_mut = self.arena.alloc_with(|| N::new_inner(new_id));
        let node = N::from_mut(node_mut);
        let mut start_node = self.start_node.borrow_mut();
        if start_node.is_none() {
            start_node.replace(node.as_ptr());
        }
        node
    }

    pub fn start_node(&'a self) -> N {
        if let Some(node_ptr) = self.start_node.borrow().as_ref() {
            return unsafe { N::from_ptr(*node_ptr) };
        }
        self.node()
    }
}

impl<'a, N: Nodal<'a>> std::default::Default for Graph<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> std::fmt::Debug for Graph<'a, crate::node::nfa::Node<'a>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for node in self.arena.iter().map(crate::node::nfa::Node::from_ref) {
            if first {
                first = false;
            } else {
                f.write_char('\n')?;
            }
            let mut is_empty = true;
            write!(f, "{:?} {{", node)?;
            for (target, transition) in node.symbol_targets() {
                f.write_str("\n    ")?;
                for range in transition.ranges() {
                    range.fmt(f)?;
                }
                write!(f, " -> {:?}", target)?;
                is_empty = false;
            }
            for target in node.epsilon_targets() {
                write!(f, "\n    [EPSILON] -> {:?}", target)?;
                is_empty = false;
            }
            if !is_empty {
                f.write_char('\n')?;
            }
            f.write_char('}')?;
        }
        Ok(())
    }
}
