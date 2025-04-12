use crate::adt::{Map, MapIter, Set, SetIter};
use crate::edge::Edge;
use crate::node::NodeId;
use crate::range::Range;
use std::cell::{Ref, RefCell};
use std::ptr::NonNull;

pub struct Node<'a>(&'a NodeInner);

impl Copy for Node<'_> {}

impl Clone for Node<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> std::convert::From<&'a NodeInner> for Node<'a> {
    fn from(value: &'a NodeInner) -> Self {
        Self(value)
    }
}

impl<'a> std::convert::From<&'a mut NodeInner> for Node<'a> {
    fn from(value: &'a mut NodeInner) -> Self {
        Self(value)
    }
}

impl Node<'_> {
    pub fn id(&self) -> NodeId {
        self.0.id
    }
}

impl std::cmp::PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id.eq(&other.0.id)
    }
}

impl std::cmp::Eq for Node<'_> {}

impl std::cmp::PartialOrd for Node<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Node<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.id.cmp(&other.0.id)
    }
}

impl std::hash::Hash for Node<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id.hash(state)
    }
}

impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "node({})", self.id())
    }
}

impl Node<'_> {
    pub(super) unsafe fn from_ptr(ptr: NonNull<NodeInner>) -> Self {
        Self(unsafe { ptr.as_ref() })
    }

    pub(super) fn as_ptr(&self) -> NonNull<NodeInner> {
        unsafe { NonNull::<NodeInner>::new_unchecked(self.0 as *const NodeInner as *mut NodeInner) }
    }
}

impl<'a> Node<'a> {
    /// Connects this node to another node with a specified edge rule.
    /// If a connection to the target node already exists, it merges
    /// the new edge rule with the existing one.
    ///
    /// # Arguments
    ///
    /// * `to` - The target node to connect to
    /// * `with` - The edge rule describing valid transitions to the target
    pub fn connect(&self, to: Node<'a>, with: impl Into<Edge>) {
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

impl<'a> Node<'a> {
    pub fn connect_with_epsilon(&self, to: Node<'a>) {
        let mut targets = self.0.epsilon_targets.borrow_mut();
        targets.insert(to.as_ptr());
    }

    /// Returns a set of all nodes reachable from this one through epsilon
    /// transitions (including itself).
    ///
    /// Performs a recursive traversal of the node's epsilon transitions to
    /// calculate the epsilon closure. Each node is visited only once.
    #[allow(clippy::mutable_key_type)]
    pub fn eclosure(&self) -> Set<Node<'a>> {
        fn finder<'a>(node: Node<'a>, closure: &mut Set<Node<'a>>) {
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

impl<'a> Node<'a> {
    /// Returns an iterator over the targets of the node joint with symbol edges.
    pub fn symbol_targets(&self) -> SymbolTargetsIter<'a> {
        SymbolTargetsIter::new(self)
    }

    /// Returns an iterator over the targets of the node joint with epsilon edges.
    pub fn epsilon_targets(&self) -> EpsilonTargetsIter<'a> {
        EpsilonTargetsIter::new(self)
    }
}

pub(super) struct NodeInner {
    id: NodeId,
    targets: RefCell<Map<NonNull<NodeInner>, Edge>>,
    epsilon_targets: RefCell<Set<NonNull<NodeInner>>>,
}

impl NodeInner {
    pub(super) fn new(id: NodeId) -> Self {
        Self {
            id,
            targets: Default::default(),
            epsilon_targets: Default::default(),
        }
    }
}

pub struct EpsilonTargetsIter<'a> {
    _lock: Ref<'a, Set<NonNull<NodeInner>>>,
    iter: SetIter<'a, NonNull<NodeInner>>,
}

impl<'a> EpsilonTargetsIter<'a> {
    pub fn new(node: &Node<'a>) -> Self {
        // guarantees that borrow_mut() is impossible during lifetime of this
        // iterator
        let lock = node.0.epsilon_targets.borrow();

        // `_lock` (with type `cell::Ref`) should return an iterator of inside
        // structure with lifetime of `_lock` itself. But I break it here, to
        // get iterator with lifetime 'a instead of `_lock`s one. It will allow
        // in `Iterator` implementation to convert references to `NonNull<NodeInner>` into
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

impl<'a> std::iter::Iterator for EpsilonTargetsIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|node_ptr| unsafe { Node::from_ptr(*node_ptr) })
    }
}

pub struct SymbolTargetsIter<'a> {
    lock: Ref<'a, Map<NonNull<NodeInner>, Edge>>,
    iter: MapIter<'a, NonNull<NodeInner>, Edge>,
}

impl<'a> SymbolTargetsIter<'a> {
    pub fn new(node: &Node<'a>) -> Self {
        // implementation details are in `EpsilonTargetsIter`'s constructor
        let lock = node.0.targets.borrow();
        let ptr = node.0.targets.as_ptr();
        let iter = unsafe { &*ptr }.iter();
        Self { lock, iter }
    }
}

impl<'a> std::iter::Iterator for SymbolTargetsIter<'a> {
    type Item = (Node<'a>, EdgeIter<'a>);

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

pub struct EdgeIter<'a> {
    _lock: Ref<'a, Map<NonNull<NodeInner>, Edge>>,
    iter: std::slice::Iter<'a, Range>,
}

impl std::iter::Iterator for EdgeIter<'_> {
    type Item = Range;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }
}
