use crate::algo::{self, VisitResult::*};
use crate::graph::Graph;
use crate::isa::Inst;
use crate::node::Node;
use redt::Map;
use std::collections::BTreeSet;
use std::rc::Rc;

#[allow(clippy::mutable_key_type)]
pub fn determinize(nfa: &Graph<'_>, dfa: &Graph<'_>) {
    let mut determinizer = Determinizer::new(nfa, dfa);
    determinizer.determinize();
}

struct Determinizer<'na, 'ng, 'da, 'dg> {
    nfa: &'ng Graph<'na>,
    dfa: &'dg Graph<'da>,
    inst_map: Map<Node<'na>, BTreeSet<Inst>>,
    convert_map: Map<Rc<BTreeSet<Node<'na>>>, Node<'da>>,
}

#[allow(clippy::mutable_key_type)]
impl<'na, 'ng, 'da, 'dg> Determinizer<'na, 'ng, 'da, 'dg> {
    fn new(nfa: &'ng Graph<'na>, dfa: &'dg Graph<'da>) -> Self {
        Self {
            nfa,
            dfa,
            inst_map: Map::new(),
            convert_map: Map::new(),
        }
    }

    fn determinize(&mut self) {
        let start_node = self.nfa.start_node();
        let start_dfa_node = self.dfa.node();
        let e_closure = Rc::new(self.e_closure(start_node));
        let mut unvisited = Vec::with_capacity(512);
        unvisited.push((Rc::clone(&e_closure), start_dfa_node));
        self.convert_map.insert(e_closure, start_dfa_node);
        while let Some((closure, source_dfa_node)) = unvisited.pop() {
            for symbol in u8::MIN..=u8::MAX {
                let (s_closure, inst_set) = self.closure(&closure, symbol);
                let s_closure = Rc::new(s_closure);
                if !s_closure.is_empty() {
                    let tr = if let Some(target_dfa_node) = self.convert_map.get(&s_closure) {
                        source_dfa_node.connect(*target_dfa_node)
                    } else {
                        let target_dfa_node = self.dfa.node();
                        self.convert_map
                            .insert(Rc::clone(&s_closure), target_dfa_node);
                        unvisited.push((Rc::clone(&s_closure), target_dfa_node));
                        source_dfa_node.connect(target_dfa_node)
                    };
                    tr.merge(symbol);
                    tr.merge_instructs(inst_set, Some(symbol.into()));
                }
            }
        }
    }

    fn e_closure(&mut self, start_node: Node<'na>) -> BTreeSet<Node<'na>> {
        let mut e_closure = BTreeSet::new();
        e_closure.insert(start_node);
        algo::visit_transitions(start_node, |source, tr, target| {
            if tr.is_epsilon() {
                e_closure.insert(target);

                let mut new_set = BTreeSet::new();
                for inst in tr.instructs() {
                    new_set.insert(inst);
                }
                if let Some(instructions) = self.inst_map.get(&source) {
                    for inst in instructions.iter().copied() {
                        new_set.insert(inst);
                    }
                }
                if !new_set.is_empty() {
                    self.inst_map.entry(target).or_default().extend(new_set);
                }

                Recurse
            } else {
                Continue
            }
        });
        e_closure
    }

    fn closure(
        &mut self,
        start_nodes: &BTreeSet<Node<'na>>,
        symbol: u8,
    ) -> (BTreeSet<Node<'na>>, BTreeSet<Inst>) {
        let mut closure = BTreeSet::new();
        let mut inst_set = BTreeSet::new();
        for node in start_nodes.iter().copied() {
            algo::visit_transitions(node, |source, tr, target| {
                if tr.contains(symbol) {
                    let e_closure = self.e_closure(target);
                    closure.extend(e_closure);

                    if let Some(insts) = self.inst_map.get(&source) {
                        inst_set.extend(insts);
                    }
                    inst_set.extend(tr.instructs_for(symbol));
                }
                Continue
            });
        }
        (closure, inst_set)
    }
}

#[cfg(test)]
#[path = "utest/determ.rs"]
mod utest;
