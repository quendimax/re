use crate::adt::{Map, MapIter};
use crate::graph::{AutomatonKind, Graph};
use crate::range::Range;
use crate::symbol::Epsilon;
use crate::transition::Transition;
use std::cell::{Cell, UnsafeCell};
use std::collections::BTreeSet;
use std::fmt::Write;
use std::iter::FilterMap;
use std::ptr::NonNull;

/// Integer type that represents node identifier. It is expected that it is
/// unique within a graph.
pub type NodeId = u32;

/// Integer type that represents node identifier. It is expected that it is
/// unique both within a graph and in the running process.
pub type UniqId = u64;

/// Node for an NFA graph.
///
/// It contains ID (unique within its graph owner). Also it can be connected to
/// another node via [`Transition`]'s.
pub struct Node<'a>(&'a NodeInner);

pub(crate) struct NodeInner {
    node_id: NodeId,
    graph_id: NodeId,
    owner: NonNull<Graph>,
    borrow: Cell<BorrowFlag>,
    accept: Cell<bool>,
    targets: UnsafeCell<Map<NonNull<NodeInner>, Transition>>,
    variant: NodeVariant,
}

enum NodeVariant {
    DfaNode {
        occupied_symbols: UnsafeCell<Transition>,
    },
    NfaNode,
}

use NodeVariant::{DfaNode, NfaNode};

/// Public API
impl<'a> Node<'a> {
    /// Returns the node's identifier that is unique within its graph owner.
    #[inline]
    pub fn nid(self) -> NodeId {
        self.0.node_id
    }

    /// Returns the node's graph owner identifier.
    #[inline]
    pub fn gid(self) -> NodeId {
        self.0.graph_id
    }

    /// Returns the node's identifier unique within the running process.
    #[inline]
    pub fn uid(self) -> UniqId {
        let gid = (self.0.graph_id as UniqId) << NodeId::BITS;
        let nid = self.0.node_id as UniqId;
        gid | nid
    }

    /// Checks if the node is an DFA node.
    pub fn is_dfa(self) -> bool {
        matches!(self.0.variant, DfaNode { .. })
    }

    /// Checks if the node is an NFA node.
    pub fn is_nfa(self) -> bool {
        matches!(self.0.variant, NfaNode)
    }

    /// Checks if the node is an acceptable N/DFA state.
    #[inline]
    pub fn is_acceptable(self) -> bool {
        self.0.accept.get()
    }

    /// Make the node acceptable.
    pub fn acceptize(self) -> Self {
        self.0.accept.set(true);
        self
    }

    /// Make the node unacceptable.
    pub fn disacceptize(self) -> Self {
        self.0.accept.set(false);
        self
    }

    /// Returns a reference to the graph that is an owner of this node.
    pub fn owner(self) -> &'a Graph {
        unsafe { self.0.owner.as_ref() }
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
        assert_eq!(
            self.gid(),
            to.gid(),
            "only nodes of the same graph can be joint"
        );
        ConnectOp::connect(self, to, with);
    }

    #[allow(clippy::mutable_key_type)]
    pub fn closure<T>(self, symbol: T) -> BTreeSet<Node<'a>>
    where
        Self: ClosureOp<'a, T>,
    {
        ClosureOp::closure(&self, symbol)
    }

    /// Returns an iterator over target nodes, i.e. nodes that this node is
    /// connected to.
    ///
    /// This iterator walks over pairs (`Node`, `TransitionRef`).
    #[inline]
    pub fn targets(self) -> TargetIter<'a> {
        TargetIter::new(self)
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
    pub(crate) fn new_inner(node_id: u32, graph: &'a Graph) -> NodeInner {
        match graph.kind() {
            AutomatonKind::NFA => NodeInner {
                graph_id: graph.gid(),
                node_id,
                owner: graph.into(),
                borrow: Cell::new(0),
                accept: Cell::new(false),
                targets: Default::default(),
                variant: NfaNode,
            },
            AutomatonKind::DFA => NodeInner {
                graph_id: graph.gid(),
                node_id,
                owner: graph.into(),
                borrow: Cell::new(0),
                accept: Cell::new(false),
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

pub trait ClosureOp<'a, T> {
    #[allow(clippy::mutable_key_type)]
    fn closure(&self, symbol: T) -> BTreeSet<Node<'a>>;
}

impl<'a> ClosureOp<'a, u8> for Node<'a> {
    #[allow(clippy::mutable_key_type)]
    fn closure(&self, symbol: u8) -> BTreeSet<Node<'a>> {
        let e_closure = self.closure(Epsilon);
        e_closure.closure(symbol)
    }
}

