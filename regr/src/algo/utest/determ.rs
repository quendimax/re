use super::*;
use crate::{Arena, Graph};
use redt::tset;

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
}
