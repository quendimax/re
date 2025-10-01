use crate::arena::Arena;
use crate::isa::Inst;
use crate::node::Node;
use crate::symbol::{Epsilon, SymbolSet};
use bumpalo::collections::Vec as BumpVec;
use redt::ops::{ContainOp, IncludeOp, IntersectOp};
use redt::{ByteIter, Legible, RangeIter, RangeU8, SetU8, Step};
use std::cell::{Ref, RefCell};
use std::fmt::Write;
use std::ops::Deref;

/// Transition is a struct that contains symbols that connect two nodes. The
/// symbols can be bytes and Epsilon.
///
/// # Implementation
///
/// Symbols are the corresponding bits in `chunks` bitmap from 4x`Chunk` values.
/// The 256-th bit is for Epsilon.
pub struct Transition<'a>(&'a TransitionInner<'a>);

pub(crate) struct TransitionInner<'a> {
    symset: RefCell<SymbolSet>,
    insts: RefCell<BumpVec<'a, (Inst, &'a mut SymbolSet)>>,
    arena: &'a Arena,
    source: Option<Node<'a>>,
}

/// Crate API
impl<'a> Transition<'a> {
    /// Creates a new empty transition.
    pub(crate) fn new(source: Node<'a>, target: Node<'a>) -> Self {
        assert_eq!(
            source.gid(),
            target.gid(),
            "can't connect two nodes from different graphs"
        );
        let arena = source.arena();
        Self(arena.alloc_with(|| TransitionInner {
            symset: RefCell::new(SymbolSet::default()),
            insts: RefCell::new(BumpVec::new_in(&arena.shared_bump)),
            arena,
            source: Some(source),
        }))
    }

    /// Creates a new empty transition without source.
    ///
    /// This constructor is needed for DFA graph only. A transition without
    /// source node is used as mask of already used symbols in the transition by
    /// a DFA source node.
    pub(crate) fn without_source_in(arena: &'a Arena) -> Self {
        Self(arena.alloc_with(|| TransitionInner {
            symset: RefCell::new(SymbolSet::default()),
            insts: RefCell::new(BumpVec::new_in(&arena.shared_bump)),
            arena,
            source: None,
        }))
    }
}

impl<'a> Transition<'a> {
    /// Checks if these transitions are two references to the same transition.
    pub fn is(self, other: Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }

    /// Returns iterator over all symbols in this trasition instance in
    /// ascendent order.
    pub fn symbols(self) -> impl Iterator<Item = u8> {
        let borrow = self.0.symset.borrow();
        let borrow = Ref::map(borrow, |set| set.deref());
        ByteIter::new(borrow)
    }

    /// Returns iterator over all symbol ranges in this trasition instance in
    /// ascendent order.
    pub fn ranges(self) -> impl Iterator<Item = RangeU8> {
        let borrow = self.0.symset.borrow();
        let borrow = Ref::map(borrow, |set| set.deref());
        RangeIter::new(borrow)
    }

    pub fn instructs(self) -> impl Iterator<Item = Inst> {
        InstructIter::new(self.0.insts.borrow())
    }

    pub fn instructs_for(&self, symbol: u8) -> impl Iterator<Item = Inst> {
        InstructForIter::new(self.0.insts.borrow(), symbol)
    }

    /// Merges the `other` object into this transition.
    pub fn merge<T>(&self, other: T)
    where
        T: Copy,
        Self: TransitionOps<T>,
    {
        if let Some(source) = self.0.source {
            source.assert_dfa(other);
        }
        TransitionOps::merge(self, other);
    }

    /// Adds the specified instruction to the all transition's symbols.
    pub fn merge_instruct(&self, instruct: Inst) {
        let symset = self.0.symset.borrow().clone();
        let mut insts = self.0.insts.borrow_mut();
        match insts.binary_search_by(|probe| probe.0.cmp(&instruct)) {
            Ok(index) => *insts[index].1 = symset,
            Err(index) => {
                let new_bitmap = self.0.arena.alloc_with(|| symset);
                insts.insert(index, (instruct, new_bitmap));
            }
        }
    }

