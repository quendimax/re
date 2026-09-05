use crate::{Edge, Graph, Node, Tag};
use recz_adt::{Map, OrdSet, Set, SetU8};
use std::collections::BTreeSet;
use std::rc::Rc;

pub fn determine(nfa: &Graph) -> Graph {
    let dfa = Graph::new();
    let mut determ = Determinator::new(nfa, &dfa);
    determ.determine();
    drop(determ);
    dfa
}

struct Determinator<'d, 'n> {
    nfa: &'n Graph,
    dfa: &'d Graph,
    has_final_nodes: bool,
    conv_table: Map<Rc<BTreeSet<Node<'n>>>, Node<'d>>,
    e_closure_table: Map<Rc<OrdSet<Node<'n>>>, Rc<EClosure<'n>>>,
    final_tags: Map<Node<'d>, Vec<Tag>>,
    stack: Vec<(Node<'n>, Edge<'n>, Node<'n>)>,
    visited: Set<Node<'n>>,
}

struct EClosure<'n> {
    /// Epsilon closure of the nodes.
    nodes: Rc<BTreeSet<Node<'n>>>,

    /// All tags that are going to the final nodes.
    final_tags: Set<Tag>,

    /// The table that contains corresponding to every symbol a set of nodes
    /// that have outgoing edges with this symbols, and tags that are associated
    /// with those nodes.
    sym_table: Map<u8, (Set<Tag>, Rc<OrdSet<Node<'n>>>)>,

    /// If any of `nodes` is a final node, this is `true`.
    is_final: bool,
}

impl<'d, 'n> Determinator<'d, 'n> {
    fn new(nfa: &'n Graph, dfa: &'d Graph) -> Self {
        assert!(
            !nfa.is(dfa),
            "NFA and DFA must be different graph instances"
        );
        assert!(dfa.is_empty(), "DFA must be empty");
        Self {
            nfa,
            dfa,
            has_final_nodes: false,
            conv_table: Map::default(),
            e_closure_table: Map::default(),
            final_tags: Map::default(),
            stack: Vec::with_capacity(nfa.node_count()),
            visited: Set::default(),
        }
    }

    fn determine(&mut self) {
        if self.nfa.is_empty() {
            return;
        }

        for group in self.nfa.groups() {
            self.dfa.group(group.label().clone());
        }

        let start_closure = self.e_close(Rc::new([self.nfa.start_node()].into()));
        self.lambda(start_closure);

        if self.has_final_nodes {
            let epilogue_node = self.dfa.node();
            for node in self.dfa.nodes() {
                if node.is_final() {
                    let edge = node.connect(epilogue_node);
                    edge.add_tags(self.final_tags[&node].iter().copied());
                }
            }
            epilogue_node.epilogize();
        }

        // #[cfg(debug_assertions)]
        self.verify_dfa();
    }

    fn lambda(&mut self, closure: Rc<EClosure<'n>>) -> Node<'d> {
        let dfa_node = self.dfa.node();
        self.conv_table.insert(Rc::clone(&closure.nodes), dfa_node);
        if closure.is_final {
            self.has_final_nodes = true;
            dfa_node.finalize();
            let mut final_tags = Vec::with_capacity(closure.final_tags.len());
            final_tags.extend(closure.final_tags.iter().copied());
            self.final_tags.insert(dfa_node, final_tags);
        }

        for (symbol, (tags, nodes)) in &closure.sym_table {
            let sym_closure = self.e_close(Rc::clone(nodes));
            let sym_dfa_node = self
                .conv_table
                .get(&sym_closure.nodes)
                .copied()
                .unwrap_or_else(|| self.lambda(sym_closure));
            let edge = dfa_node.connect(sym_dfa_node);
            edge.add_symbol(*symbol);
            edge.add_tags(tags.iter().copied());
        }

        dfa_node
    }

    fn e_close(&mut self, nodes: Rc<OrdSet<Node<'n>>>) -> Rc<EClosure<'n>> {
        if let Some(closure) = self.e_closure_table.get(&nodes) {
            return Rc::clone(closure);
        }

        #[allow(clippy::mutable_key_type)]
        let mut e_closure = BTreeSet::default();
        let tag_table = Map::<Node<'n>, Set<Tag>>::default();
        let sym_table = Map::<u8, (Set<Tag>, Rc<OrdSet<Node<'n>>>)>::default();
        let mut is_final = false;

        for node in nodes.raw_iter().copied() {
            e_closure.insert(node);
            is_final |= node.is_final();

            for (edge, target) in node.targets() {
                if edge.is_epsilon() {
                    self.stack.push((target, edge, node));
                } else {
                    for symbol in edge.symbols() {
                        let (sym_tags, closure) = sym_table.entry(symbol).or_default();
                        sym_tags.lazy_extend(edge.tags());
                        closure.insert(target);
                    }
                }
            }

            self.visited.clear();
            while let Some((node, edge, source)) = self.stack.pop() {
                e_closure.insert(node);
                self.visited.insert(node);
                is_final |= node.is_final();

                let mut tags_added = false;
                if edge.tag_count() > 0 {
                    let tags = tag_table.entry(node).or_default();
                    tags_added |= tags.lazy_extend(edge.tags());
                }
                if let Some(source_tags) = tag_table.get(&source) {
                    let tags = tag_table.entry(node).or_default();
                    tags_added |= tags.lazy_extend(source_tags.iter().copied());
                }

                for (edge, target) in node.targets() {
                    if edge.is_epsilon() {
                        if !self.visited.contains(&target) || tags_added {
                            self.stack.push((target, edge, node));
                        }
                    } else {
                        for symbol in edge.symbols() {
                            let (sym_tags, sym_closure) = sym_table.entry(symbol).or_default();
                            if let Some(tags) = tag_table.get(&node) {
                                sym_tags.lazy_extend(tags.iter().copied());
                            }
                            sym_tags.lazy_extend(edge.tags());
                            sym_closure.insert(target);
                        }
                    }
                }
            }
        }

        let final_tags = Set::default();
        for (node, tags) in &tag_table {
            if node.is_final() || node.is_epilogue() {
                final_tags.lazy_extend(tags.iter().copied());
            }
        }

        let closure = self
            .e_closure_table
            .insert(
                nodes,
                Rc::new(EClosure {
                    nodes: Rc::new(e_closure),
                    final_tags,
                    sym_table,
                    is_final,
                }),
            )
            .unwrap();
        Rc::clone(closure)
    }

    fn verify_dfa(&self) {
        for node in self.dfa.nodes() {
            let outgoing_symbols = SetU8::default();
            let mut has_epsilon = false;
            for (edge, target) in node.targets() {
                if edge.is_epsilon() {
                    if has_epsilon {
                        panic!("multiple outgoing epsilon edges are not allowed")
                    }
                    has_epsilon = true;
                    if !target.is_epilogue() {
                        panic!("only the epilogue node can be the target of epsilon edge")
                    }
                    if !node.is_final() {
                        panic!(
                            "only final nodes can have outgoing epsilon edges to the epilogue node"
                        )
                    }
                } else {
                    if outgoing_symbols.is_disjoint(&edge.symbols().into()) {
                        outgoing_symbols.insert_bytes(edge.symbols());
                    } else {
                        let overlap = outgoing_symbols.intersection(&edge.symbols().into());
                        panic!(
                            "DFA node has overlapping symbols: {} for nodes {} and {}",
                            overlap, node, target
                        );
                    }
                }
            }
        }
    }
}
