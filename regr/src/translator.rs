use crate::graph::Graph;
use crate::isa::Inst;
use crate::node::Node;
use crate::tag::{Tag, TagBank};
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
    tag_bank: TagBank,
}

impl<'a, 'g> Translator<'a, 'g> {
    pub fn new(graph: &'g Graph<'a>) -> Self {
        // TODO: add optional checker for DFA graph
        Self {
            graph,
            next_reg: Cell::new(0),
            tag_bank: TagBank::new(),
        }
    }

    pub fn translate(&mut self, hir: &Hir, start_hode: Node<'a>, end_node: Node<'a>) {
        let mut tag = None;
        _ = self.translate_hir(hir, pair(start_hode, end_node), &mut tag);
    }

    fn translate_hir(&mut self, hir: &Hir, sub: Pair<'a>, tag: &mut Option<Tag>) -> Summary {
        match hir {
            Hir::Literal(literal) => self.translate_literal(literal, sub, tag),
            Hir::Class(class) => self.translate_class(class, sub, tag),
            Hir::Group(group) => self.translate_group(group, sub, tag),
            Hir::Repeat(repeat) => self.translate_repeat(repeat, sub, tag),
            Hir::Concat(concat) => self.translate_concat(concat, sub, tag),
            Hir::Disjunct(disjunct) => self.translate_disjunct(disjunct, sub, tag),
        }
    }

    fn translate_literal(&self, literal: &[u8], sub: Pair<'a>, tag: &mut Option<Tag>) -> Summary {
        if literal.is_empty() {
            sub.first.connect(sub.last);
            return Summary::empty();
        }
        let mut first = sub.first;
        for byte in &literal[..literal.len() - 1] {
            let next = self.graph.node();
            first.connect(next).merge(*byte);
            first = next;
        }
        let last_byte = literal.last().unwrap();
        first.connect(sub.last).merge(*last_byte);
        if let Some(tag) = tag {
            tag.add_offset(literal.len());
        }
        Summary::empty()
    }

    fn translate_class(&self, class: &SetU8, sub: Pair<'a>, tag: &mut Option<Tag>) -> Summary {
        for range in class.ranges() {
            sub.first.connect(sub.last).merge(range);
        }
        if let Some(tag) = tag {
            tag.add_offset(1);
        }
        Summary::empty()
    }

    // Only this function can create a new tag
    fn translate_group(
        &mut self,
        group: &GroupHir,
        sub: Pair<'a>,
        tag: &mut Option<Tag>,
    ) -> Summary {
        let first = self.graph.node();
        let tr_in = sub.first.connect(first);

        let last = self.graph.node();
        let tr_out = last.connect(sub.last);

        let mut summary = Summary::empty();

        let open_tag = if let Some((open_tag, _)) = self.graph.tag_group(group.label()) {
            open_tag
        } else {
            tag.unwrap_or_else(|| self.tag_bank.absolute())
        };

        match open_tag {
            Tag::Absolute { id, reg } => {
                tr_in.merge_instruct(Inst::WritePos(id, reg), None);
                summary.absolute_tags.insert(id);
            }
            Tag::PseudoAbsolute { id, .. } => {
                summary.pseudo_absolute_tags.insert(id);
            }
            Tag::Relative { .. } => (),
        }

        let mut inner_tag = Some(self.tag_bank.relative(open_tag, 0));
        let sum = self.translate_hir(group.inner(), pair(first, last), &mut inner_tag);
        summary.merge(&sum);

        let close_tag = if let Some((_, close_tag)) = self.graph.tag_group(group.label()) {
            close_tag
        } else {
            inner_tag.unwrap_or_else(|| self.tag_bank.absolute())
        };

        match close_tag {
            Tag::Absolute { id, reg } => {
                tr_out.merge_instruct(Inst::WritePos(id, reg), None);
                summary.absolute_tags.insert(id);
            }
            Tag::PseudoAbsolute { id, .. } => {
                summary.pseudo_absolute_tags.insert(id);
            }
            Tag::Relative { .. } => (),
        }

        *tag = Some(self.tag_bank.relative(close_tag, 0));
        self.graph.add_tag_group(group.label(), open_tag, close_tag);

        summary
    }

