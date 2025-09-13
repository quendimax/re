use super::{Translator, pair};
use crate::arena::Arena;
use crate::graph::Graph;
use pretty_assertions::assert_eq;
use redt::{Range, SetU8};
use resy::Hir;

fn dsp<T: std::fmt::Display>(obj: &T) -> String {
    let mut result = String::new();
    for line in format!("{obj}").split('\n') {
        if !line.ends_with('{') && !line.ends_with('}') {
            result.push_str("    ");
        }
        result.push_str(line.trim());
        result.push('\n');
    }
    result.trim().to_string()
}

#[test]
fn translate_literal() {
    fn tr(literal: &[u8]) -> String {
        let mut arena = Arena::new();
        let graph = Graph::nfa_in(&mut arena);
        let translator = Translator::new(&graph);
        let pair = pair(graph.node(), graph.node());
        translator.translate_literal(literal, pair);
        dsp(&graph)
    }

    assert_eq!(
        tr(b""),
        dsp(&"
            node(0) {
                [Epsilon] -> node(1)
            }
            node(1) {}
        ")
    );
    assert_eq!(
        tr(b"ab"),
        dsp(&"
            node(0) {
                ['a'] -> node(2)
            }
            node(2) {
                ['b'] -> node(1)
            }
            node(1) {}
        ")
    );
}

#[test]
fn translate_class() {
    fn tr(set: &SetU8) -> String {
        let mut arena = Arena::new();
        let graph = Graph::nfa_in(&mut arena);
        let translator = Translator::new(&graph);
        let pair = pair(graph.node(), graph.node());
        translator.translate_class(set, pair);
        dsp(&graph)
    }

    let mut set = SetU8::new();
    set.merge_range(Range::new(4, 50));
    set.merge_byte(100);
    assert_eq!(
        tr(&set),
        dsp(&"
            node(0) {
                [04h-'2' | 'd'] -> node(1)
            }
            node(1) {}
        ")
    );
}

#[test]
fn translate_repeat() {
    fn tr(repeat: &Hir) -> String {
        assert!(repeat.is_repeat());
        let mut arena = Arena::new();
        let graph = Graph::nfa_in(&mut arena);
        let translator = Translator::new(&graph);
        let pair = pair(graph.node(), graph.node());
        let Hir::Repeat(repeat) = repeat else {
            unreachable!()
        };
        translator.translate_repeat(repeat, pair);
        dsp(&graph)
    }

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 0, None);
    assert_eq!(
        tr(&hir),
        dsp(&"
            node(0) {
                [Epsilon] -> node(1)
                [Epsilon] -> node(2)
            }
            node(1) {}
            node(2) {
                ['a'] -> node(3)
            }
            node(3) {
                [Epsilon] -> node(1)
                [Epsilon] -> node(2)
            }
        ")
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 1, None);
    assert_eq!(
        tr(&hir),
        dsp(&"
            node(0) {
                [Epsilon] -> node(2)
            }
            node(2) {
                ['a'] -> node(3)
            }
            node(3) {
                [Epsilon] -> node(1)
                [Epsilon] -> node(2)
            }
            node(1) {}
        ")
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 3, None);
    assert_eq!(
        tr(&hir),
        dsp(&"
            node(0) {
                ['a'] -> node(2)
            }
            node(2) {
                ['a'] -> node(3)
            }
            node(3) {
                [Epsilon] -> node(4)
            }
            node(4) {
                ['a'] -> node(5)
            }
            node(5) {
                [Epsilon] -> node(1)
                [Epsilon] -> node(4)
            }
            node(1) {}
        ")
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 0, Some(0));
    assert_eq!(
        tr(&hir),
        dsp(&"
            node(0) {
                [Epsilon] -> node(1)
            }
            node(1) {}
        ")
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 3, Some(3));
    assert_eq!(
        tr(&hir),
        dsp(&"
            node(0) {
                ['a'] -> node(2)
            }
            node(2) {
                ['a'] -> node(3)
            }
            node(3) {
                ['a'] -> node(1)
            }
            node(1) {}
        ")
    );

    let literal = Hir::literal("a");
    let hir = Hir::repeat(literal, 1, Some(3));
    assert_eq!(
        tr(&hir),
        dsp(&"
            node(0) {
                ['a'] -> node(2)
            }
            node(2) {
                [Epsilon] -> node(1)
                [Epsilon] -> node(3)
            }
            node(1) {}
            node(3) {
                ['a'] -> node(4)
            }
            node(4) {
                [Epsilon] -> node(5)
            }
            node(5) {
                [Epsilon] -> node(1)
                [Epsilon] -> node(6)
            }
            node(6) {
                ['a'] -> node(7)
            }
            node(7) {
                [Epsilon] -> node(8)
            }
            node(8) {
                [Epsilon] -> node(1)
            }
        ")
    );
}

#[test]
#[should_panic(expected = "invalid repetition counters: {3,2}")]
fn translate_repeat_fails() {
    let literal = Hir::literal("a");
    let repeat = Hir::repeat(literal, 3, Some(2));
    let mut arena = Arena::new();
    let graph = Graph::nfa_in(&mut arena);
    let translator = Translator::new(&graph);
    let pair = pair(graph.node(), graph.node());
    let Hir::Repeat(repeat) = repeat else {
        unreachable!()
    };
    translator.translate_repeat(&repeat, pair);
}
