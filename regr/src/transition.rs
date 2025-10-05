use crate::arena::Arena;
use crate::isa::Inst;
use crate::node::Node;
use crate::ops::*;
use crate::symbol::Epsilon;
use bumpalo::collections::Vec as BumpVec;
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
    symset: RefCell<SetU8>,
    insts: RefCell<BumpVec<'a, (Inst, &'a mut SetU8)>>,
    arena: &'a Arena,
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
            symset: RefCell::new(SetU8::empty()),
            insts: RefCell::new(BumpVec::new_in(&arena.shared_bump)),
            arena,
        }))
    }
}

impl<'a> Transition<'a> {
    /// Checks if these transitions are two references to the same transition.
    #[inline]
    pub fn is(self, other: Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }

    #[inline]
    pub fn is_epsilon(&self) -> bool {
        self.0.symset.borrow().is_empty()
    }

    /// Returns iterator over all symbols in this trasition instance in
    /// ascendent order.
    pub fn symbols(self) -> impl Iterator<Item = u8> {
        let borrow = self.0.symset.borrow();
        ByteIter::new(borrow)
    }

    /// Returns iterator over all symbol ranges in this trasition instance in
    /// ascendent order.
    pub fn ranges(self) -> impl Iterator<Item = RangeU8> {
        let borrow = self.0.symset.borrow();
        RangeIter::new(borrow)
    }

    /// Returns a clone of the symbol set in this transition instance.
    pub fn as_set(&self) -> Ref<'_, SetU8> {
        self.0.symset.borrow()
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
        Self: Mergeable<T>,
    {
        Mergeable::merge(self, other);
    }

    /// Adds an instruction to specific symbols in this transition. If the
    /// specified symbols are not present in this transition, they are ignored.
    /// For `None`, the instruction is added to all symbols.
    pub fn merge_instruct(&self, instruct: Inst, for_symbols: Option<SetU8>) {
        let mut symset = self.0.symset.borrow().clone();
        if let Some(for_symbols) = for_symbols {
            symset &= for_symbols;
        }
        let mut insts = self.0.insts.borrow_mut();
        match insts.binary_search_by(|probe| probe.0.cmp(&instruct)) {
            Ok(index) => *insts[index].1 |= symset,
            Err(index) => {
                let new_bitmap = self.0.arena.alloc_with(|| symset);
                insts.insert(index, (instruct, new_bitmap));
            }
        }
    }

    /// Adds the specified operations to the all transition's symbols.
    pub fn merge_instructs(
        &self,
        instructs: impl IntoIterator<Item = Inst>,
        for_symbols: Option<SetU8>,
    ) {
        for inst in instructs {
            self.merge_instruct(inst, for_symbols.clone());
        }
    }

    pub fn intersects<T>(&self, other: T) -> bool
    where
        Self: Intersectable<T>,
    {
        Intersectable::intersects(self, other)
    }

    pub fn contains<T>(&self, other: T) -> bool
    where
        Self: Containable<T>,
    {
        Containable::contains(self, other)
    }
}

impl<T> Containable<T> for Transition<'_>
where
    SetU8: Containable<T>,
{
    fn contains(&self, rhs: T) -> bool {
        self.0.symset.borrow().contains(rhs)
    }
}

impl Containable<Epsilon> for Transition<'_> {
    fn contains(&self, _: Epsilon) -> bool {
        self.0.symset.borrow().is_empty()
    }
}

impl<'a, 'b> Containable<Transition<'b>> for Transition<'a> {
    #[inline]
    fn contains(&self, other: Transition<'b>) -> bool {
        self.0
            .symset
            .borrow()
            .contains(other.0.symset.borrow().deref())
    }
}

impl<'a, 'b> Containable<&Transition<'b>> for Transition<'a> {
    #[inline]
    fn contains(&self, other: &Transition<'b>) -> bool {
        Containable::contains(self, *other)
    }
}

impl<T> Intersectable<T> for Transition<'_>
where
    SetU8: Intersectable<T>,
{
    fn intersects(&self, rhs: T) -> bool {
        self.0.symset.borrow().intersects(rhs)
    }
}

impl Intersectable<Epsilon> for Transition<'_> {
    fn intersects(&self, _: Epsilon) -> bool {
        self.0.symset.borrow().is_empty()
    }
}

impl<'a, 'b> redt::ops::Intersectable<Transition<'b>> for Transition<'a> {
    #[inline]
    fn intersects(&self, other: Transition<'b>) -> bool {
        self.0
            .symset
            .borrow()
            .intersects(other.0.symset.borrow().deref())
    }
}

impl<'a, 'b> Intersectable<&Transition<'b>> for Transition<'a> {
    #[inline]
    fn intersects(&self, other: &Transition<'b>) -> bool {
        Intersectable::intersects(self, *other)
    }
}

impl<T> Mergeable<T> for Transition<'_>
where
    SetU8: Includable<T>,
{
    fn merge(&self, rhs: T) -> &Self {
        self.0.symset.borrow_mut().include(rhs);
        self
    }
}

impl<'a, 'b> Mergeable<Transition<'b>> for Transition<'a> {
    fn merge(&self, other: Transition<'b>) -> &Self {
        let other_symset = other.0.symset.borrow();
        let other_symset = other_symset.deref();
        self.0.symset.borrow_mut().include(other_symset);
        for (other_inst, other_symset) in other.0.insts.borrow().iter() {
            self.merge_instruct(*other_inst, Some((*other_symset).clone()));
        }
        self
    }
}

impl<'a, 'b> Mergeable<&Transition<'b>> for Transition<'a> {
    fn merge(&self, other: &Transition<'b>) -> &Self {
        Mergeable::merge(self, *other);
        self
    }
}

impl<T> Rejectable<T> for Transition<'_>
where
    T: Clone,
    SetU8: Excludable<T>,
{
    fn reject(&self, rhs: T) -> &Self {
        for (_, symset) in self.0.insts.borrow_mut().iter_mut() {
            symset.exclude(rhs.clone());
        }
        self.0.symset.borrow_mut().exclude(rhs);
        self
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
    insts: Ref<'a, BumpVec<'a, (Inst, &'a mut SetU8)>>,
    index_iter: std::ops::Range<usize>,
}

impl<'a> InstructIter<'a> {
    fn new(insts: Ref<'a, BumpVec<'a, (Inst, &'a mut SetU8)>>) -> Self {
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
    insts: Ref<'a, BumpVec<'a, (Inst, &'a mut SetU8)>>,
    index_iter: std::ops::Range<usize>,
    symbol: u8,
}

impl<'a> InstructForIter<'a> {
    fn new(insts: Ref<'a, BumpVec<'a, (Inst, &'a mut SetU8)>>, symbol: u8) -> Self {
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
        let gr_a = Graph::new_in(&mut arena_a);
        let gr_b = Graph::new_in(&mut arena_b);
        let _ = Transition::new(gr_a.node(), gr_b.node());
    }
}