    /// Adds the specified operations to the all transition's symbols.
    pub fn merge_instructs(&self, instructs: impl IntoIterator<Item = Inst>) {
        for inst in instructs {
            self.merge_instruct(inst);
        }
    }

    pub fn intersects<T>(&self, other: T) -> bool
    where
        Self: IntersectOp<T>,
    {
        IntersectOp::intersects(self, other)
    }

    pub fn contains<T>(&self, other: T) -> bool
    where
        Self: ContainOp<T>,
    {
        ContainOp::contains(self, other)
    }
}

impl<T> ContainOp<T> for Transition<'_>
where
    SymbolSet: ContainOp<T>,
{
    fn contains(&self, rhs: T) -> bool {
        self.0.symset.borrow().contains(rhs)
    }
}

impl<'a, 'b> ContainOp<Transition<'b>> for Transition<'a> {
    #[inline]
    fn contains(&self, other: Transition<'b>) -> bool {
        self.0
            .symset
            .borrow()
            .contains(other.0.symset.borrow().deref())
    }
}

impl<T> IntersectOp<T> for Transition<'_>
where
    SymbolSet: IntersectOp<T>,
{
    fn intersects(&self, rhs: T) -> bool {
        self.0.symset.borrow().intersects(rhs)
    }
}

impl<'a, 'b> redt::ops::IntersectOp<Transition<'b>> for Transition<'a> {
    #[inline]
    fn intersects(&self, other: Transition<'b>) -> bool {
        self.0
            .symset
            .borrow()
            .intersects(other.0.symset.borrow().deref())
    }
}

pub trait TransitionOps<T>: redt::ops::ContainOp<T> + redt::ops::IntersectOp<T> {
    fn merge(&self, other: T);
}

macro_rules! impl_merge_op {
    ($($type:ty),* $(,)?) => {
        $(
            impl TransitionOps<$type> for Transition<'_> {
                fn merge(&self, other: $type) {
                    self.0.symset.borrow_mut().include(other);
                }
            }
        )*
    };
}

impl_merge_op!(u8, RangeU8, SetU8, &SetU8, Epsilon, &SymbolSet);

impl<'a, 'b> TransitionOps<Transition<'b>> for Transition<'a> {
    fn merge(&self, other: Transition<'b>) {
        let other_symset = other.0.symset.borrow();
        let other_symset = other_symset.deref();
        self.0.symset.borrow_mut().include(other_symset);

        let mut self_insts = self.0.insts.borrow_mut();
        for (other_insts, other_symset) in other.0.insts.borrow().iter() {
            if let Some((_, self_symset)) = self_insts
                .iter_mut()
                .find(|(self_inst, _)| self_inst == other_insts)
            {
                self_symset.include((*other_symset) as &SymbolSet);
            } else {
                let self_symset = self.0.arena.alloc_with(|| (*other_symset).clone());
                self_insts.push((*other_insts, self_symset));
            }
        }
        self_insts.sort_by(|(l_inst, _), (r_inst, _)| l_inst.cmp(r_inst));
    }
}

impl<'a, 'b> ContainOp<&Transition<'b>> for Transition<'a> {
    #[inline]
    fn contains(&self, other: &Transition<'b>) -> bool {
        ContainOp::contains(self, *other)
    }
}

impl<'a, 'b> IntersectOp<&Transition<'b>> for Transition<'a> {
    #[inline]
    fn intersects(&self, other: &Transition<'b>) -> bool {
        IntersectOp::intersects(self, *other)
    }
}

impl<'a, 'b> TransitionOps<&Transition<'b>> for Transition<'a> {
    fn merge(&self, other: &Transition<'b>) {
        TransitionOps::merge(self, *other);
    }
}