    fn translate_repeat(
        &mut self,
        repeat: &RepeatHir,
        mut sub: Pair<'a>,
        tag: &mut Option<Tag>,
    ) -> Summary {
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
                sub.first.connect(first);
                last.connect(sub.last);
                last.connect(first);
                sub.first.connect(sub.last);
                *tag = None;
                self.translate_hir(repeat.inner(), pair(first, last), tag)
            }
            //
            //          ╭────ε────╮
            //          ↓         │
            // (1)──ε─→(2)──'a'─→(3)──ε─→(4)
            //
            (1, None) => {
                let first = self.graph.node();
                let last = self.graph.node();
                sub.first.connect(first);
                last.connect(sub.last);
                last.connect(first);
                *tag = None;
                self.translate_hir(repeat.inner(), pair(first, last), tag)
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
                    *tag = None;
                    let s = self.translate_hir(repeat.inner(), pair(first, last), tag);
                    summary.merge(&s);
                    first = last;
                }
                sub.first = first;
                let first = self.graph.node();
                let last = self.graph.node();
                *tag = None;
                let s = self.translate_hir(repeat.inner(), pair(first, last), tag);
                summary.merge(&s);
                sub.first.connect(first);
                last.connect(sub.last);
                last.connect(first);
                summary
            }
            //
            // (0)──'a'──(1)──'a'──...──'a'─→(n)
            //
            (n, Some(m)) if n == m => {
                let mut summary = Summary::empty();
                if n == 0 {
                    sub.first.connect(sub.last);
                } else {
                    let mut first = sub.first;
                    for _ in 0..n - 1 {
                        let last = self.graph.node();
                        *tag = None;
                        let s = self.translate_hir(repeat.inner(), pair(first, last), tag);
                        summary.merge(&s);
                        first = last;
                    }
                    *tag = None;
                    let s = self.translate_hir(repeat.inner(), pair(first, sub.last), tag);
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
                    *tag = None;
                    let s = self.translate_hir(repeat.inner(), pair(first, last), tag);
                    summary.merge(&s);
                    first = last;
                }
                for _ in n..m {
                    let mid_one = self.graph.node();
                    first.connect(mid_one);
                    let mid_two = self.graph.node();
                    *tag = None;
                    let s = self.translate_hir(repeat.inner(), pair(mid_one, mid_two), tag);
                    summary.merge(&s);
                    let last = self.graph.node();
                    mid_two.connect(last);
                    first.connect(sub.last);
                    first = last;
                }
                first.connect(sub.last);
                summary
            }
            (n, Some(m)) => {
                panic!("invalid repetition counters: {{{n},{m}}}");
            }
        }
    }

    fn translate_concat(
        &mut self,
        concat: &ConcatHir,
        sub: Pair<'a>,
        tag: &mut Option<Tag>,
    ) -> Summary {
        let items = concat.items();
        if items.is_empty() {
            sub.first.connect(sub.last);
            return Summary::empty();
        }
        let mut summary = Summary::empty();
        let mut first = sub.first;
        for hir in &items[..items.len() - 1] {
            let last = self.graph.node();
            let sum = self.translate_hir(hir, pair(first, last), tag);
            summary.merge(&sum);
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
    fn translate_disjunct(
        &mut self,
        disjunct: &DisjunctHir,
        sub: Pair<'a>,
        tag: &mut Option<Tag>,
    ) -> Summary {
        let mut tr_outs = Vec::new();
        let mut summaries = Vec::new();
        for hir in disjunct.alternatives() {
            let first = self.graph.node();
            let last = self.graph.node();
            sub.first.connect(first);
            let tr_out = last.connect(sub.last);
            tr_outs.push(tr_out);
            let mut tag = tag.map(|t| self.tag_bank.pseudo_absolute(t));
            let sum = self.translate_hir(hir, pair(first, last), &mut tag);
            summaries.push(sum);
        }
        let mut summary = Summary::empty();
        for (tr, sum) in tr_outs.iter().zip(summaries.iter()) {
            summary.merge(sum);
            for (other_tr, sum) in tr_outs.iter().zip(summaries.iter()) {
                if !tr.is(*other_tr) {
                    for tag in &sum.absolute_tags {
                        tr.merge_instruct(Inst::InvalidateTag(*tag), None);
                    }
                    for tag in &sum.pseudo_absolute_tags {
                        tr.merge_instruct(Inst::InvalidateTag(*tag), None);
                    }
                }
            }
        }
        if let Some(len) = disjunct.exact_len()
            && let Some(tag) = tag
        {
            tag.add_offset(len);
        } else {
            *tag = None;
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

#[derive(Debug)]
struct Summary {
    absolute_tags: redt::Set<u32>,
    pseudo_absolute_tags: redt::Set<u32>,
}

impl Summary {
    fn empty() -> Self {
        Self {
            absolute_tags: redt::Set::new(),
            pseudo_absolute_tags: redt::Set::new(),
        }
    }

    fn merge(&mut self, other: &Summary) {
        self.absolute_tags.extend(other.absolute_tags.iter());
        self.pseudo_absolute_tags
            .extend(other.pseudo_absolute_tags.iter());
    }
}

#[cfg(test)]
#[path = "translator.utest.rs"]
mod utest;
