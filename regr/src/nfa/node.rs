use crate::adt::{Map, MapIter, Set, SetIter};
use crate::edge::Edge;
use crate::node::NodeId;
use crate::range::Range;
use crate::symbol::Symbol;
use std::cell::{Ref, RefCell};
use std::ptr::NonNull;

pub struct Node<'a, T>(&'a NodeInner<T>);

impl<T> Copy for Node<'_, T> {}

impl<T> Clone for Node<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> std::convert::From<&'a NodeInner<T>> for Node<'a, T> {
    fn from(value: &'a NodeInner<T>) -> Self {
        Self(value)
    }
}

impl<'a, T> std::convert::From<&'a mut NodeInner<T>> for Node<'a, T> {
    fn from(value: &'a mut NodeInner<T>) -> Self {
        Self(value)
    }
}

impl<T> Node<'_, T> {
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
        self.0.id.cmp(&other.0.id)
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

impl<T> std::fmt::Display for Node<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl<T> Node<'_, T> {
    pub(super) unsafe fn from_ptr(ptr: NodePtr<T>) -> Self {
        Self(unsafe { ptr.as_ref() })
    }

    pub(super) fn as_ptr(&self) -> NodePtr<T> {
        unsafe { NodePtr::new_unchecked(self.0 as *const NodeInner<T> as *mut NodeInner<T>) }
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
        let to = to.as_ptr();
        let with = with.into();
        let mut targets = self.0.targets.borrow_mut();
        if let Some(edge) = targets.get_mut(&to) {
            edge.merge(&with);
        } else {
            targets.insert(to, with);
        }
    }
}

impl<'a, T> Node<'a, T> {
    pub fn connect_with_epsilon(&self, to: Node<'a, T>) {
        let mut targets = self.0.epsilon_targets.borrow_mut();
        targets.insert(to.as_ptr());
    }

    /// Returns a set of all nodes reachable from this one through epsilon
    /// transitions (including itself).
    ///
    /// Performs a recursive traversal of the node's epsilon transitions to
    /// calculate the epsilon closure. Each node is visited only once.
    #[allow(clippy::mutable_key_type)]
    pub fn eclosure(&self) -> Set<Node<'a, T>> {
        fn finder<'a, T>(node: Node<'a, T>, closure: &mut Set<Node<'a, T>>) {
            if closure.contains(&node) {
                return;
            }
            closure.insert(node);
            for target in node.epsilon_targets() {
                finder(target, closure);
            }
        }
        let mut closure_set = Set::new();
        finder(*self, &mut closure_set);
        closure_set
    }
}

impl<'a, T> Node<'a, T> {
    /// Returns an iterator over the targets of the node joint with symbol edges.
    pub fn symbol_targets(&self) -> SymbolTargetsIter<'a, T> {
        SymbolTargetsIter::new(self)
    }

    /// Returns an iterator over the targets of the node joint with epsilon edges.
    pub fn epsilon_targets(&self) -> EpsilonTargetsIter<'a, T> {
        EpsilonTargetsIter::new(self)
    }
}

pub(super) type NodePtr<T> = NonNull<NodeInner<T>>;

pub(super) struct NodeInner<T> {
    id: NodeId,
    targets: RefCell<Map<NodePtr<T>, Edge<T>>>,
    epsilon_targets: RefCell<Set<NodePtr<T>>>,
}

impl<T> NodeInner<T> {
    pub(super) fn new(id: NodeId) -> Self {
        Self {
            id,
            targets: Default::default(),
            epsilon_targets: Default::default(),
        }
    }
}

pub struct EpsilonTargetsIter<'a, T> {
    _lock: Ref<'a, Set<NodePtr<T>>>,
    iter: SetIter<'a, NodePtr<T>>,
}

impl<'a, T> EpsilonTargetsIter<'a, T> {
    pub fn new(node: &Node<'a, T>) -> Self {
        // guarantees that borrow_mut() is impossible during lifetime of this
        // iterator
        let lock = node.0.epsilon_targets.borrow();

        // `_lock` (with type `cell::Ref`) should return an iterator of inside
        // structure with lifetime of `_lock` itself. But I break it here, to
        // get iterator with lifetime 'a instead of `_lock`s one. It will allow
        // in `Iterator` implementation to convert references to `NodePtr` into
        // `Node` instances with lifetime 'a. It is safe though it is not
        // allowed to put references to the RefCell's inner structure outside,
        // because the RefCell contains pointers to the nodes, and the iterator
        // will return copies of these pointers (via Node wrapper), but not
        // references to the pointers. So references to the RefCell's inner
        // contents are never gone out of this iterator.
        let ptr = node.0.epsilon_targets.as_ptr();
        let iter = unsafe { &*ptr }.iter();
        Self { _lock: lock, iter }
    }
}

impl<'a, T> std::iter::Iterator for EpsilonTargetsIter<'a, T> {
    type Item = Node<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|node_ptr| unsafe { Node::from_ptr(*node_ptr) })
    }
}

pub struct SymbolTargetsIter<'a, T> {
    lock: Ref<'a, Map<NodePtr<T>, Edge<T>>>,
    iter: MapIter<'a, NodePtr<T>, Edge<T>>,
}

impl<'a, T> SymbolTargetsIter<'a, T> {
    pub fn new(node: &Node<'a, T>) -> Self {
        // implementation details are in `EpsilonTargetsIter`'s constructor
        let lock = node.0.targets.borrow();
        let ptr = node.0.targets.as_ptr();
        let iter = unsafe { &*ptr }.iter();
        Self { lock, iter }
    }
}

impl<'a, T> std::iter::Iterator for SymbolTargetsIter<'a, T> {
    type Item = (Node<'a, T>, EdgeIter<'a, T>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((node_ptr, edge)) = self.iter.next() {
            let node = unsafe { Node::from_ptr(*node_ptr) };
            let iter = EdgeIter {
                _lock: Ref::clone(&self.lock),
                iter: edge.ranges(),
            };
            Some((node, iter))
        } else {
            None
        }
    }
}

pub struct EdgeIter<'a, T> {
    _lock: Ref<'a, Map<NodePtr<T>, Edge<T>>>,
    iter: std::slice::Iter<'a, Range<T>>,
}

impl<T: Copy> std::iter::Iterator for EdgeIter<'_, T> {
    type Item = Range<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }
}
