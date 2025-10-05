use crate::arena::Arena;
use crate::symbol::Epsilon;
use crate::transition::Transition;
use redt::Map;
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
    is_final: Cell<bool>,
    targets: RefCell<Map<Node<'a>, Transition<'a>>>,
    arena: &'a Arena,
}

/// Public API
impl<'a> Node<'a> {
    pub(crate) const ID_MASK: u64 = (1 << (u64::BITS / 2)) - 1;
    pub(crate) const ID_BITS: u32 = u64::BITS / 2;

    /// Returns the node's identifier that is unique within its owner.
    #[inline]
    pub fn nid(self) -> u32 {
        (self.0.uid & Self::ID_MASK) as u32
    }

    /// Returns the node's graph owner identifier.
    #[inline]
    pub fn gid(self) -> u32 {
        (self.0.uid >> Self::ID_BITS) as u32
    }

    /// Returns the node's identifier unique within the running process.
    #[inline]
    pub fn uid(self) -> u64 {
        self.0.uid
    }

    /// Checks if the node is a final N/DFA state.
    #[inline]
    pub fn is_final(self) -> bool {
        self.0.is_final.get()
    }

    /// Make the node final.
    pub fn finalize(self) -> Self {
        self.0.is_final.set(true);
        self
    }

    /// Make the node non-final.
    pub fn definalize(self) -> Self {
        self.0.is_final.set(false);
        self
    }

    /// Arena owner of this node.
    #[inline]
    pub fn arena(&self) -> &'a Arena {
        self.0.arena
    }

    /// Connects this node to another node with an Epsilon transition.
    pub fn connect(self, to: Node<'a>) -> Transition<'a> {
        assert_eq!(
            self.gid(),
            to.gid(),
            "only nodes of the same graph can be joint"
        );
        let mut targets = self.0.targets.borrow_mut();
        if let Some(tr) = targets.get(&to) {
            *tr
        } else {
            let tr = Transition::new(self, to);
            targets.insert(to, tr);
            tr
        }
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
    pub fn targets(self) -> impl Deref<Target = Map<Node<'a>, Transition<'a>>> {
        self.0.targets.borrow()
    }

    /// Calls a function for each target node, i.e. nodes that this node is
    /// connected to.
    ///
    /// This iterator walks over pairs (`Node`, `TransitionRef`).
    #[inline]
    pub fn for_each_target(self, f: impl FnMut(Node<'a>, Transition<'a>)) {
        let mut f = f;
        for (target, tr) in self.0.targets.borrow().iter() {
            f(*target, *tr);
        }
    }

    /// Iterates over epsilon target nodes, i.e. nodes that this node is
    /// connected to with Epsilon transition.
    pub fn for_each_epsilon_target(self, f: impl FnMut(Node<'a>)) {
        let mut f = f;
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
    pub(crate) fn new_in(arena: &'a Arena, gid: u32, nid: u32) -> Node<'a> {
        let uid = ((gid as u64) << Node::ID_BITS) | nid as u64;
        arena.alloc_node_with(|| NodeInner {
            uid,
            is_final: Cell::new(false),
            targets: Default::default(),
            arena,
        })
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
                if self.is_final() {
                    f.write_str("node((")?;
                } else {
                    f.write_str("node(")?;
                }
                std::fmt::$trait::fmt(&self.nid(), f)?;
                if self.is_final() {
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
