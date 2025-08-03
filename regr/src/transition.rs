use crate::arena::Arena;
use crate::instruct::Inst;
use crate::node::Node;
use crate::symbol::Epsilon;
use bumpalo::collections::Vec as BumpVec;
use redt::{Legible, RangeU8, Step};
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
    /// Returns iterator over all symbols in this trasition instance in
    /// ascendent order.
    pub fn symbols(self) -> impl Iterator<Item = u8> {
        SymbolIter::new(self.0.symset.borrow())
    }

    /// Returns iterator over all symbol ranges in this trasition instance in
    /// ascendent order.
    pub fn ranges(self) -> impl Iterator<Item = RangeU8> {
        RangeIter::new(self.0.symset.borrow())
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

    /// Adds the specified instruction to the all transition's symbols without
    /// sorting.
    ///
    /// This method is used internally for optimization purposes. It doesn't
    /// sort the instruction array at the end of the method.
    fn merge_inst_wo_sort(&self, instruct: Inst) -> bool {
        let symset = self.0.symset.borrow().clone();
        let mut insts = self.0.insts.borrow_mut();
        if insts.iter().any(|(inst, _)| *inst == instruct) {
            false
        } else {
            let new_bitmap = self.0.arena.alloc_with(|| symset);
            insts.push((instruct, new_bitmap));
            true
        }
    }

    /// Adds the specified instruction to the all transition's symbols.
    pub fn merge_instruct(&self, instruct: Inst) {
        if self.merge_inst_wo_sort(instruct) {
            let mut insts = self.0.insts.borrow_mut();
            insts.sort_by(|(l_inst, _), (r_inst, _)| l_inst.cmp(r_inst));
        }
    }

    /// Adds the specified operations to the all transition's symbols.
    pub fn merge_instructs(&self, instructs: impl IntoIterator<Item = Inst>) {
        let iter = instructs.into_iter();
        let mut merged = false;
        for inst in iter {
            merged |= self.merge_inst_wo_sort(inst);
        }
        if merged {
            let mut insts = self.0.insts.borrow_mut();
            insts.sort_by(|(l_inst, _), (r_inst, _)| l_inst.cmp(r_inst));
        }
    }

    pub fn intersects<T>(&self, other: T) -> bool
    where
        Self: TransitionOps<T>,
    {
        TransitionOps::intersects(self, other)
    }

    pub fn contains<T>(&self, other: T) -> bool
    where
        Self: TransitionOps<T>,
    {
        TransitionOps::contains(self, other)
    }
}

pub trait TransitionOps<T> {
    fn contains(&self, other: T) -> bool;
    fn intersects(&self, other: T) -> bool;
    fn merge(&self, other: T);
}

macro_rules! impl_transition_ops {
    ($other:ident: $other_ty:ty [ $($prefix:tt)? ]) => {
        paste::paste! {
            impl<'a> TransitionOps<$other_ty> for Transition<'a> {
                #[inline]
                fn contains(&self, $other: $other_ty) -> bool {
                    self.0.symset.borrow().[<contains_$other>]($($prefix)? $other)
                }

                #[inline]
                fn intersects(&self, $other: $other_ty) -> bool {
                    self.0.symset.borrow().[<intersects_$other>]($($prefix)? $other)
                }

                #[inline]
                fn merge(&self, $other: $other_ty) {
                    self.0.symset.borrow_mut().[<merge_$other>]($($prefix)? $other);
                }
            }
        }
    };
}

impl_transition_ops!(epsilon: Epsilon []);
impl_transition_ops!(epsilon: &Epsilon [*]);
impl_transition_ops!(symbol: u8 []);
impl_transition_ops!(symbol: &u8 [*]);
impl_transition_ops!(range: RangeU8 []);
impl_transition_ops!(range: &RangeU8 [*]);

impl<'a, 'b> TransitionOps<Transition<'b>> for Transition<'a> {
    #[inline]
    fn contains(&self, other: Transition<'b>) -> bool {
        self.0
            .symset
            .borrow()
            .contains_symset(other.0.symset.borrow().deref())
    }

    #[inline]
    fn intersects(&self, other: Transition<'b>) -> bool {
        self.0
            .symset
            .borrow()
            .intersects_symset(other.0.symset.borrow().deref())
    }

