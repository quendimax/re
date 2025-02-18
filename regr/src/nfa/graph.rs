use super::node::{Node, NodeInner, NodePtr};
use crate::arena::Arena;
use crate::node::NodeId;
use std::cell::RefCell;
use std::fmt::Write;

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
        let node = Node::from(self.arena.alloc_with(|| NodeInner::new(new_id)));
        let mut start_node = self.start_node.borrow_mut();
        if start_node.is_none() {
            start_node.replace(node.as_ptr());
        }
        println!("new node with id {}", node.id());
        node
    }

    pub fn start_node(&self) -> Node<'_, T> {
        if let Some(node_ptr) = self.start_node.borrow().as_ref() {
            return unsafe { Node::from_ptr(*node_ptr) };
        }
        self.node()
    }
}

impl<T> std::default::Default for Graph<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: std::fmt::Debug + Copy + PartialEq> std::fmt::Debug for Graph<T> {
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
            for (target, range_iter) in node.symbol_targets() {
                f.write_str("\n    ")?;
                for range in range_iter {
                    range.fmt(f)?;
                }
                write!(f, " -> node {}", target.id())?;
                is_empty = false;
            }
            for target in node.epsilon_targets() {
                write!(f, "\n    [EPSILON] -> node {}", target.id())?;
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
