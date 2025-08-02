use crate::arena::Arena;
use crate::node::Node;
use crate::symbol::Epsilon;
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

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct TransitionInner<'a> {
    symset: RefCell<SymbolSet>,
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
            source: Some(source),
        }))
    }

    /// Creates a new empty transition without source.
    pub(crate) fn without_source_in(arena: &'a Arena) -> Self {
        Self(arena.alloc_with(|| TransitionInner {
            symset: RefCell::new(SymbolSet::default()),
            source: None,
        }))
    }
}

impl<'a> Transition<'a> {
    /// Returns iterator over all symbols in this trasition instance in
    /// ascendent order.
    pub fn symbols(self) -> SymbolIter<'a> {
        SymbolIter::new(self.0.symset.borrow())
    }

    /// Returns iterator over all symbol ranges in this trasition instance in
    /// ascendent order.
    pub fn ranges(self) -> RangeIter<'a> {
        RangeIter::new(self.0.symset.borrow())
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
impl_transition_ops!(transition: Transition<'_> []);
impl_transition_ops!(transition: &Transition<'_> [*]);

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

pub struct SymbolIter<'a> {
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

pub struct RangeIter<'a> {
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

    fn contains_transition(&self, other: Transition<'_>) -> bool {
        self.contains_symset(other.0.symset.borrow().deref())
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
        self.epsilon && other.epsilon
            || self.chunks[0] & other.chunks[0] != 0
            || self.chunks[1] & other.chunks[1] != 0
            || self.chunks[2] & other.chunks[2] != 0
            || self.chunks[3] & other.chunks[3] != 0
    }

    fn intersects_transition(&self, other: Transition<'_>) -> bool {
        self.intersects_symset(other.0.symset.borrow().deref())
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

    fn merge_transition(&mut self, other: Transition<'_>) {
        self.merge_symset(other.0.symset.borrow().deref());
    }
}
