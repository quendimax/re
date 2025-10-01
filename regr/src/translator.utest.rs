use super::{Translator, pair};
use crate::arena::Arena;
use crate::graph::Graph;
use crate::tag::TagBank;
use pretty_assertions::assert_eq;
use redt::{Range, SetU8, lit};
use resy::Hir;

#[test]
fn translate_literal() {
    fn tr(literal: &[u8]) -> String {
        let mut arena = Arena::new();
        let graph = Graph::nfa_in(&mut arena);
        let translator = Translator::new(&graph);
        let pair = pair(graph.node(), graph.node());
        let mut tag = None;
        translator.translate_literal(literal, pair, &mut tag);
        graph.to_string()
    }

    assert_eq!(
        tr(b""),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
        )
    );
    assert_eq!(
        tr(b"ab"),
        lit!(
            ///node(0) {
            ///    ['a'] -> node(2)
            ///}
            ///node(2) {
            ///    ['b'] -> node(1)
            ///}
            ///node(1) {}
        )
    );
}

#[test]
fn translate_class() {
    fn tr(set: &SetU8) -> String {
        let mut arena = Arena::new();
        let graph = Graph::nfa_in(&mut arena);
        let translator = Translator::new(&graph);
        let pair = pair(graph.node(), graph.node());
        let mut tag = None;
        translator.translate_class(set, pair, &mut tag);
        graph.to_string()
    }

    let mut set = SetU8::new();
    set.include(Range::new(4, 50));
    set.include(100);
    assert_eq!(
        tr(&set),
        lit!(
            ///node(0) {
            ///    [04h-'2' | 'd'] -> node(1)
            ///}
            ///node(1) {}
        )
    );
}

#[test]
fn translate_repeat() {
    fn tr(repeat: &Hir) -> String {
        assert!(repeat.is_repeat());
        let mut arena = Arena::new();
        let graph = Graph::nfa_in(&mut arena);
        let mut translator = Translator::new(&graph);
        let pair = pair(graph.node(), graph.node());
        let Hir::Repeat(repeat) = repeat else {
            unreachable!()
        };
        let mut tag = None;
        translator.translate_repeat(repeat, pair, &mut tag);
        graph.to_string()
    }

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 0, None);
    assert_eq!(
        tr(&hir),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(2)
            ///}
            ///node(1) {}
            ///node(2) {
            ///    ['a'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(2)
            ///}
        )
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 1, None);
    assert_eq!(
        tr(&hir),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///}
            ///node(2) {
            ///    ['a'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(2)
            ///}
            ///node(1) {}
        )
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 3, None);
    assert_eq!(
        tr(&hir),
        lit!(
            ///node(0) {
            ///    ['a'] -> node(2)
            ///}
            ///node(2) {
            ///    ['a'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(4)
            ///}
            ///node(4) {
            ///    ['a'] -> node(5)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(4)
            ///}
            ///node(1) {}
        )
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 0, Some(0));
    assert_eq!(
        tr(&hir),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
        )
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 3, Some(3));
    assert_eq!(
        tr(&hir),
        lit!(
            ///node(0) {
            ///    ['a'] -> node(2)
            ///}
            ///node(2) {
            ///    ['a'] -> node(3)
            ///}
            ///node(3) {
            ///    ['a'] -> node(1)
            ///}
            ///node(1) {}
        )
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 1, Some(3));
    assert_eq!(
        tr(&hir),
        lit!(
            ///node(0) {
            ///    ['a'] -> node(2)
            ///}
            ///node(2) {
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(3)
            ///}
            ///node(1) {}
            ///node(3) {
            ///    ['a'] -> node(4)
            ///}
            ///node(4) {
            ///    [Epsilon] -> node(5)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(1)
            ///    [Epsilon] -> node(6)
            ///}
            ///node(6) {
            ///    ['a'] -> node(7)
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(8)
            ///}
            ///node(8) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
}

#[test]
#[should_panic(expected = "invalid repetition counters: {3,2}")]
fn translate_repeat_fails() {
    let literal = Hir::literal("a");
    let repeat = Hir::repeat(literal, 3, Some(2));
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let mut translator = Translator::new(&graph);
    let sub = pair(graph.node(), graph.node());
    let Hir::Repeat(repeat) = repeat else {
        unreachable!()
    };
    let mut tag = None;
    translator.translate_repeat(&repeat, sub, &mut tag);
}

#[test]
fn translate_concat() {
    let concat = Hir::concat([]);
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let translator = Translator::new(&graph);
    let sub = pair(graph.node(), graph.node());
    let Hir::Literal(concat) = concat else {
        unreachable!()
    };
    let mut tag = None;
    translator.translate_literal(&concat, sub, &mut tag);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
        )
    );

    let concat = Hir::concat([Hir::literal("a"), Hir::literal("b"), Hir::literal("c")]);
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let mut translator = Translator::new(&graph);
    let sub = pair(graph.node(), graph.node());
    let Hir::Concat(concat) = concat else {
        unreachable!()
    };
    let mut tag = None;
    translator.translate_concat(&concat, sub, &mut tag);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    ['a'] -> node(2)
            ///}
            ///node(2) {
            ///    ['b'] -> node(3)
            ///}
            ///node(3) {
            ///    ['c'] -> node(1)
            ///}
            ///node(1) {}
        )
    );
}

