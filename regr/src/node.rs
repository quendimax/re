use crate::adt::Map;
use crate::graph::{AutomatonKind, Graph};
use crate::span::Span;
use crate::symbol::Epsilon;
use crate::transition::Transition;
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::fmt::Write;
use std::ops::Deref;

/// Node for an NFA graph.
///
/// It contains ID (unique within its graph owner). Also it can be connected to
/// another node via [`Transition`]'s.
pub struct Node<'a>(&'a NodeInner<'a>);

pub(crate) struct NodeInner<'a> {
    uid: u64,
    accept: Cell<bool>,
    targets: RefCell<Map<Node<'a>, Transition>>,
    variant: NodeVariant,
}

enum NodeVariant {
    DfaNode {
        occupied_symbols: RefCell<Transition>,
    },
    NfaNode,
}

use NodeVariant::{DfaNode, NfaNode};

/// Public API
impl<'a> Node<'a> {
    pub(crate) const ID_MASK: u64 = (1 << (u64::BITS / 2)) - 1;
    pub(crate) const ID_BITS: u32 = u64::BITS / 2;

    /// Returns the node's identifier that is unique within its owner.
    #[inline]
    pub fn nid(self) -> u64 {
        self.0.uid & Self::ID_MASK
    }

    /// Returns the node's graph owner identifier.
    #[inline]
    pub fn gid(self) -> u64 {
        self.0.uid >> Self::ID_BITS
    }

    /// Returns the node's identifier unique within the running process.
    #[inline]
    pub fn uid(self) -> u64 {
        self.0.uid
    }

    pub fn kind(self) -> AutomatonKind {
        match self.0.variant {
            DfaNode { .. } => AutomatonKind::DFA,
            NfaNode => AutomatonKind::NFA,
        }
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
    pub fn targets(self) -> impl Deref<Target = Map<Node<'a>, Transition>> {
        self.0.targets.borrow()
    }

    /// Iterates over epsilon target nodes, i.e. nodes that this node is
    /// connected to with Epsilon transition.
    pub fn for_each_epsilon_target(self, f: impl FnMut(Node<'a>)) {
        let mut f = f;
        if matches!(self.0.variant, DfaNode { .. }) {
            panic!("iteration over Epsilon targets is possible for NFA nodes only");
        }
        for (target, transition) in self.0.targets.borrow().iter() {
            if transition.contains(Epsilon) {
                f(*target);
            }
        }
    }

    /// Collects epsilon target nodes, i.e. nodes that this node is
    /// connected to with Epsilon transition.
    pub fn collect_epsilon_targets<B: FromIterator<Node<'a>>>(self) -> B {
        let targets = self.0.targets.borrow();
        let iter = targets.iter().filter_map(|(target, tr)| {
            if tr.contains(Epsilon) {
                Some(*target)
            } else {
                None
            }
        });
        FromIterator::from_iter(iter)
    }
}

/// Crate API
impl<'a> Node<'a> {
    pub(crate) fn new_inner(uid: u64, graph: &Graph<'a>) -> NodeInner<'a> {
        match graph.kind() {
            AutomatonKind::NFA => NodeInner {
                uid,
                accept: Cell::new(false),
                targets: Default::default(),
                variant: NfaNode,
            },
            AutomatonKind::DFA => NodeInner {
                uid,
                accept: Cell::new(false),
                targets: Default::default(),
                variant: DfaNode {
                    occupied_symbols: Default::default(),
                },
            },
        }
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
            for (target_node, transition) in node.targets().iter() {
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
            node.for_each_epsilon_target(|target_node| {
                closure_impl(target_node, closure);
            });
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
                if let DfaNode { occupied_symbols } = &self.0.variant {
                    let mut occupied_symbols = occupied_symbols.borrow_mut();
                    if occupied_symbols.contains(with) {
                        panic!("connection {with:?} already exists; DFA node can't have more");
                    }
                    occupied_symbols.merge(with);
                }

                #[allow(clippy::mutable_key_type)]
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
    };
}

impl_connect!(u8);
impl_connect!(&u8);
impl_connect!(Span);
impl_connect!(&Span);
impl_connect!(&Transition);

impl<'a> ConnectOp<'a, std::ops::RangeInclusive<u8>> for Node<'a> {
    #[inline]
    fn connect(self, to: Node<'a>, with: std::ops::RangeInclusive<u8>) {
        ConnectOp::connect(self, to, Span::from(with))
    }
}

impl<'a> ConnectOp<'a, Epsilon> for Node<'a> {
    fn connect(self, to: Node<'a>, with: Epsilon) {
        match &self.0.variant {
            DfaNode { .. } => {
                panic!("NFA nodes can't be connected with Epsilon");
            }
            NfaNode => {
                #[allow(clippy::mutable_key_type)]
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

impl<'a> std::convert::From<&'a NodeInner<'a>> for Node<'a> {
    fn from(inner: &'a NodeInner<'a>) -> Self {
        Self(inner)
    }
}

impl<'a> std::convert::From<&'a mut NodeInner<'a>> for Node<'a> {
    fn from(inner: &'a mut NodeInner<'a>) -> Self {
        Self(inner)
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
