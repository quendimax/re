use super::{NodeBase, NodeId};
use crate::adt::Set;
use crate::transition::Transition;
use std::cell::RefCell;
use std::ptr::NonNull;

/// Node for an NFA graph.
///
/// It contains ID (unique within its graph owner). Also it can be connected to
/// another node using [`Edge`] of symbols.
pub struct Node<'a>(&'a NodeInner);

struct NodeInner {
    base: NodeBase<Self>,
    _epsilon_targets: RefCell<Set<NonNull<Self>>>,
}

impl NodeInner {
    pub(crate) fn new(id: NodeId) -> Self {
        Self {
            base: NodeBase::<Self>::new(id),
            _epsilon_targets: Default::default(),
        }
    }
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
}

/// Private API
impl Node<'_> {
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
