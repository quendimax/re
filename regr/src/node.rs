use crate::Epsilon;
use crate::adt::{Map, MapIter, MapKeyIter, Set, SetIter};
use crate::graph::AutomatonKind;
use crate::transition::Transition;
use std::cell::{Ref, RefCell};
use std::fmt::Write;
use std::ptr::NonNull;

/// Integer type that represents node identifier. It is expected that it is
/// unique within a graph.
pub type NodeId = u32;

/// Node for an NFA graph.
///
/// It contains ID (unique within its graph owner). Also it can be connected to
/// another node using [`Edge`] of symbols.
pub struct Node<'a>(&'a NodeInner);

pub(crate) struct NodeInner {
    id: NodeId,
    targets: RefCell<Map<NonNull<NodeInner>, Transition>>,
    variant: NodeVariant,
}

enum NodeVariant {
    DfaNode {
        occupied_symbols: RefCell<Transition>,
    },
    NfaNode {
        epsilon_targets: RefCell<Set<NonNull<NodeInner>>>,
    },
}

use NodeVariant::{DfaNode, NfaNode};

/// Public API
impl<'a> Node<'a> {
    #[inline]
    pub fn id(self) -> NodeId {
        self.0.id
    }

    /// Connects this node to another node with a specified edge rule.
    /// If a connection to the target node already exists, it merges
    /// the new edge rule with the existing one.
    ///
    /// # Arguments
    ///
    /// * `to` - The target node to connect to
    /// * `with` - The edge rule describing valid transitions to the target
    pub fn connect<T>(self, to: Node<'a>, with: T)
    where
        Self: ConnectOp<'a, T>,
    {
        ConnectOp::connect(self, to, with);
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
impl<'a> Node<'a> {
    pub(crate) fn new_inner(id: NodeId, kind: AutomatonKind) -> NodeInner {
        match kind {
            AutomatonKind::NFA => NodeInner {
                id,
                targets: Default::default(),
                variant: NfaNode {
                    epsilon_targets: Default::default(),
                },
            },
            AutomatonKind::DFA => NodeInner {
                id,
                targets: Default::default(),
                variant: DfaNode {
                    occupied_symbols: Default::default(),
                },
            },
        }
    }

    #[inline]
    pub(crate) fn from_ref(value: &'a NodeInner) -> Self {
        Self(value)
    }

    #[inline]
    pub(crate) fn from_mut(value: &'a mut NodeInner) -> Self {
        Self(value)
    }

    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: NonNull<NodeInner>) -> Self {
        Self(unsafe { ptr.as_ref() })
    }

    #[inline]
    pub(crate) fn as_ptr(self) -> NonNull<NodeInner> {
        unsafe { NonNull::<NodeInner>::new_unchecked(self.0 as *const NodeInner as *mut NodeInner) }
    }
}

pub trait ConnectOp<'a, T> {
    fn connect(self, to: Node<'a>, with: T);
}

impl<'a, T> ConnectOp<'a, T> for Node<'a>
where
    T: Copy + std::fmt::Debug,
    Transition: crate::ops::ContainOp<T> + crate::ops::MergeOp<T>,
{
    fn connect(self, to: Node<'a>, with: T) {
        if let DfaNode { occupied_symbols } = &self.0.variant {
            let mut occupied_symbols = occupied_symbols.borrow_mut();
            if occupied_symbols.contains(with) {
                panic!("connection {with:?} already exists; DFA node can't have more");
            }
            occupied_symbols.merge(with);
        }

        let to = to.as_ptr();
        let mut targets = self.0.targets.borrow_mut();
        if let Some(tr) = targets.get_mut(&to) {
            tr.merge(with);
        } else {
            let mut tr = Transition::default();
            tr.merge(with);
            targets.insert(to, tr);
        }
    }
}

impl<'a> ConnectOp<'a, Epsilon> for Node<'a> {
    fn connect(self, to: Node<'a>, _with: Epsilon) {
        match &self.0.variant {
            DfaNode { .. } => {
                panic!("NFA nodes can't be connected with Epsilon");
            }
            NfaNode { epsilon_targets } => {
                let mut epsilon_targets = epsilon_targets.borrow_mut();
                epsilon_targets.insert(to.as_ptr());
            }
        }
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
    fn eq(&self, other: &Self) -> bool {
        self.0.id.eq(&other.0.id)
    }
}

impl std::cmp::Ord for Node<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.id.cmp(&other.0.id)
    }
}

impl std::cmp::PartialOrd for Node<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for Node<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.id.hash(state)
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Node<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("node(")?;
                std::fmt::$trait::fmt(&self.0.id, f)?;
                f.write_char(')')
            }
        }
    };
}

impl_fmt!(std::fmt::Display);
impl_fmt!(std::fmt::Debug);
impl_fmt!(std::fmt::Binary);
impl_fmt!(std::fmt::Octal);
impl_fmt!(std::fmt::UpperHex);
impl_fmt!(std::fmt::LowerHex);

pub struct TargetIter<'a> {
    _lock: Ref<'a, Map<NonNull<NodeInner>, Transition>>,
    iter: MapKeyIter<'a, NonNull<NodeInner>, Transition>,
}

impl<'a> TargetIter<'a> {
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
        let lock = node.0.targets.borrow();
        let ptr = node.0.targets.as_ptr();
        let iter = unsafe { &*ptr }.keys();
        Self { _lock: lock, iter }
    }
}

impl<'a> std::iter::Iterator for TargetIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|node_ptr| unsafe { Node::from_ptr(*node_ptr) })
    }
}

pub struct SymbolTargetIter<'a> {
    _lock: Ref<'a, Map<NonNull<NodeInner>, Transition>>,
    iter: MapIter<'a, NonNull<NodeInner>, Transition>,
}

impl<'a> SymbolTargetIter<'a> {
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
        let lock = node.0.targets.borrow();
        let ptr = node.0.targets.as_ptr();
        let iter = unsafe { &*ptr }.iter();
        Self { _lock: lock, iter }
    }
}

impl<'a> std::iter::Iterator for SymbolTargetIter<'a> {
    type Item = (Node<'a>, TransitionRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(node_ptr, transition)| {
            let node = unsafe { Node::from_ptr(*node_ptr) };
            let transition_ref = unsafe { TransitionRef::new(Ref::clone(&self._lock), transition) };
            (node, transition_ref)
        })
    }
}

pub struct TransitionRef<'a> {
    _lock: Ref<'a, Map<NonNull<NodeInner>, Transition>>,
    transition_ptr: NonNull<Transition>,
}

impl<'a> TransitionRef<'a> {
    unsafe fn new(
        lock: Ref<'a, Map<NonNull<NodeInner>, Transition>>,
        transition: &Transition,
    ) -> Self {
        let transition_ptr =
            unsafe { NonNull::new_unchecked(transition as *const Transition as *mut Transition) };
        Self {
            _lock: lock,
            transition_ptr,
        }
    }
}

impl std::ops::Deref for TransitionRef<'_> {
    type Target = Transition;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.transition_ptr.as_ref() }
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
        if let NfaNode { epsilon_targets } = &node.0.variant {
            let lock = epsilon_targets.borrow();
            let ptr = epsilon_targets.as_ptr();
            let iter = unsafe { &*ptr }.iter();
            return Self { _lock: lock, iter };
        }
        panic!("Iterator over Epsilon targets is possible for NFA nodes only")
    }
}

impl<'a> std::iter::Iterator for EpsilonTargetIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|node_ptr| unsafe { Node::from_ptr(*node_ptr) })
    }
}