    fn merge(&self, other: Transition<'b>) {
        let other_symset = other.0.symset.borrow();
        let other_symset = other_symset.deref();
        self.0.symset.borrow_mut().merge_symset(other_symset);

        let mut self_insts = self.0.insts.borrow_mut();
        for (other_insts, other_symset) in other.0.insts.borrow().iter() {
            if let Some((_, self_symset)) = self_insts
                .iter_mut()
                .find(|(self_inst, _)| self_inst == other_insts)
            {
                self_symset.merge_symset(other_symset);
            } else {
                let self_symset = self.0.arena.alloc_with(|| (*other_symset).clone());
                self_insts.push((*other_insts, self_symset));
            }
        }
        self_insts.sort_by(|(l_inst, _), (r_inst, _)| l_inst.cmp(r_inst));
    }
}

impl<'a, 'b> TransitionOps<&Transition<'b>> for Transition<'a> {
    #[inline]
    fn contains(&self, other: &Transition<'b>) -> bool {
        TransitionOps::contains(self, *other)
    }

    #[inline]
    fn intersects(&self, other: &Transition<'b>) -> bool {
        TransitionOps::intersects(self, *other)
    }

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
        if self.contains(crate::symbol::Epsilon) {
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

                if self.contains(crate::symbol::Epsilon) {
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
            if symset.contains_symbol(self.symbol) {
                return Some(self.insts[index].0);
            }
        }
        None
    }
}

struct SymbolIter<'a> {
    symset: Ref<'a, SymbolSet>,
    chunk: Chunk,
    shift: u32,
}

impl<'a> SymbolIter<'a> {
    fn new(symset: Ref<'a, SymbolSet>) -> Self {
        let chunk = symset.chunks[0];
        Self {
            symset,
            chunk,
            shift: 0,
        }
    }
}

impl std::iter::Iterator for SymbolIter<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const SHIFT_OVERFLOW: u32 = (BITMAP_LEN << 6) as u32;
        while self.shift < SHIFT_OVERFLOW {
            if self.chunk != 0 {
                let trailing_zeros = self.chunk.trailing_zeros();
                self.chunk &= self.chunk.wrapping_sub(1);
                let symbol = trailing_zeros + self.shift;
                return Some(symbol as u8);
            }
            if self.shift < SHIFT_OVERFLOW - 64 {
                self.shift += 64;
                self.chunk = self.symset.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}

struct RangeIter<'a> {
    symset: Ref<'a, SymbolSet>,
    chunk: Chunk,
    shift: u32,
}

impl<'a> RangeIter<'a> {
    fn new(symset: Ref<'a, SymbolSet>) -> Self {
        let chunk = symset.chunks[0];
        Self {
            symset,
            chunk,
            shift: 0,
        }
    }
}

impl std::iter::Iterator for RangeIter<'_> {
    type Item = RangeU8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const SHIFT_OVERFLOW: u32 = (BITMAP_LEN << 6) as u32;
        while self.shift < SHIFT_OVERFLOW {
            if self.chunk != 0 {
                let trailing_zeros = self.chunk.trailing_zeros();
                self.chunk |= self.chunk.wrapping_sub(1);

                let trailing_ones = self.chunk.trailing_ones();
                self.chunk &= self.chunk.wrapping_add(1);

                let start = trailing_zeros + self.shift;
                let end = trailing_ones - 1 + self.shift;

                return Some(RangeU8::new_unchecked(start as u8, end as u8));
            }

            if self.shift < SHIFT_OVERFLOW - 64 {
                self.shift += 64;
                self.chunk = self.symset.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}

type Chunk = u64;

/// Quantity of `Chunk` values in the `chunks` member for symbols' bits.
const BITMAP_LEN: usize = (u8::MAX as usize + 1) / Chunk::BITS as usize;

/// A set of symbols that can be used to represent a language alphabet + Epsilon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SymbolSet {
    chunks: [Chunk; BITMAP_LEN],
    epsilon: bool,
}

impl SymbolSet {
    /// Creates a new empty symbol set.
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            chunks: [0; BITMAP_LEN],
            epsilon: false,
        }
    }
}

impl std::default::Default for SymbolSet {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolSet {
    fn contains_epsilon(&self, _: Epsilon) -> bool {
        self.epsilon
    }

    fn contains_symbol(&self, symbol: u8) -> bool {
        self.chunks[symbol as usize >> 6] & (1 << (symbol & (u8::MAX >> 2))) != 0
    }

