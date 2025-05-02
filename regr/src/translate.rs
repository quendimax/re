use crate::error::Result;
use crate::error::err;
use crate::graph::Graph;
use crate::node::Node;
use crate::range::{Range, range};
use crate::symbol::Epsilon;
use crate::transition::Transition;
use regex_syntax::hir::{self, Hir, HirKind};
use utf8_ranges::{Utf8Sequence, Utf8Sequences};

pub struct Translator<'a> {
    graph: &'a Graph,
}

impl<'a> Translator<'a> {
    pub fn new(builder: &'a Graph) -> Translator<'a> {
        Self { graph: builder }
    }

    pub fn from_hir_to_nfa(&self, hir: &Hir) -> Result<(Node<'a>, Node<'a>)> {
        let start = self.graph.node();
        let finish = self.walk_hir(hir, start)?;
        Ok((start, finish))
    }

    fn walk_hir(&self, hir: &Hir, start: Node<'a>) -> Result<Node<'a>> {
        match hir.kind() {
            HirKind::Alternation(alternation) => self.walk_alternation(alternation, start),
            HirKind::Capture(capture) => self.walk_capture(capture, start),
            HirKind::Class(hir::Class::Bytes(class)) => self.walk_class_bytes(class, start),
            HirKind::Class(hir::Class::Unicode(class)) => self.walk_class_unicode(class, start),
            HirKind::Concat(subs) => self.walk_concat(subs, start),
            HirKind::Empty => self.walk_empty(start),
            HirKind::Literal(literal) => self.walk_literal(literal, start),
            HirKind::Look(_) => self.walk_look(),
            HirKind::Repetition(repetition) => self.walk_repetition(repetition, start),
        }
    }

    fn walk_alternation(&self, alters: &Vec<Hir>, start: Node<'a>) -> Result<Node<'a>> {
        let finish = self.graph.node();
        for alter in alters {
            let sub_start = self.graph.node();
            start.connect(sub_start, Epsilon);
            let sub_finish = self.walk_hir(alter, sub_start)?;
            sub_finish.connect(finish, Epsilon);
        }
        Ok(finish)
    }

    /// Captures are not supported for now, so it just treats it as a Hir instance.
    fn walk_capture(&self, capture: &hir::Capture, start: Node<'a>) -> Result<Node<'a>> {
        self.walk_hir(&capture.sub, start)
    }

    fn walk_class_bytes(&self, class: &hir::ClassBytes, start: Node<'a>) -> Result<Node<'a>> {
        let mut tr = Transition::default();
        for rg in class.ranges() {
            tr.merge(range(rg.start()..=rg.end()));
        }
        let finish = self.graph.node();
        start.connect(finish, &tr);
        Ok(finish)
    }

    fn walk_class_unicode(&self, class: &hir::ClassUnicode, start: Node<'a>) -> Result<Node<'a>> {
        let finish = self.graph.node();
        for rg in class.ranges() {
            for utf8_seq in Utf8Sequences::new(rg.start(), rg.end()) {
                match utf8_seq {
                    Utf8Sequence::One(range) => {
                        start.connect(finish, Range::from(range.start..=range.end));
                    }
                    Utf8Sequence::Two([r1, r2]) => {
                        let mid = self.graph.node();
                        start.connect(mid, Range::from(r1.start..=r1.end));
                        mid.connect(finish, Range::from(r2.start..=r2.end));
                    }
                    Utf8Sequence::Three([r1, r2, r3]) => {
                        let mid1 = self.graph.node();
                        let mid2 = self.graph.node();
                        start.connect(mid1, Range::from(r1.start..=r1.end));
                        mid1.connect(mid2, Range::from(r2.start..=r2.end));
                        mid2.connect(finish, Range::from(r3.start..=r3.end));
                    }
                    Utf8Sequence::Four([r1, r2, r3, r4]) => {
                        let mid1 = self.graph.node();
                        let mid2 = self.graph.node();
                        let mid3 = self.graph.node();
                        start.connect(mid1, Range::from(r1.start..=r1.end));
                        mid1.connect(mid2, Range::from(r2.start..=r2.end));
                        mid2.connect(mid3, Range::from(r3.start..=r3.end));
                        mid3.connect(finish, Range::from(r4.start..=r4.end));
                    }
                }
            }
        }
        Ok(finish)
    }

    fn walk_concat(&self, subs: &Vec<Hir>, start: Node<'a>) -> Result<Node<'a>> {
        let mut prev = start;
        for sub in subs {
            prev = self.walk_hir(sub, prev)?;
        }
        Ok(prev)
    }

    fn walk_empty(&self, start: Node<'a>) -> Result<Node<'a>> {
        let new_node = self.graph.node();
        start.connect(new_node, Epsilon);
        Ok(new_node)
    }

    fn walk_literal(&self, literal: &hir::Literal, start: Node<'a>) -> Result<Node<'a>> {
        let mut prev_node = start;
        for c in literal.0.iter() {
            let new_node = self.graph.node();
            prev_node.connect(new_node, range(*c));
            prev_node = new_node;
        }
        Ok(prev_node)
    }

    fn walk_look(&self) -> Result<Node<'a>> {
        err::unsupported_feature("look around is not supported")
    }

    fn walk_repetition(&self, repetition: &hir::Repetition, start: Node<'a>) -> Result<Node<'a>> {
        if !repetition.greedy {
            return err::unsupported_feature("non-greedy repetitions are not supported");
        }
        let mut last = start;
        for _ in 0..repetition.min {
            last = self.walk_hir(&repetition.sub, last)?;
        }
        let bypass_start = last;
        if let Some(max) = repetition.max {
            for _ in repetition.min..max {
                last = self.walk_hir(&repetition.sub, last)?;
                bypass_start.connect(last, Epsilon);
            }
        } else {
            let sub_start = self.graph.node();
            let sub_finish = self.walk_hir(&repetition.sub, sub_start)?;
            let bypass_finish = self.graph.node();
            bypass_start.connect(sub_start, Epsilon);
            sub_finish.connect(bypass_finish, Epsilon);
            sub_finish.connect(sub_start, Epsilon);
            bypass_start.connect(bypass_finish, Epsilon);
            last = bypass_finish;
        }
        Ok(last)
    }
}
