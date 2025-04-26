use super::{NodeBase, NodeId, Nodal};
use crate::adt::{Set, SetIter};
use crate::transition::Transition;
use std::cell::{Ref, RefCell};
use std::ptr::NonNull;

/// Node for an NFA graph.
///
/// It contains ID (unique within its graph owner). Also it can be connected to
/// another node using [`Edge`] of symbols.
pub struct Node<'a>(&'a NodeInner);

pub(crate) struct NodeInner {
    base: NodeBase<Self>,
    epsilon_targets: RefCell<Set<NonNull<Self>>>,
}

/// Public API
impl<'a> Node<'a> {
    #[inline]
    pub fn id(self) -> NodeId {
        self.0.base.id
    }

    #[inline]
    pub fn connect(self, to: Node<'a>, with: impl Into<Transition>) {
        self.0.base.connect(to.as_ptr(), with);
    }

    /// Returns an iterator over target nodes, i.e. nodes that this node is
    /// connected to.
    #[inline]
    pub fn targets(self) -> TargetIter<'a> {
        TargetIter::new(self)
    }

    /// Returns an iterator over target nodes, i.e. nodes that this node is
    /// connected to.
    #[inline]
    pub fn symbol_targets(self) -> SymbolTargetIter<'a> {
        SymbolTargetIter::new(self)
    }

    /// Returns an iterator over epsilon target nodes, i.e. nodes that this node is
    /// connected to with Epsilon transition.
    #[inline]
    pub fn epsilon_targets(self) -> EpsilonTargetIter<'a> {
        EpsilonTargetIter::new(self)
    }
}

/// Private API
impl<'a> Nodal<'a> for Node<'a> {
    type InnerNode = NodeInner;

    fn new_inner(id: NodeId) -> Self::InnerNode {
        Self::InnerNode {
            base: NodeBase::<Self::InnerNode>::new(id),
            epsilon_targets: Default::default(),
        }
    }

    #[inline]
    fn from_ref(value: &'a Self::InnerNode) -> Self {
        Self(value)
    }

    #[inline]
    fn from_mut(value: &'a mut NodeInner) -> Self {
        Self(value)
    }

    #[inline]
    unsafe fn from_ptr(ptr: NonNull<NodeInner>) -> Self {
        Self(unsafe { ptr.as_ref() })
    }

    #[inline]
    fn as_ptr(self) -> NonNull<NodeInner> {
        unsafe { NonNull::<NodeInner>::new_unchecked(self.0 as *const NodeInner as *mut NodeInner) }
    }
}

impl Copy for Node<'_> {}

impl Clone for Node<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl std::cmp::Eq for Node<'_> {}

impl std::cmp::PartialEq for Node<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.base.eq(&other.0.base)
    }
}

impl std::cmp::Ord for Node<'_> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.base.cmp(&other.0.base)
    }
}

impl std::cmp::PartialOrd for Node<'_> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for Node<'_> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.base.hash(state)
    }
}

impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node({})", self.id())
    }
}

pub struct TargetIter<'a> {
    iter: super::TargetIter<'a, NodeInner>,
}

impl<'a> TargetIter<'a> {
    fn new(node: Node<'a>) -> Self {
        Self {
            iter: super::TargetIter::new(&node.0.base),
        }
    }
}

impl<'a> std::iter::Iterator for TargetIter<'a> {
    type Item = Node<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|node_ptr| unsafe { Node::from_ptr(node_ptr) })
    }
}

pub struct SymbolTargetIter<'a> {
    iter: super::SymbolTargetIter<'a, NodeInner>,
}

impl<'a> SymbolTargetIter<'a> {
    #[inline]
    fn new(node: Node<'a>) -> Self {
        Self {
            iter: super::SymbolTargetIter::new(&node.0.base),
        }
    }
}

impl<'a> std::iter::Iterator for SymbolTargetIter<'a> {
    type Item = (Node<'a>, &'a Transition);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(node_ptr, transition)| (unsafe { Node::from_ptr(node_ptr) }, transition))
    }
}

pub struct EpsilonTargetIter<'a> {
    _lock: Ref<'a, Set<NonNull<NodeInner>>>,
    iter: SetIter<'a, NonNull<NodeInner>>,
}

impl<'a> EpsilonTargetIter<'a> {
    pub fn new(node: Node<'a>) -> Self {
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
        let lock = node.0.epsilon_targets.borrow();
        let ptr = node.0.epsilon_targets.as_ptr();
        let iter = unsafe { &*ptr }.iter();
        Self { _lock: lock, iter }
    }
}

impl<'a> std::iter::Iterator for EpsilonTargetIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|node_ptr| unsafe { Node::from_ptr(*node_ptr) })
    }
}
