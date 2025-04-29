use crate::arena::Arena;
use crate::node::{Node, NodeId, NodeInner};
use std::cell::RefCell;
use std::fmt::Write;
use std::ptr::NonNull;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AutomatonKind {
    NFA,
    DFA,
}

#[allow(private_bounds)]
pub struct Graph {
    arena: Arena<NodeInner>,
    next_id: RefCell<NodeId>,
    start_node: RefCell<Option<NonNull<NodeInner>>>,
    kind: AutomatonKind,
}

#[allow(private_bounds)]
impl Graph {
    /// Creates a new NFA graph.
    pub fn nfa() -> Self {
        Self::with_capacity(0, AutomatonKind::NFA)
    }

    /// Creates a new NFA graph with preallocated memory for at least `capacity`
    /// nodes.
    pub fn with_capacity(capacity: usize, kind: AutomatonKind) -> Self {
        let arena = Arena::with_capacity(capacity);
        Self {
            arena,
            next_id: RefCell::new(0),
            start_node: RefCell::new(None),
            kind,
        }
    }

    /// Creates a new NFA node.
    pub fn node(&self) -> Node<'_> {
        let new_id = self
            .next_id
            .replace_with(|v| v.checked_add(1).expect("node id overflow"));
        let node_mut = self.arena.alloc_with(|| Node::new_inner(new_id, self.kind));
        let node = Node::from_mut(node_mut);
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
    #[inline]
    fn default() -> Self {
        Self::nfa()
    }
}

impl std::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for node in self.arena.iter().map(Node::from_ref) {
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