impl Copy for Transition<'_> {}

impl Clone for Transition<'_> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl std::cmp::Eq for Transition<'_> {}

impl std::cmp::PartialEq for Transition<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.symset.borrow().eq(other.0.symset.borrow().deref())
            && self.0.insts.borrow().eq(other.0.insts.borrow().deref())
    }
}

impl std::fmt::Display for Transition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        let mut iter = self.ranges();
        let mut range = iter.next();
        let mut has_symbols = false;
        while let Some(cur_range) = range {
            has_symbols = true;
            if let Some(next_range) = iter.next() {
                if cur_range.last().steps_between(next_range.start()) == 1 {
                    range = Some(RangeU8::new(cur_range.start(), next_range.last()));
                    continue;
                } else {
                    std::fmt::Display::fmt(&cur_range.display(), f)?;
                    f.write_str(" | ")?;
                    range = Some(next_range);
                }
            } else {
                std::fmt::Display::fmt(&cur_range.display(), f)?;
                break;
            }
        }
        if self.contains(Epsilon) {
            if has_symbols {
                f.write_str(" | ")?;
            }
            f.write_str("Epsilon")?;
        }
        f.write_char(']')
    }
}

macro_rules! impl_fmt {
    (std::fmt::$trait:ident) => {
        impl std::fmt::$trait for Transition<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_char('[')?;
                let mut first_iter = true;
                for range in self.ranges() {
                    if first_iter {
                        first_iter = false;
                    } else {
                        f.write_str(" | ")?;
                    }
                    ::std::fmt::$trait::fmt(&range, f)?;
                }

                if self.contains(Epsilon) {
                    if !first_iter {
                        f.write_str(" | ")?;
                    }
                    f.write_str("Epsilon")?;
                }
                f.write_char(']')
            }
        }
    };
}

impl_fmt!(std::fmt::Debug);
impl_fmt!(std::fmt::Binary);
impl_fmt!(std::fmt::Octal);
impl_fmt!(std::fmt::LowerHex);
impl_fmt!(std::fmt::UpperHex);

struct InstructIter<'a> {
    insts: Ref<'a, BumpVec<'a, (Inst, &'a mut SymbolSet)>>,
    index_iter: std::ops::Range<usize>,
}

impl<'a> InstructIter<'a> {
    fn new(insts: Ref<'a, BumpVec<'a, (Inst, &'a mut SymbolSet)>>) -> Self {
        let index_iter = 0..insts.len();
        Self { insts, index_iter }
    }
}

impl std::iter::Iterator for InstructIter<'_> {
    type Item = Inst;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.index_iter.next().map(|index| self.insts[index].0)
    }
}

struct InstructForIter<'a> {
    insts: Ref<'a, BumpVec<'a, (Inst, &'a mut SymbolSet)>>,
    index_iter: std::ops::Range<usize>,
    symbol: u8,
}

impl<'a> InstructForIter<'a> {
    fn new(insts: Ref<'a, BumpVec<'a, (Inst, &'a mut SymbolSet)>>, symbol: u8) -> Self {
        let index_iter = 0..insts.len();
        Self {
            insts,
            index_iter,
            symbol,
        }
    }
}

impl std::iter::Iterator for InstructForIter<'_> {
    type Item = Inst;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for index in &mut self.index_iter {
            let (_, symset) = &self.insts[index];
            if symset.contains(self.symbol) {
                return Some(self.insts[index].0);
            }
        }
        None
    }
}
#[cfg(test)]
mod utest {
    use crate::Graph;

    use super::*;

    #[test]
    #[should_panic]
    fn transition_new() {
        let mut arena_a = Arena::new();
        let mut arena_b = Arena::new();
        let gr_a = Graph::nfa_in(&mut arena_a);
        let gr_b = Graph::nfa_in(&mut arena_b);
        let _ = Transition::new(gr_a.node(), gr_b.node());
    }
}
