use crate::algo::{self, VisitResult::*};
use crate::graph::Graph;
use crate::isa::Inst;
use crate::node::Node;
use redt::{Map, Set};

#[allow(clippy::mutable_key_type)]
pub fn determinize(_nfa: &Graph<'_>, _dfa: &Graph<'_>) {
    todo!()
}

#[allow(clippy::mutable_key_type)]
pub fn e_closure<'a>(nodes: &Set<Node<'a>>) -> (Set<Node<'a>>, Map<Node<'a>, Set<Inst>>) {
    let mut inst_map: Map<Node<'a>, Set<Inst>> = Map::new();
    let mut closure = Set::new();
    for node in nodes.iter().copied() {
        closure.insert(node);
        algo::visit_transitions(node, |source, tr, target| {
            if tr.is_epsilon() {
                closure.insert(target);

                let mut new_set = Set::new();
                for inst in tr.instructs() {
                    new_set.insert(inst);
                }
                if let Some(instructions) = inst_map.get(&source) {
                    for inst in instructions.iter().copied() {
                        new_set.insert(inst);
                    }
                }
                inst_map.insert(target, new_set);

                Recurse
            } else {
                Continue
            }
        });
    }
    (closure, inst_map)
}
