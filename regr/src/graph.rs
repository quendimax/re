use crate::arena::Arena;
use crate::node::{ClosureOp, Node, NodeId, NodeInner};
use crate::symbol::Epsilon;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::ops::Deref;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::Mutex;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AutomatonKind {
    NFA,
    DFA,
}

static GRAPH_ID: Mutex<NodeId> = Mutex::new(0);

#[allow(private_bounds)]
pub struct Graph {
    arena: Arena<NodeInner>,
    graph_id: u32,
    next_id: Cell<NodeId>,
    start_node: Cell<Option<NonNull<NodeInner>>>,
    kind: AutomatonKind,
}

#[allow(private_bounds)]
impl Graph {
    /// Creates a new DFA graph.
    pub fn dfa() -> Self {
        Self::with_capacity(0, AutomatonKind::DFA)
    }

    /// Creates a new NFA graph.
    pub fn nfa() -> Self {
        Self::with_capacity(0, AutomatonKind::NFA)
    }

    /// Returns the graph's kind.
    pub fn kind(&self) -> AutomatonKind {
        self.kind
    }

    /// Checks if this graph is DFA.
    #[inline]
    pub fn is_dfa(&self) -> bool {
        match self.kind {
            AutomatonKind::DFA => true,
            AutomatonKind::NFA => false,
        }
    }

    /// Checks if this graph is NFA.
    #[inline]
    pub fn is_nfa(&self) -> bool {
        match self.kind {
            AutomatonKind::DFA => false,
            AutomatonKind::NFA => true,
        }
    }

    /// This graph's unique ID.
    #[inline]
    pub fn gid(&self) -> NodeId {
        self.graph_id
    }

    /// Creates a new NFA graph with preallocated memory for at least `capacity`
    /// nodes.
    pub fn with_capacity(capacity: usize, kind: AutomatonKind) -> Self {
        let arena = Arena::with_capacity(capacity);

        let mut graph_id = GRAPH_ID.lock().expect("graph id mutex failed");
        let new_graph_id = *graph_id;
        *graph_id = graph_id.checked_add(1).expect("graph id overflow");

        Self {
            arena,
            graph_id: new_graph_id,
            next_id: Cell::new(0),
            start_node: Cell::new(None),
            kind,
        }
    }

    /// Creates a new node.
    pub fn node(&self) -> Node<'_> {
        let new_node_id = self.next_id.get();
        self.next_id
            .set(new_node_id.checked_add(1).expect("node id overflow"));
        let node_mut = self.arena.alloc_with(|| Node::new_inner(new_node_id, self));
        let node = Node::from_mut(node_mut);
        if self.start_node.get().is_none() {
            self.start_node.set(Some(node.as_ptr()));
        }
        node
    }

    #[inline]
    pub fn start_node(&self) -> Node<'_> {
        if let Some(node_ptr) = self.start_node.get() {
            return unsafe { Node::from_ptr(node_ptr) };
        }
        self.node()
    }

    /// Builds a new DFA determining the specified NFA.
    ///
    /// If instead of NFA, a DFA is passed as the argument, this meathod just
    /// builds a clone of it.
    pub fn determined<'n>(nfa: &'n Self) -> Self {
        let dfa = Graph::dfa();
        type ConvertMap<'n, 'd> = BTreeMap<Rc<BTreeSet<Node<'n>>>, Node<'d>>;
        #[allow(clippy::mutable_key_type)]
        let mut convert_map: ConvertMap<'n, '_> = BTreeMap::new();

        #[allow(clippy::mutable_key_type)]
        fn convert_impl<'n, 'd>(
            nfa_closure: Rc<BTreeSet<Node<'n>>>,
            convert_map: &mut ConvertMap<'n, 'd>,
            dfa: &'d Graph,
        ) -> Node<'d> {
            if let Some(dfa_node) = convert_map.get(&nfa_closure) {
                return *dfa_node;
            }

            let dfa_node = dfa.node();
            convert_map.insert(Rc::clone(&nfa_closure), dfa_node);

            for symbol in u8::MIN..=u8::MAX {
                let symbol_closure = Rc::new(nfa_closure.closure(symbol));
                if !symbol_closure.is_empty() {
                    let target_dfa_node = convert_impl(symbol_closure, convert_map, dfa);
                    dfa_node.connect(target_dfa_node, symbol);
                }
            }
            dfa_node
        }

        let start_e_closure = Rc::new(nfa.start_node().closure(Epsilon));
        convert_impl(start_e_closure, &mut convert_map, &dfa);
        dfa
    }
}

impl std::default::Default for Graph {
    #[inline]
    fn default() -> Self {
        Self::nfa()
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Graph {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut first = true;
                for node in self.arena.iter().map(Node::from_ref) {
                    if first {
                        first = false;
                    } else {
                        f.write_char('\n')?;
                    }
                    let mut is_empty = true;
                    std::fmt::$trait::fmt(&node, f)?;
                    f.write_str(" {")?;
                    for (target, transition) in node.targets() {
                        f.write_str("\n    ")?;
                        std::fmt::$trait::fmt(transition.deref(), f)?;
                        f.write_str(" -> ")?;
                        if node == target {
                            f.write_str("self")?;
                        } else {
                            std::fmt::$trait::fmt(&target, f)?;
                        }
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
    };
}

impl_fmt!(std::fmt::Display);
impl_fmt!(std::fmt::Debug);
impl_fmt!(std::fmt::Binary);
impl_fmt!(std::fmt::Octal);
impl_fmt!(std::fmt::UpperHex);
impl_fmt!(std::fmt::LowerHex);