    fn contains_range(&self, range: RangeU8) -> bool {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        unsafe {
            match ms_index - ls_index {
                0 => {
                    let mask = ls_mask & ms_mask;
                    *self.chunks.get_unchecked(ls_index) & mask == mask
                }
                1 => {
                    *self.chunks.get_unchecked(ls_index) & ls_mask == ls_mask
                        && *self.chunks.get_unchecked(ls_index + 1) & ms_mask == ms_mask
                }
                2 => {
                    *self.chunks.get_unchecked(ls_index) & ls_mask == ls_mask
                        && *self.chunks.get_unchecked(ls_index + 1) == Chunk::MAX
                        && *self.chunks.get_unchecked(ls_index + 2) & ms_mask == ms_mask
                }
                3 => {
                    *self.chunks.get_unchecked(0) & ls_mask == ls_mask
                        && *self.chunks.get_unchecked(1) == Chunk::MAX
                        && *self.chunks.get_unchecked(2) == Chunk::MAX
                        && *self.chunks.get_unchecked(3) & ms_mask == ms_mask
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }

    fn contains_symset(&self, other: &SymbolSet) -> bool {
        self.epsilon & other.epsilon == other.epsilon
            && self.chunks[0] & other.chunks[0] == other.chunks[0]
            && self.chunks[1] & other.chunks[1] == other.chunks[1]
            && self.chunks[2] & other.chunks[2] == other.chunks[2]
            && self.chunks[3] & other.chunks[3] == other.chunks[3]
    }

    fn intersects_epsilon(&self, _: Epsilon) -> bool {
        self.epsilon
    }

    fn intersects_symbol(&self, symbol: u8) -> bool {
        self.chunks[symbol as usize >> 6] & (1 << (symbol & (u8::MAX >> 2))) != 0
    }

    fn intersects_range(&self, range: RangeU8) -> bool {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        unsafe {
            match ms_index - ls_index {
                0 => {
                    let mask = ls_mask & ms_mask;
                    *self.chunks.get_unchecked(ls_index) & mask != 0
                }
                1 => {
                    *self.chunks.get_unchecked(ls_index) & ls_mask != 0
                        || *self.chunks.get_unchecked(ls_index + 1) & ms_mask != 0
                }
                2 => {
                    *self.chunks.get_unchecked(ls_index) & ls_mask != 0
                        || *self.chunks.get_unchecked(ls_index + 1) != 0
                        || *self.chunks.get_unchecked(ls_index + 2) & ms_mask != 0
                }
                3 => {
                    *self.chunks.get_unchecked(0) & ls_mask != 0
                        || *self.chunks.get_unchecked(1) != 0
                        || *self.chunks.get_unchecked(2) != 0
                        || *self.chunks.get_unchecked(3) & ms_mask != 0
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }

    fn intersects_symset(&self, other: &SymbolSet) -> bool {
        (self.epsilon && other.epsilon)
            || self.chunks[0] & other.chunks[0] != 0
            || self.chunks[1] & other.chunks[1] != 0
            || self.chunks[2] & other.chunks[2] != 0
            || self.chunks[3] & other.chunks[3] != 0
    }

    fn merge_epsilon(&mut self, _: Epsilon) {
        self.epsilon = true;
    }

    fn merge_symbol(&mut self, symbol: u8) {
        self.chunks[symbol as usize >> 6] |= 1 << (symbol & (u8::MAX >> 2));
    }

    fn merge_range(&mut self, range: RangeU8) {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        unsafe {
            match ms_index - ls_index {
                0 => {
                    *self.chunks.get_unchecked_mut(ls_index) |= ls_mask & ms_mask;
                }
                1 => {
                    *self.chunks.get_unchecked_mut(ls_index) |= ls_mask;
                    *self.chunks.get_unchecked_mut(ls_index + 1) |= ms_mask;
                }
                2 => {
                    *self.chunks.get_unchecked_mut(ls_index) |= ls_mask;
                    *self.chunks.get_unchecked_mut(ls_index + 1) |= Chunk::MAX;
                    *self.chunks.get_unchecked_mut(ls_index + 2) |= ms_mask;
                }
                3 => {
                    *self.chunks.get_unchecked_mut(0) |= ls_mask;
                    *self.chunks.get_unchecked_mut(1) |= Chunk::MAX;
                    *self.chunks.get_unchecked_mut(2) |= Chunk::MAX;
                    *self.chunks.get_unchecked_mut(3) |= ms_mask;
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }

    fn merge_symset(&mut self, other: &SymbolSet) {
        self.chunks[0] |= other.chunks[0];
        self.chunks[1] |= other.chunks[1];
        self.chunks[2] |= other.chunks[2];
        self.chunks[3] |= other.chunks[3];
        self.epsilon |= other.epsilon;
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