#[test]
fn translate_disjunct() {
    let disjunct = Hir::disjunct([Hir::literal("a"), Hir::literal("b"), Hir::literal("c")]);
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let mut translator = Translator::new(&graph);
    let sub = pair(graph.node(), graph.node());
    let Hir::Disjunct(disjunct) = disjunct else {
        unreachable!()
    };
    let mut tag = None;
    translator.translate_disjunct(&disjunct, sub, &mut tag);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///    [Epsilon] -> node(4)
            ///    [Epsilon] -> node(6)
            ///}
            ///node(2) {
            ///    ['a'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
            ///node(4) {
            ///    ['b'] -> node(5)
            ///}
            ///node(5) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(6) {
            ///    ['c'] -> node(7)
            ///}
            ///node(7) {
            ///    [Epsilon] -> node(1)
            ///}
        )
    );
}

#[test]
fn translate_group() {
    let group = Hir::group(1, Hir::empty());
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let mut translator = Translator::new(&graph);
    let sub = pair(graph.node(), graph.node());
    let Hir::Group(group) = group else {
        unreachable!()
    };
    let mut tag = None;
    translator.translate_group(&group, sub, &mut tag);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///        wrpos t0/r0
            ///}
            ///node(2) {
            ///    [Epsilon] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
        )
    );
    let (open_tag, close_tag) = graph.tag_group(1).unwrap();
    assert_eq!(open_tag.to_string(), "a-tag(id=0, reg=0)");
    assert_eq!(close_tag.to_string(), "r-tag(id=1, start_tag=0, offset=0)");

    let group = Hir::group(1, Hir::literal("a"));
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let mut translator = Translator::new(&graph);
    let sub = pair(graph.node(), graph.node());
    let Hir::Group(group) = group else {
        unreachable!()
    };
    let mut tag = None;
    translator.translate_group(&group, sub, &mut tag);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///        wrpos t0/r0
            ///}
            ///node(2) {
            ///    ['a'] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
        )
    );
    let (open_tag, close_tag) = graph.tag_group(1).unwrap();
    assert_eq!(open_tag.to_string(), "a-tag(id=0, reg=0)");
    assert_eq!(close_tag.to_string(), "r-tag(id=1, start_tag=0, offset=1)");

    let group = Hir::group(1, Hir::empty());
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let mut translator = Translator::new(&graph);
    let sub = pair(graph.node(), graph.node());
    let Hir::Group(group) = group else {
        unreachable!()
    };
    let mut tag_bank = TagBank::new();
    let abs = tag_bank.absolute();
    let mut tag = Some(tag_bank.relative(abs, 0));
    translator.translate_group(&group, sub, &mut tag);
    assert_eq!(
        graph.to_string(),
        lit!(
            ///node(0) {
            ///    [Epsilon] -> node(2)
            ///}
            ///node(2) {
            ///    [Epsilon] -> node(3)
            ///}
            ///node(3) {
            ///    [Epsilon] -> node(1)
            ///}
            ///node(1) {}
        )
    );
    let (open_tag, close_tag) = graph.tag_group(1).unwrap();
    assert_eq!(open_tag.to_string(), "r-tag(id=1, start_tag=0, offset=0)");
    assert_eq!(close_tag.to_string(), "r-tag(id=0, start_tag=0, offset=0)");
}
