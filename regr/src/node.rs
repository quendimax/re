pub mod nfa;

use crate::adt::{Map, MapKeyIter};
use crate::edge::Edge;
use std::cell::{Ref, RefCell};
use std::ptr::NonNull;

/// Integer type that represents node identifier. It is expected that it is
/// unique within a graph.
pub type NodeId = u32;

#[derive(Debug)]
struct NodeBase<N> {
    id: NodeId,
    targets: RefCell<Map<NonNull<N>, Edge>>,
}

impl<N> NodeBase<N> {
    /// Creates a new NodeBase instance. It is expected that `id` is unique for
    /// a Graph.
    fn new(id: NodeId) -> Self {
        Self {
            id,
            targets: Default::default(),
        }
    }

    /// Connects this node to another node with a specified edge rule.
    /// If a connection to the target node already exists, it merges
    /// the new edge rule with the existing one.
    ///
    /// # Arguments
    ///
    /// * `to` - The target node to connect to
    /// * `with` - The edge rule describing valid transitions to the target
    fn connect(&self, to: NonNull<N>, with: impl Into<Edge>) {
        let with = with.into();
        let mut targets = self.targets.borrow_mut();
        if let Some(edge) = targets.get_mut(&to) {
            edge.merge(&with);
        } else {
            targets.insert(to, with);
        }
    }
}

impl<N> std::cmp::Eq for NodeBase<N> {}

impl<N> std::cmp::PartialEq for NodeBase<N> {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl<N> std::cmp::Ord for NodeBase<N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<N> std::cmp::PartialOrd for NodeBase<N> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<N> std::hash::Hash for NodeBase<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

struct TargetIter<'a, N> {
    _lock: Ref<'a, Map<NonNull<N>, Edge>>,
    iter: MapKeyIter<'a, NonNull<N>, Edge>,
}

impl<'a, N> TargetIter<'a, N> {
    pub fn new(node: &'a NodeBase<N>) -> Self {
        // `_lock` (with type `cell::Ref`) should return an iterator of inside
        // structure with lifetime of `_lock` itself. But I break it here, to
        // get iterator with lifetime 'a instead of `_lock`s one. It will allow
        // in `Iterator` implementation to convert references to
        // `NonNull<NodeInner>` into `Node` instances with lifetime 'a. It is
        // safe though it is not allowed to put references to the RefCell's
        // inner structure outside, because the RefCell contains pointers to the
        // nodes, and the iterator will return copies of these pointers (via
        // Node wrapper), but not references to the pointers. So references to
        // the RefCell's inner contents are never gone out of this iterator.
        let lock = node.targets.borrow();
        let ptr = node.targets.as_ptr();
        let iter = unsafe { &*ptr }.keys();
        Self { _lock: lock, iter }
    }
}

impl<N> std::iter::Iterator for TargetIter<'_, N> {
    type Item = NonNull<N>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }
}
