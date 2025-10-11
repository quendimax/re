use redt::range;
use regr::{
    Arena, Graph,
    algo::{self, VisitResult::*},
};

#[test]
fn verify_dfa() {
    let mut arena = Arena::new();
    let nfa = Graph::new_in(&mut arena);
    let a = nfa.node();
    let b = nfa.node();
    let c = nfa.node();
    let d = nfa.node();
    a.connect(a).merge(range(1, 255));
    a.connect(b);
    b.connect(c).merge(b'a');
    c.connect(d).merge(b'b');
    assert!(algo::verify_dfa(&nfa));

    a.connect(b).merge(b'a');
    assert!(!algo::verify_dfa(&nfa));
}

#[test]
fn visit_nodes() {
    let mut arena = Arena::new();
    let gr = Graph::new_in(&mut arena);
    let a = gr.node();
    let b = gr.node();
    let c = gr.node();
    let d = gr.node();
    let e = gr.node();

    a.connect(b);
    b.connect(c);
    c.connect(a);
    a.connect(d);
    e.connect(a);

    let mut vec = Vec::new();
    algo::visit_nodes(a, |node| {
        vec.push(node);
        Recurse
    });
    vec.sort();
    assert_eq!(vec, [a, b, c, d]);

    let mut vec = Vec::new();
    algo::visit_nodes(e, |node| {
        vec.push(node);
        Recurse
    });
    vec.sort();
    assert_eq!(vec, [a, b, c, d, e]);
}

#[test]
fn visit_nodes_in_tree() {
    let mut arena = Arena::new();
    let gr = Graph::new_in(&mut arena);
    let a = gr.node();
    let b = gr.node();
    let c = gr.node();
    let d = gr.node();
    let e = gr.node();
    let f = gr.node();
    let g = gr.node();

    a.connect(b);
    a.connect(c);

    b.connect(d);
    b.connect(e);

    e.connect(f);
    e.connect(g);

    let mut vec = Vec::new();
    algo::visit_nodes(a, |node| {
        vec.push(node);
        if node == b { Continue } else { Recurse }
    });
    vec.sort();
    assert_eq!(vec, [a, b, c]);

    let mut vec = Vec::new();
    algo::visit_nodes(a, |node| {
        vec.push(node);
        if node == e { Continue } else { Recurse }
    });
    vec.sort();
    assert_eq!(vec, [a, b, c, d, e]);
}

#[test]
fn for_each_transition() {
    let mut arena = Arena::new();
    let gr = Graph::new_in(&mut arena);
    let a = gr.node();
    let b = gr.node();
    let c = gr.node();
    let d = gr.node();
    let e = gr.node();

    a.connect(b);
    b.connect(c);
    c.connect(a);
    a.connect(d);
    e.connect(a);

    let mut vec = Vec::new();
    algo::visit_transitions(a, |source, _transition, target| {
        vec.push((source, target));
        Recurse
    });
    vec.sort();
    assert_eq!(vec, [(a, b), (a, d), (b, c), (c, a)]);

    let mut vec = Vec::new();
    algo::visit_transitions(e, |source, _transition, target| {
        vec.push((source, target));
        Recurse
    });
    vec.sort();
    assert_eq!(vec, [(a, b), (a, d), (b, c), (c, a), (e, a)]);
}

#[test]
fn for_each_transition_in_tree() {
    let mut arena = Arena::new();
    let gr = Graph::new_in(&mut arena);
    let a = gr.node();
    let b = gr.node();
    let c = gr.node();
    let d = gr.node();
    let e = gr.node();
    let f = gr.node();
    let g = gr.node();

    a.connect(b);
    a.connect(c);

    b.connect(d);
    b.connect(e);

    e.connect(f);
    e.connect(g);

    let mut vec = Vec::new();
    algo::visit_transitions(a, |source, _, target| {
        vec.push((source, target));
        if target == b { Continue } else { Recurse }
    });
    vec.sort();
    assert_eq!(vec, [(a, b), (a, c)]);

    let mut vec = Vec::new();
    algo::visit_transitions(a, |source, _, target| {
        vec.push((source, target));
        if target == e { Continue } else { Recurse }
    });
    vec.sort();
    assert_eq!(vec, [(a, b), (a, c), (b, d), (b, e)]);
}
