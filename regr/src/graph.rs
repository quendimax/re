use crate::arena::Arena;
use crate::node::{ClosureOp, Node};
use crate::symbol::Epsilon;
use crate::tag::Tag;
use redt::{Map, Set};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
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
    tag_groups: RefCell<Map<u32, (Tag, Tag)>>, // label -> (open_tag, close_tag)
}

static NEXT_GRAPH_ID: AtomicU32 = AtomicU32::new(1);

impl<'a> Graph<'a> {
    pub fn new_in(arena: &'a mut Arena, kind: AutomatonKind) -> Self {
        let gid = NEXT_GRAPH_ID.fetch_add(1, Ordering::Relaxed) as u64;
        if gid == 0 {
            panic!("graph id overflow");
        }

        arena.bind_graph(gid);

        Self {
            gid,
            arena,
            next_nid: Cell::new(0),
            start_node: Cell::new(None),
            kind,
            tag_groups: RefCell::new(Map::new()),
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

    /// Returns the start node of the graph. If the graph is empty, creates a
    /// node, and returns it.
    #[inline]
    pub fn start_node(&self) -> Node<'a> {
        self.start_node.get().unwrap_or_else(|| self.node())
    }

    /// Returns true if the graph is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.start_node.get().is_none()
    }

    /// Arena owner of the graph's nodes and transitions.
    #[inline]
    pub fn arena(&self) -> &'a Arena {
        self.arena
    }

    pub fn add_tag_group(&self, label: u32, open_tag: Tag, close_tag: Tag) {
        let mut tag_table = self.tag_groups.borrow_mut();
        tag_table.entry(label).or_insert((open_tag, close_tag));
    }

    pub fn tag_group(&self, label: u32) -> Option<(Tag, Tag)> {
        self.tag_groups.borrow().get(&label).cloned()
    }

    /// Builds a new DFA from `self` using determinization algorithm.
    ///
    /// If instead of NFA, this graph is a DFA, this method just builds a clone
    /// of it.
    pub fn determinize_in<'d>(&self, arena: &'d mut Arena) -> Graph<'d> {
        type ConvertMap<'n, 'd> = BTreeMap<Rc<BTreeSet<Node<'n>>>, Node<'d>>;

        struct Lambda<'a, 'n, 'd> {
            #[allow(clippy::mutable_key_type)]
            convert_map: ConvertMap<'n, 'd>,
            dfa: &'a Graph<'d>,
        }
        impl<'a, 'n, 'd> Lambda<'a, 'n, 'd> {
            fn convert(&mut self, nfa_closure: Rc<BTreeSet<Node<'n>>>) -> Node<'d> {
                if let Some(dfa_node) = self.convert_map.get(&nfa_closure) {
                    return *dfa_node;
                }

                let dfa_node = self.dfa.node();
                for nfa_node in nfa_closure.iter() {
                    if nfa_node.is_final() {
                        dfa_node.finalize();
                        break;
                    }
                }
                self.convert_map.insert(Rc::clone(&nfa_closure), dfa_node);

                for symbol in u8::MIN..=u8::MAX {
                    let symbol_closure = Rc::new(nfa_closure.closure(symbol));
                    if !symbol_closure.is_empty() {
                        let target_dfa_node = self.convert(symbol_closure);
                        let tr = dfa_node.connect(target_dfa_node);
                        tr.merge(symbol);
                    }
                }
                dfa_node
            }
        }

        let dfa = Graph::dfa_in(arena);
        let start_e_closure = Rc::new(self.start_node().closure(Epsilon));
        Lambda {
            convert_map: ConvertMap::new(),
            dfa: &dfa,
        }
        .convert(start_e_closure);
        dfa
    }

    /// Visits each node of the graph, i.e. every node reachable from the start
    /// node.
    pub fn for_each_node<F>(&self, f: F)
    where
        F: FnMut(Node<'a>),
    {
        struct Lambda<'a, F: FnMut(Node<'a>)> {
            visited: Set<Node<'a>>,
            handler: F,
        }
        impl<'a, F: FnMut(Node<'a>)> Lambda<'a, F> {
            fn visit(&mut self, node: Node<'a>) {
                self.visited.insert(node);
                (self.handler)(node);
                for target in node.targets().keys() {
                    if !self.visited.contains(target) {
                        self.visit(*target);
                    }
                }
            }
        }
        Lambda {
            visited: Set::new(),
            handler: f,
        }
        .visit(self.start_node());
    }
}

impl std::ops::Drop for Graph<'_> {
    fn drop(&mut self) {
        self.arena.unbind_graph();
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl ::std::fmt::$trait for Graph<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                struct Lambda<'b, 'c> {
                    first: bool,
                    f: &'b mut std::fmt::Formatter<'c>,
                    visited: ::redt::Set<u64>,
                }
                impl<'a, 'b, 'c> Lambda<'b, 'c> {
                    fn call(&mut self, node: Node<'a>) -> std::fmt::Result {
                        self.visited.insert(node.uid());
                        if self.first {
                            self.first = false;
                        } else {
                            self.f.write_char('\n')?;
                        }
                        let mut is_empty = true;
                        ::std::fmt::$trait::fmt(&node, self.f)?;
                        self.f.write_str(" {")?;
                        let refer = node.targets();
                        let mut targets: Vec<_> = refer.iter().collect();
                        targets.sort_by_key(|(target, _)| target.uid()); // make order consistent
                        for (target, transition) in targets.iter() {
                            self.f.write_str("\n    ")?;
                            ::std::fmt::$trait::fmt(transition, self.f)?;
                            self.f.write_str(" -> ")?;
                            if node == **target {
                                self.f.write_str("self")?;
                            } else {
                                ::std::fmt::$trait::fmt(&target, self.f)?;
                            }
                            for inst in transition.instructs() {
                                self.f.write_str("\n        ")?;
                                write!(self.f, "{inst}")?;
                            }
                            is_empty = false;
                        }
                        if !is_empty {
                            self.f.write_char('\n')?;
                        }
                        self.f.write_char('}')?;
                        for target in node.targets().keys().copied() {
                            if !self.visited.contains(&target.uid()) {
                                self.call(target)?;
                            }
                        }
                        Ok(())
                    }
                }
                if let Some(start_node) = self.start_node.get() {
                    Lambda {
                        first: true,
                        f,
                        visited: ::redt::Set::new(),
                    }
                    .call(start_node)
                } else {
                    Ok(())
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

#[cfg(test)]
mod utest {
    use super::*;

    #[test]
    #[should_panic(expected = "graph id overflow")]
    fn graph_ctor_panic() {
        NEXT_GRAPH_ID.store(u32::MAX, Ordering::Relaxed);
        let mut arena = Arena::new();
        _ = Graph::nfa_in(&mut arena);
        _ = Graph::nfa_in(&mut arena);
    }
}
