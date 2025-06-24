use crate::arena::Arena;
use crate::node::{ClosureOp, Node};
use crate::symbol::Epsilon;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AutomatonKind {
    NFA,
    DFA,
}

pub struct Graph<'a> {
    gid: u64,
    arena: &'a Arena,
    next_nid: Cell<u32>,
    start_node: Cell<Option<Node<'a>>>,
    kind: AutomatonKind,
}

static NEXT_GRAPH_ID: AtomicU32 = AtomicU32::new(0);

impl<'a> Graph<'a> {
    pub fn new_in(arena: &'a mut Arena, kind: AutomatonKind) -> Self {
        let gid = NEXT_GRAPH_ID.load(Ordering::Relaxed);
        let gid = gid.checked_add(1).expect("graph id overflow");
        NEXT_GRAPH_ID.store(gid, Ordering::Relaxed);
        let gid = gid as u64;

        arena.bind_graph(gid);

        Self {
            gid,
            arena,
            next_nid: Cell::new(0),
            start_node: Cell::new(None),
            kind,
        }
    }

    /// This graph's ID.
    #[inline]
    pub fn gid(&self) -> u64 {
        self.gid
    }

    /// Creates a new DFA graph.
    pub fn dfa_in(arena: &'a mut Arena) -> Self {
        Self::new_in(arena, AutomatonKind::DFA)
    }

    /// Creates a new NFA graph.
    pub fn nfa_in(arena: &'a mut Arena) -> Self {
        Self::new_in(arena, AutomatonKind::NFA)
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

    /// Creates a new node.
    pub fn node(&self) -> Node<'a> {
        let nid = self.next_nid.replace(
            self.next_nid
                .get()
                .checked_add(1)
                .expect("node id overflow"),
        );
        let node: Node<'a> = self.arena.alloc_node_with(|| {
            let uid = (self.gid << Node::ID_BITS) | nid as u64;
            Node::new_inner(uid, self)
        });
        if self.start_node.get().is_none() {
            self.start_node.set(Some(node));
        }
        node
    }

    #[inline]
    pub fn start_node(&self) -> Node<'a> {
        self.start_node.get().unwrap_or_else(|| self.node())
    }

    pub fn owner(&self) -> &'a Arena {
        self.arena
    }

    /// Builds a new DFA from itself using determinization algorithm.
    ///
    /// If instead of NFA, this graph is a DFA, this method just builds a clone
    /// of it.
    pub fn determine_in<'d>(&self, arena: &'d mut Arena) -> Graph<'d> {
        let dfa = Graph::dfa_in(arena);
        type ConvertMap<'n, 'd> = BTreeMap<Rc<BTreeSet<Node<'n>>>, Node<'d>>;
        #[allow(clippy::mutable_key_type)]
        let mut convert_map: ConvertMap<'a, 'd> = BTreeMap::new();

        struct Lambda<'a, 'n, 'd> {
            convert_map: &'a mut ConvertMap<'n, 'd>,
            dfa: &'a Graph<'d>,
        }

        impl<'a, 'n, 'd> Lambda<'a, 'n, 'd> {
            fn convert(&mut self, nfa_closure: Rc<BTreeSet<Node<'n>>>) -> Node<'d> {
                if let Some(dfa_node) = self.convert_map.get(&nfa_closure) {
                    return *dfa_node;
                }

                let dfa_node = self.dfa.node();
                self.convert_map.insert(Rc::clone(&nfa_closure), dfa_node);

                for symbol in u8::MIN..=u8::MAX {
                    let symbol_closure = Rc::new(nfa_closure.closure(symbol));
                    if !symbol_closure.is_empty() {
                        let target_dfa_node = self.convert(symbol_closure);
                        dfa_node.connect(target_dfa_node, symbol);
                    }
                }
                dfa_node
            }
        }

        let start_e_closure = Rc::new(self.start_node().closure(Epsilon));
        Lambda {
            convert_map: &mut convert_map,
            dfa: &dfa,
        }
        .convert(start_e_closure);
        dfa
    }
}

impl std::ops::Drop for Graph<'_> {
    fn drop(&mut self) {
        self.arena.unbind_graph();
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Graph<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut first = true;
                for node in self.arena.nodes() {
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
