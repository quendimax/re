use super::node::{Node, NodeInner};
use crate::arena::Arena;
use crate::node::NodeId;
use std::cell::RefCell;
use std::fmt::Write;
use std::ptr::NonNull;

pub struct Graph {
    arena: Arena<NodeInner>,
    next_id: RefCell<NodeId>,
    start_node: RefCell<Option<NonNull<NodeInner>>>,
}

impl Graph {
    /// Creates a new DFA graph.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Creates a new DFA graph with preallocated memory for at least `capacity` nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            arena: Arena::with_capacity(capacity),
            next_id: RefCell::new(0),
            start_node: RefCell::new(None),
        }
    }

    pub fn node(&self) -> Node<'_> {
        let new_id = self
            .next_id
            .replace_with(|v| v.checked_add(1).expect("node id overflow"));
        let node = Node::from(self.arena.alloc_with(|| NodeInner::new(new_id)));
        let mut start_node = self.start_node.borrow_mut();
        if start_node.is_none() {
            start_node.replace(node.as_ptr());
        }
        node
    }

    pub fn start_node(&self) -> Node<'_> {
        if let Some(node_ptr) = self.start_node.borrow().as_ref() {
            return unsafe { Node::from_ptr(*node_ptr) };
        }
        self.node()
    }
}

impl std::default::Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for node in self.arena.iter().map(Node::from) {
            if first {
                first = false;
            } else {
                f.write_char('\n')?;
            }
            let mut is_empty = true;
            write!(f, "node {} {{", node.id())?;

            // TODO:


            if !is_empty {
                f.write_char('\n')?;
            }
            f.write_char('}');
        }
        Ok(())
    }
}