impl<'a> ClosureOp<'a, u8> for BTreeSet<Node<'a>> {
    #[allow(clippy::mutable_key_type)]
    fn closure(&self, symbol: u8) -> BTreeSet<Node<'a>> {
        let mut closure = BTreeSet::new();
        for node in self.iter() {
            for (target_node, transition) in node.targets() {
                if transition.contains(symbol) {
                    let e_closure = target_node.closure(Epsilon);
                    closure.extend(e_closure);
                }
            }
        }
        closure
    }
}

impl<'a> ClosureOp<'a, Epsilon> for Node<'a> {
    #[allow(clippy::mutable_key_type)]
    fn closure(&self, _: Epsilon) -> BTreeSet<Node<'a>> {
        let mut closure = BTreeSet::new();
        fn closure_impl<'a>(node: Node<'a>, closure: &mut BTreeSet<Node<'a>>) {
            if closure.contains(&node) {
                return;
            }
            closure.insert(node);
            for target_node in node.epsilon_targets() {
                closure_impl(target_node, closure);
            }
        }
        closure_impl(*self, &mut closure);
        closure
    }
}

pub trait ConnectOp<'a, T> {
    fn connect(self, to: Node<'a>, with: T);
}

macro_rules! impl_connect {
    ($symty:ty) => {
        impl<'a> ConnectOp<'a, $symty> for Node<'a> {
            fn connect(self, to: Node<'a>, with: $symty) {
                _ = BorrowMut::new(&self.0.borrow);
                if let DfaNode { occupied_symbols } = &self.0.variant {
                    let occupied_symbols = unsafe { occupied_symbols.get().as_mut() }.unwrap();
                    if occupied_symbols.contains(with) {
                        panic!("connection {with:?} already exists; DFA node can't have more");
                    }
                    occupied_symbols.merge(with);
                }

                let to = to.as_ptr();
                let targets = unsafe { self.0.targets.get().as_mut() }.unwrap();
                if let Some(tr) = targets.get_mut(&to) {
                    tr.merge(with);
                } else {
                    let mut tr = Transition::default();
                    tr.merge(with);
                    targets.insert(to, tr);
                }
            }
        }
    };
}

impl_connect!(u8);
impl_connect!(Range);
impl_connect!(&Transition);

impl<'a> ConnectOp<'a, std::ops::RangeInclusive<u8>> for Node<'a> {
    #[inline]
    fn connect(self, to: Node<'a>, with: std::ops::RangeInclusive<u8>) {
        ConnectOp::connect(self, to, Range::from(with))
    }
}

impl<'a> ConnectOp<'a, Epsilon> for Node<'a> {
    fn connect(self, to: Node<'a>, with: Epsilon) {
        match &self.0.variant {
            DfaNode { .. } => {
                panic!("NFA nodes can't be connected with Epsilon");
            }
            NfaNode => {
                _ = BorrowMut::new(&self.0.borrow);
                let to = to.as_ptr();
                let targets = unsafe { self.0.targets.get().as_mut() }.unwrap();
                if let Some(tr) = targets.get_mut(&to) {
                    tr.merge(with);
                } else {
                    let mut tr = Transition::default();
                    tr.merge(with);
                    targets.insert(to, tr);
                }
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
        self.uid().eq(&other.uid())
    }
}

impl std::cmp::Ord for Node<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.uid().cmp(&other.uid())
    }
}

