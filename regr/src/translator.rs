use crate::arena::Arena;
use crate::graph::Graph;
use crate::node::Node;
use crate::symbol::Epsilon;
use redt::SetU8;
use resy::{ConcatHir, DisjunctHir, GroupHir, Hir, RepeatHir};

/// Builds a new NFA graph from the given HIR in the specified arena.
pub fn translate_in<'a>(arena: &'a mut Arena, hir: &Hir) -> Graph<'a> {
    let graph = Graph::dfa_in(arena);
    let translator = Translator::new(&graph);
    let sub_graph = Pair {
        first: graph.start_node(),
        last: graph.node(),
    };
    translator.translate_hir(hir, sub_graph);
    graph
}

struct Pair<'a> {
    first: Node<'a>,
    last: Node<'a>,
}

fn pair<'a>(first: Node<'a>, last: Node<'a>) -> Pair<'a> {
    Pair { first, last }
}

/// Translator for translating a HIR into a NFA.
pub struct Translator<'a, 'g> {
    graph: &'g Graph<'a>,
}

impl<'a, 'g> Translator<'a, 'g> {
    pub fn new(graph: &'g Graph<'a>) -> Self {
        assert!(graph.is_nfa(), "translator can build only NFA graphs");
        Self { graph }
    }

    pub fn translate(&self, hir: &Hir, start_hode: Node<'a>, end_node: Node<'a>) {
        self.translate_hir(hir, pair(start_hode, end_node));
    }

    fn translate_hir(&self, hir: &Hir, sub: Pair<'a>) {
        match hir {
            Hir::Literal(literal) => self.translate_literal(literal, sub),
            Hir::Class(class) => self.translate_class(class, sub),
            Hir::Group(group) => self.translate_group(group, sub),
            Hir::Repeat(repeat) => self.translate_repeat(repeat, sub),
            Hir::Concat(concat) => self.translate_concat(concat, sub),
            Hir::Disjunct(disjunct) => self.translate_disjunct(disjunct, sub),
        }
    }

    fn translate_literal(&self, literal: &[u8], sub: Pair<'a>) {
        let mut first = sub.first;
        for byte in &literal[..literal.len() - 1] {
            let next = self.graph.node();
            first.connect(next).merge(byte);
            first = next;
        }
        if let Some(last_byte) = literal.last() {
            first.connect(sub.last).merge(last_byte);
        } else {
            first.connect(sub.last).merge(Epsilon);
        }
    }

    fn translate_class(&self, class: &SetU8, sub: Pair<'a>) {
        for range in class.ranges() {
            sub.first.connect(sub.last).merge(range);
        }
    }

    fn translate_group(&self, _group: &GroupHir, _sub: Pair<'a>) {
        unimplemented!()
    }

    fn translate_repeat(&self, repeat: &RepeatHir, mut sub: Pair<'a>) {
        match repeat.iter_hint() {
            // Kleene star
            //          ╭────ε────╮
            //          ↓         │
            // (1)──ε─→(2)──'a'─→(3)──ε─→(4)
            //  │                         ↑
            //  ╰────────────ε────────────╯
            //
            (0, None) => {
                let first = self.graph.node();
                let last = self.graph.node();
                self.translate_hir(repeat.inner(), pair(first, last));
                sub.first.connect(first).merge(Epsilon);
                last.connect(sub.last).merge(Epsilon);
                last.connect(first).merge(Epsilon);
                sub.first.connect(sub.last).merge(Epsilon);
            }
            //
            //          ╭────ε────╮
            //          ↓         │
            // (1)──ε─→(2)──'a'─→(3)──ε─→(4)
            //
            (1, None) => {
                let first = self.graph.node();
                let last = self.graph.node();
                self.translate_hir(repeat.inner(), pair(first, last));
                sub.first.connect(first).merge(Epsilon);
                last.connect(sub.last).merge(Epsilon);
                last.connect(first).merge(Epsilon);
            }
            //
            //                               ╭─────ε─────╮
            //                               ↓           │
            // (1)──'a'──...──'a'─→(n)──ε─→(n+1)──'a'─→(n+2)──ε─→(n+3)
            //
            (n, None) => {
                let mut first = sub.first;
                for _ in 1..n {
                    let last = self.graph.node();
                    self.translate_hir(repeat.inner(), pair(first, last));
                    first = last;
                }
                sub.first = first;
                let first = self.graph.node();
                let last = self.graph.node();
                self.translate_hir(repeat.inner(), pair(first, last));
                sub.first.connect(first).merge(Epsilon);
                last.connect(sub.last).merge(Epsilon);
                last.connect(first).merge(Epsilon);
            }
            //
            // (0)──'a'──(1)──'a'──...──'a'─→(n)
            //
            (n, Some(m)) if n == m => {
                if n == 0 {
                    sub.first.connect(sub.last).merge(Epsilon);
                } else if n == m {
                    let mut first = sub.first;
                    for _ in 0..n - 1 {
                        let last = self.graph.node();
                        self.translate_hir(repeat.inner(), pair(first, last));
                        first = last;
                    }
                    self.translate_hir(repeat.inner(), pair(first, sub.last));
                }
            }
            //
            // (0)──'a'─..─'a'─→(n)──ε─→(○)──'a'─→(○)──ε─→(○)──ε─→(○)──'a'──(○)──ε─→(○)──...──ε─→(○)
            //                   │                         │                         │            ↑
            //                   │                         │                         ╰──────ε─────╯
            //                   │                         ╰───────────────────ε──────────────────╯
            //                   ╰────────────────────────────────ε───────────────────────────────╯
            //
            (n, Some(m)) if n < m => {
                let mut first = sub.first;
                for _ in 0..n {
                    let last = self.graph.node();
                    self.translate_hir(repeat.inner(), pair(first, last));
                    first = last;
                }
                for _ in n..m {
                    let mid_one = self.graph.node();
                    first.connect(mid_one).merge(Epsilon);
                    let mid_two = self.graph.node();
                    self.translate_hir(repeat.inner(), pair(mid_one, mid_two));
                    let last = self.graph.node();
                    mid_two.connect(last).merge(Epsilon);
                    first.connect(sub.last).merge(Epsilon);
                    first = last;
                }
                first.connect(sub.last).merge(Epsilon);
            }
            (n, Some(m)) => {
                panic!("invalid repetition counters: {{{n},{m}}}");
            }
        }
    }

    fn translate_concat(&self, concat: &ConcatHir, sub: Pair<'a>) {
        let items = concat.items();
        let mut first = sub.first;
        for hir in &items[..items.len() - 1] {
            let last = self.graph.node();
            self.translate_hir(hir, pair(first, last));
            first = last;
        }
        if let Some(hir) = items.last() {
            self.translate_hir(hir, pair(first, sub.last));
        } else {
            first.connect(sub.last).merge(Epsilon);
        }
    }

    /// ```txt
    ///  ╭───ε──→(○)──'a'─→(○)──ε───╮
    ///  │                          ↓
    /// (○)──ε──→(○)──'b'─→(○)──ε─→(○)
    ///  │                          ↑
    ///  ╰───ε──→(○)──'c'─→(○)──ε───╯
    /// ```
    fn translate_disjunct(&self, disjunct: &DisjunctHir, sub: Pair<'a>) {
        for hir in disjunct.alternatives() {
            let first = self.graph.node();
            let last = self.graph.node();
            self.translate_hir(hir, pair(first, last));
            sub.first.connect(first).merge(Epsilon);
            last.connect(sub.last).merge(Epsilon);
        }
    }
}
