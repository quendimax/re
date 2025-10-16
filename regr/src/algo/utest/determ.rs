use super::*;
use crate::{Arena, Graph, Inst};
use pretty_assertions::assert_eq;
use redt::{map, tmap, tset};
use std::collections::BTreeMap;

#[test]
fn e_closure() {
    let mut nfa_arena = Arena::new();
    let nfa = Graph::new_in(&mut nfa_arena);

    let a = nfa.node();
    let b = nfa.node();
    let c = nfa.node();
    let d = nfa.node();
    let e = nfa.node();
    let f = nfa.node();

    a.connect(b);
    a.connect(c);
    b.connect(d).merge(1);
    c.connect(e);
    d.connect(f);
    e.connect(f);

    let mut dfa_arena = Arena::new();
    let dfa = Graph::new_in(&mut dfa_arena);

    let mut det = Determinizer::new(&nfa, &dfa);
    assert_eq!(det.e_closure(a), tset![a, b, c, e, f]);
    assert_eq!(det.e_closure(b), tset![b]);
    assert_eq!(det.e_closure(c), tset![c, e, f]);
    assert_eq!(det.e_closure(d), tset![d, f]);
    assert_eq!(det.e_closure(e), tset![e, f]);
    assert_eq!(det.e_closure(f), tset![f]);

    f.connect(b);
    assert_eq!(det.e_closure(f), tset![b, f]);

    f.connect(c);
    assert_eq!(det.e_closure(f), tset![b, c, e, f]);

    assert!(det.inst_map.is_empty());
}

#[test]
fn e_closure_with_tags() {
    let mut nfa_arena = Arena::new();
    let nfa = Graph::new_in(&mut nfa_arena);

    let q = nfa.node();
    let a = nfa.node();
    let b = nfa.node();
    let c = nfa.node();
    let d = nfa.node();
    let e = nfa.node();
    let f = nfa.node();
    let g = nfa.node();

    q.connect(a).merge_instruct(Inst::WritePos(0, 0), None);
    a.connect(b).merge_instruct(Inst::WritePos(1, 1), None);
    a.connect(c).merge_instruct(Inst::WritePos(2, 2), None);
    b.connect(d).merge(1);
    c.connect(e).merge(2);
    d.connect(f).merge_instruct(Inst::InvalidateTag(2), None);
    e.connect(f).merge_instruct(Inst::InvalidateTag(1), None);
    f.connect(g).merge_instruct(Inst::WritePos(3, 3), None);

    let mut dfa_arena = Arena::new();
    let dfa = Graph::new_in(&mut dfa_arena);

    let mut det = Determinizer::new(&nfa, &dfa);
    assert_eq!(det.e_closure(q), tset![q, a, b, c]);
    assert_eq!(
        det.inst_map,
        map! {
            a => tset![Inst::WritePos(0, 0)],
            b => tset![Inst::WritePos(0, 0), Inst::WritePos(1, 1)],
            c => tset![Inst::WritePos(0, 0), Inst::WritePos(2, 2)],
        }
    );

    assert_eq!(det.e_closure(d), tset![d, f, g]);
    assert_eq!(
        BTreeMap::from_iter(det.inst_map.iter().map(|(k, v)| (*k, v.clone()))),
        tmap! {
            a => tset![Inst::WritePos(0, 0)],
            b => tset![Inst::WritePos(0, 0), Inst::WritePos(1, 1)],
            c => tset![Inst::WritePos(0, 0), Inst::WritePos(2, 2)],
            f => tset![Inst::InvalidateTag(2)],
            g => tset![Inst::InvalidateTag(2), Inst::WritePos(3, 3)],
        }
    );

    assert_eq!(det.e_closure(e), tset![e, f, g]);
    assert_eq!(
        BTreeMap::from_iter(det.inst_map.iter().map(|(k, v)| (*k, v.clone()))),
        tmap! {
            a => tset![Inst::WritePos(0, 0)],
            b => tset![Inst::WritePos(0, 0), Inst::WritePos(1, 1)],
            c => tset![Inst::WritePos(0, 0), Inst::WritePos(2, 2)],
            f => tset![Inst::InvalidateTag(1), Inst::InvalidateTag(2)],
            g => tset![Inst::InvalidateTag(1), Inst::InvalidateTag(2), Inst::WritePos(3, 3)],
        }
    );
}
