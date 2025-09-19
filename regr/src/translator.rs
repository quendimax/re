use crate::graph::Graph;
use crate::isa::Inst;
use crate::node::Node;
use crate::symbol::Epsilon;
use crate::tag::Tag;
use redt::SetU8;
use resy::{ConcatHir, DisjunctHir, GroupHir, Hir, RepeatHir};
use std::cell::Cell;

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
    next_reg: Cell<u32>,
}

impl<'a, 'g> Translator<'a, 'g> {
    pub fn new(graph: &'g Graph<'a>) -> Self {
        assert!(graph.is_nfa(), "translator can build only NFA graphs");
        Self {
            graph,
            next_reg: Cell::new(0),
        }
    }

    pub fn translate(&self, hir: &Hir, start_hode: Node<'a>, end_node: Node<'a>) {
        _ = self.translate_hir(hir, pair(start_hode, end_node), None);
    }

    fn translate_hir(&self, hir: &Hir, sub: Pair<'a>, tag: Option<Tag>) -> Summary {
        match hir {
            Hir::Literal(literal) => self.translate_literal(literal, sub),
            Hir::Class(class) => self.translate_class(class, sub),
            Hir::Group(group) => self.translate_group(group, sub, tag),
            Hir::Repeat(repeat) => self.translate_repeat(repeat, sub),
            Hir::Concat(concat) => self.translate_concat(concat, sub, tag),
            Hir::Disjunct(disjunct) => self.translate_disjunct(disjunct, sub),
        }
    }

    fn translate_literal(&self, literal: &[u8], sub: Pair<'a>) -> Summary {
        if literal.is_empty() {
            sub.first.connect(sub.last).merge(Epsilon);
            return Summary::empty();
        }
        let mut first = sub.first;
        for byte in &literal[..literal.len() - 1] {
            let next = self.graph.node();
            first.connect(next).merge(byte);
            first = next;
        }
        let last_byte = literal.last().unwrap();
        first.connect(sub.last).merge(last_byte);
        Summary::empty()
    }

    fn translate_class(&self, class: &SetU8, sub: Pair<'a>) -> Summary {
        for range in class.ranges() {
            sub.first.connect(sub.last).merge(range);
        }
        Summary::empty()
    }

    fn translate_group(&self, group: &GroupHir, sub: Pair<'a>, tag: Option<Tag>) -> Summary {
        let first = self.graph.node();
        let tr_in = sub.first.connect(first);
        tr_in.merge(Epsilon);

        let last = self.graph.node();
        let tr_out = last.connect(sub.last);
        tr_out.merge(Epsilon);

        let (open_tag, close_tag) = self.graph.tag_group(group.label()).unwrap_or_else(|| {
            let open_tag = tag.unwrap_or_else(|| Tag::primary(self.next_reg()));
            let close_tag = if let Some(len) = group.inner().exact_len() {
                Tag::secondary(self.next_reg(), open_tag.starting_tag(), len)
            } else {
                Tag::primary(self.next_reg())
            };
            self.graph.add_tag_group(group.label(), open_tag, close_tag);
            (open_tag, close_tag)
        });

        if open_tag.is_primary() {
            tr_in.merge_instruct(Inst::StorePos(open_tag.id()));
        }
        if close_tag.is_primary() {
            tr_out.merge_instruct(Inst::StorePos(close_tag.id()));
        }

        let mut summary = self.translate_hir(group.inner(), pair(first, last), Some(open_tag));
        summary.used_regs.insert(open_tag.starting_tag());
        summary.used_regs.insert(close_tag.starting_tag());
        summary
    }

