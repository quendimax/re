use redt::range;
use regr::{Arena, Graph, algo};

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
fn for_each_node() {
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
    algo::for_each_node(a, |node| {
        vec.push(node);
        true
    });
    vec.sort();
    assert_eq!(vec, [a, b, c, d]);

    let mut vec = Vec::new();
    algo::for_each_node(e, |node| {
        vec.push(node);
        true
    });
    vec.sort();
    assert_eq!(vec, [a, b, c, d, e]);
}

#[test]
fn for_each_node_in_tree() {
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
    algo::for_each_node(a, |node| {
        vec.push(node);
        node != b
    });
    vec.sort();
    assert_eq!(vec, [a, b, c]);

    let mut vec = Vec::new();
    algo::for_each_node(a, |node| {
        vec.push(node);
        node != e
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
    algo::for_each_transition(a, |source, _transition, target| {
        vec.push((source, target));
        true
    });
    vec.sort();
    assert_eq!(vec, [(a, b), (a, d), (b, c), (c, a)]);

    let mut vec = Vec::new();
    algo::for_each_transition(e, |source, _transition, target| {
        vec.push((source, target));
        true
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
    algo::for_each_transition(a, |source, _, target| {
        vec.push((source, target));
        target != b
    });
    vec.sort();
    assert_eq!(vec, [(a, b), (a, c)]);

    let mut vec = Vec::new();
    algo::for_each_transition(a, |source, _, target| {
        vec.push((source, target));
        target != e
    });
    vec.sort();
    assert_eq!(vec, [(a, b), (a, c), (b, d), (b, e)]);
}