impl std::cmp::PartialOrd for Node<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::hash::Hash for Node<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid().hash(state)
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Node<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if self.is_acceptable() {
                    f.write_str("node((")?;
                } else {
                    f.write_str("node(")?;
                }
                std::fmt::$trait::fmt(&self.nid(), f)?;
                if self.is_acceptable() {
                    f.write_str("))")
                } else {
                    f.write_char(')')
                }
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
    borrow_ref: BorrowRef<'a>,
    iter: MapIter<'a, NonNull<NodeInner>, Transition>,
}

impl<'a> TargetIter<'a> {
    pub fn new(node: Node<'a>) -> Self {
        let borrow_ref = BorrowRef::new(&node.0.borrow);
        let targets = unsafe { node.0.targets.get().as_ref() }.unwrap();
        let iter = targets.iter();
        Self { borrow_ref, iter }
    }
}

impl<'a> std::iter::Iterator for TargetIter<'a> {
    type Item = (Node<'a>, TransitionRef<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(node_ptr, transition)| {
            let node = unsafe { Node::from_ptr(*node_ptr) };
            let transition_ref = unsafe { TransitionRef::new(self.borrow_ref.clone(), transition) };
            (node, transition_ref)
        })
    }
}

pub struct TransitionRef<'a> {
    _borrow_ref: BorrowRef<'a>,
    transition_ptr: NonNull<Transition>,
}

impl<'a> TransitionRef<'a> {
    unsafe fn new(borrow_ref: BorrowRef<'a>, transition: &Transition) -> Self {
        let transition_ptr =
            unsafe { NonNull::new_unchecked(transition as *const Transition as *mut Transition) };
        Self {
            _borrow_ref: borrow_ref,
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

type EpsilonTargetIterFn<'a> = fn((Node<'a>, TransitionRef<'a>)) -> Option<Node<'a>>;

pub struct EpsilonTargetIter<'a> {
    iter: FilterMap<TargetIter<'a>, EpsilonTargetIterFn<'a>>,
}

impl<'a> EpsilonTargetIter<'a> {
    pub fn new(node: Node<'a>) -> Self {
        if matches!(node.0.variant, NfaNode) {
            return Self {
                iter: node
                    .targets()
                    .filter_map(|(n, tr)| if tr.contains(Epsilon) { Some(n) } else { None }),
            };
        }
        panic!("iteration over Epsilon targets is possible for NFA nodes only");
    }
}

impl<'a> std::iter::Iterator for EpsilonTargetIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

type BorrowFlag = isize;
const UNUSED: BorrowFlag = 0;

struct BorrowRef<'a>(&'a Cell<BorrowFlag>);

impl<'a> BorrowRef<'a> {
    fn new(borrow: &'a Cell<BorrowFlag>) -> Self {
        let b = borrow.get().wrapping_add(1);
        if b > UNUSED {
            borrow.set(b);
            Self(borrow)
        } else {
            panic!("already mutably borrowed");
        }
    }
}

impl std::ops::Drop for BorrowRef<'_> {
    fn drop(&mut self) {
        let borrow = self.0.get();
        debug_assert!(borrow > UNUSED);
        self.0.set(borrow - 1);
    }
}

impl Clone for BorrowRef<'_> {
    fn clone(&self) -> Self {
        BorrowRef::new(self.0)
    }
}

struct BorrowMut<'a>(&'a Cell<BorrowFlag>);

impl<'a> BorrowMut<'a> {
    fn new(borrow: &'a Cell<BorrowFlag>) -> Self {
        let b = borrow.get();
        if b == UNUSED {
            borrow.set(b - 1);
            Self(borrow)
        } else {
            panic!("already borrowed");
        }
    }
}

impl std::ops::Drop for BorrowMut<'_> {
    fn drop(&mut self) {
        let borrow = self.0.get().wrapping_add(1);
        debug_assert!(borrow == UNUSED);
        self.0.set(borrow)
    }
}
