use super::node::{Node, NodeInner, NodePtr};
use crate::arena::Arena;
use crate::node::NodeId;
use std::cell::RefCell;

pub struct Graph<T> {
    arena: Arena<NodeInner<T>>,
    next_id: RefCell<NodeId>,
    start_node: RefCell<Option<NodePtr<T>>>,
}

impl<T> Graph<T> {
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
    pub fn node(&self) -> Node<'_, T> {
        let new_id = self
            .next_id
            .replace_with(|v| v.checked_add(1).expect("node id overflow"));
        let node_ref: &NodeInner<T> = self.arena.alloc_with(|| NodeInner::new(new_id));
        let mut start_node = self.start_node.borrow_mut();
        if start_node.is_none() {
            start_node.replace(NodePtr::from_ref(node_ref));
        }
        Node::from(node_ref)
    }

    pub fn start_node(&self) -> Node<'_, T> {
        if let Some(node_ptr) = self.start_node.borrow().as_ref() {
            return Node::from(*node_ptr);
        }
        self.node()
    }
}

impl<T> std::default::Default for Graph<T> {
    fn default() -> Self {
        Self::new()
    }
}
