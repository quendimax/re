use redt::range;
use regr::algo;
use regr::{Arena, Epsilon, Graph};

#[test]
fn verify_dfa() {
    let mut arena = Arena::new();
    let nfa = Graph::new_in(&mut arena);
    let a = nfa.node();
    let b = nfa.node();
    let c = nfa.node();
    let d = nfa.node();
    a.connect(a).merge(range(1, 255));
    a.connect(b).merge(Epsilon);
    b.connect(c).merge(b'a');
    c.connect(d).merge(b'b');
    assert!(algo::verify_dfa(&nfa));
}