    fn translate_repeat(&self, repeat: &RepeatHir, mut sub: Pair<'a>) -> Summary {
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
                sub.first.connect(first).merge(Epsilon);
                last.connect(sub.last).merge(Epsilon);
                last.connect(first).merge(Epsilon);
                sub.first.connect(sub.last).merge(Epsilon);
                self.translate_hir(repeat.inner(), pair(first, last), None)
            }
            //
            //          ╭────ε────╮
            //          ↓         │
            // (1)──ε─→(2)──'a'─→(3)──ε─→(4)
            //
            (1, None) => {
                let first = self.graph.node();
                let last = self.graph.node();
                sub.first.connect(first).merge(Epsilon);
                last.connect(sub.last).merge(Epsilon);
                last.connect(first).merge(Epsilon);
                self.translate_hir(repeat.inner(), pair(first, last), None)
            }
            //
            //                               ╭─────ε─────╮
            //                               ↓           │
            // (1)──'a'──...──'a'─→(n)──ε─→(n+1)──'a'─→(n+2)──ε─→(n+3)
            //
            (n, None) => {
                let mut summary = Summary::empty();
                let mut first = sub.first;
                for _ in 1..n {
                    let last = self.graph.node();
                    let s = self.translate_hir(repeat.inner(), pair(first, last), None);
                    summary.merge(&s);
                    first = last;
                }
                sub.first = first;
                let first = self.graph.node();
                let last = self.graph.node();
                let s = self.translate_hir(repeat.inner(), pair(first, last), None);
                summary.merge(&s);
                sub.first.connect(first).merge(Epsilon);
                last.connect(sub.last).merge(Epsilon);
                last.connect(first).merge(Epsilon);
                summary
            }
            //
            // (0)──'a'──(1)──'a'──...──'a'─→(n)
            //
            (n, Some(m)) if n == m => {
                let mut summary = Summary::empty();
                if n == 0 {
                    sub.first.connect(sub.last).merge(Epsilon);
                } else {
                    let mut first = sub.first;
                    for _ in 0..n - 1 {
                        let last = self.graph.node();
                        let s = self.translate_hir(repeat.inner(), pair(first, last), None);
                        summary.merge(&s);
                        first = last;
                    }
                    let s = self.translate_hir(repeat.inner(), pair(first, sub.last), None);
                    summary.merge(&s);
                }
                summary
            }
            //
            // (0)──'a'─..─'a'─→(n)──ε─→(○)──'a'─→(○)──ε─→(○)──ε─→(○)──'a'──(○)──ε─→(○)──...──ε─→(○)
            //                   │                         │                         │            ↑
            //                   │                         │                         ╰──────ε─────╯
            //                   │                         ╰───────────────────ε──────────────────╯
            //                   ╰────────────────────────────────ε───────────────────────────────╯
            //
            (n, Some(m)) if n < m => {
                let mut summary = Summary::empty();
                let mut first = sub.first;
                for _ in 0..n {
                    let last = self.graph.node();
                    let s = self.translate_hir(repeat.inner(), pair(first, last), None);
                    summary.merge(&s);
                    first = last;
                }
                for _ in n..m {
                    let mid_one = self.graph.node();
                    first.connect(mid_one).merge(Epsilon);
                    let mid_two = self.graph.node();
                    let s = self.translate_hir(repeat.inner(), pair(mid_one, mid_two), None);
                    summary.merge(&s);
                    let last = self.graph.node();
                    mid_two.connect(last).merge(Epsilon);
                    first.connect(sub.last).merge(Epsilon);
                    first = last;
                }
                first.connect(sub.last).merge(Epsilon);
                summary
            }
            (n, Some(m)) => {
                panic!("invalid repetition counters: {{{n},{m}}}");
            }
        }
    }

    fn translate_concat(&self, concat: &ConcatHir, sub: Pair<'a>, tag: Option<Tag>) -> Summary {
        let items = concat.items();
        if items.is_empty() {
            sub.first.connect(sub.last).merge(Epsilon);
            return Summary::empty();
        }
        let mut summary = Summary::empty();
        let mut tag = tag;
        let mut first = sub.first;
        for hir in &items[..items.len() - 1] {
            let last = self.graph.node();
            let sum = self.translate_hir(hir, pair(first, last), tag);
            summary.merge(&sum);
            if let (Some(inner), Some(len)) = (tag, hir.exact_len()) {
                tag = Some(Tag::secondary(
                    self.next_reg(),
                    inner.starting_tag(),
                    inner.offset() + len,
                ));
            } else {
                tag = None;
            }
            first = last;
        }
        let hir = items.last().unwrap();
        let sum = self.translate_hir(hir, pair(first, sub.last), tag);
        summary.merge(&sum);
        summary
    }

    /// ```txt
    ///  ╭───ε──→(○)──'a'─→(○)──ε───╮
    ///  │                          ↓
    /// (○)──ε──→(○)──'b'─→(○)──ε─→(○)
    ///  │                          ↑
    ///  ╰───ε──→(○)──'c'─→(○)──ε───╯
    /// ```
    fn translate_disjunct(&self, disjunct: &DisjunctHir, sub: Pair<'a>) -> Summary {
        let mut tr_outs = Vec::new();
        let mut summaries = Vec::new();
        for hir in disjunct.alternatives() {
            let first = self.graph.node();
            let last = self.graph.node();
            sub.first.connect(first).merge(Epsilon);
            let tr_out = last.connect(sub.last);
            tr_out.merge(Epsilon);
            tr_outs.push(tr_out);
            let sum = self.translate_hir(hir, pair(first, last), None);
            summaries.push(sum);
        }
        let mut summary = Summary::empty();
        for (tr, sum) in tr_outs.iter().zip(summaries.iter()) {
            summary.merge(sum);
            for (other_tr, sum) in tr_outs.iter().zip(summaries.iter()) {
                if tr != other_tr {
                    for reg in &sum.used_regs {
                        tr.merge_instruct(Inst::Invalidate(*reg));
                    }
                }
            }
        }
        summary
    }

    pub fn next_reg(&self) -> u32 {
        let new_reg = self.next_reg.get();
        self.next_reg
            .update(|id| id.checked_add(1).expect("register id overflow"));
        new_reg
    }
}

struct Summary {
    // tag: Option<Tag>,
    used_regs: redt::Set<u32>,
}

impl Summary {
    fn empty() -> Self {
        Self {
            // tag: None,
            used_regs: redt::Set::new(),
        }
    }

    fn merge(&mut self, other: &Summary) {
        self.used_regs.extend(other.used_regs.iter());
    }
}

#[cfg(test)]
#[path = "translator.utest.rs"]
mod utest;
