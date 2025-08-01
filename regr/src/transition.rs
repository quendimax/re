use crate::arena::Arena;
use crate::node::Node;
use crate::symbol::Epsilon;
use redt::{Legible, RangeU8, Step};
use std::cell::{Ref, RefCell};
use std::fmt::Write;

type Chunk = u64;

/// Quantity of `Chunk` values in the `chunks` member for symbols' bits.
const SYM_BITMAP_LEN: usize = 4;

/// Entire quantity of `Chunk` values in the `chunks` member.
const BITMAP_LEN: usize = SYM_BITMAP_LEN + 1; // + 1 for Epsilon bit

/// Index of `Chunk`-item that contains flags (epsilon, connected).
const FLAGS_CHUNK: usize = 4;

/// Mask for Epsilon bit in the flags chunk.
const EPSILON_FLAG: Chunk = 0x01;

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
    chunks: RefCell<[Chunk; BITMAP_LEN]>,
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
            chunks: RefCell::new([0; BITMAP_LEN]),
            source: Some(source),
        }))
    }

    /// Creates a new empty transition without source.
    pub(crate) fn without_source_in(arena: &'a Arena) -> Self {
        Self(arena.alloc_with(|| TransitionInner {
            chunks: RefCell::new([0; BITMAP_LEN]),
            source: None,
        }))
    }
}

impl<'a> Transition<'a> {
    /// Returns iterator over all symbols in this trasition instance in
    /// ascendent order.
    pub fn symbols(self) -> SymbolIter<'a> {
        SymbolIter::new(self.0.chunks.borrow())
    }

    /// Returns iterator over all symbol ranges in this trasition instance in
    /// ascendent order.
    pub fn ranges(self) -> RangeIter<'a> {
        RangeIter::new(self.0.chunks.borrow())
    }

    /// Merges the `other` object into this transition.
    pub fn merge<T>(&self, other: T)
    where
        T: Copy,
        Self: MergeOp<T> + IntersectOp<T>,
    {
        if let Some(source) = self.0.source {
            source.assert_dfa(other);
        }
        MergeOp::merge(self, other);
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

pub trait ContainOp<T> {
    fn contains(&self, other: T) -> bool;
}

impl ContainOp<&u8> for Transition<'_> {
    #[inline]
    fn contains(&self, symbol: &u8) -> bool {
        let chunks = self.0.chunks.borrow();
        chunks[*symbol as usize >> 6] & (1 << (symbol & (u8::MAX >> 2))) != 0
    }
}

impl ContainOp<u8> for Transition<'_> {
    #[inline]
    fn contains(&self, symbol: u8) -> bool {
        ContainOp::contains(self, &symbol)
    }
}

impl ContainOp<RangeU8> for Transition<'_> {
    #[inline]
    fn contains(&self, range: RangeU8) -> bool {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        let chunks = self.0.chunks.borrow();
        unsafe {
            match ms_index - ls_index {
                0 => {
                    let mask = ls_mask & ms_mask;
                    *chunks.get_unchecked(ls_index) & mask == mask
                }
                1 => {
                    *chunks.get_unchecked(ls_index) & ls_mask == ls_mask
                        && *chunks.get_unchecked(ls_index + 1) & ms_mask == ms_mask
                }
                2 => {
                    *chunks.get_unchecked(ls_index) & ls_mask == ls_mask
                        && *chunks.get_unchecked(ls_index + 1) == Chunk::MAX
                        && *chunks.get_unchecked(ls_index + 2) & ms_mask == ms_mask
                }
                3 => {
                    *chunks.get_unchecked(0) & ls_mask == ls_mask
                        && *chunks.get_unchecked(1) == Chunk::MAX
                        && *chunks.get_unchecked(2) == Chunk::MAX
                        && *chunks.get_unchecked(3) & ms_mask == ms_mask
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }
}

impl ContainOp<&RangeU8> for Transition<'_> {
    #[inline]
    fn contains(&self, range: &RangeU8) -> bool {
        Self::contains(self, *range)
    }
}

impl<'a> ContainOp<Transition<'a>> for Transition<'a> {
    fn contains(&self, other: Transition<'a>) -> bool {
        self.contains(&other)
    }
}

impl<'a> ContainOp<&Transition<'a>> for Transition<'a> {
    fn contains(&self, other: &Transition<'a>) -> bool {
        let self_chunks = self.0.chunks.borrow();
        let other_chunks = other.0.chunks.borrow();
        self_chunks[0] & other_chunks[0] == other_chunks[0]
            && self_chunks[1] & other_chunks[1] == other_chunks[1]
            && self_chunks[2] & other_chunks[2] == other_chunks[2]
            && self_chunks[3] & other_chunks[3] == other_chunks[3]
            && self_chunks[4] & other_chunks[4] == other_chunks[4]
    }
}

impl ContainOp<Epsilon> for Transition<'_> {
    fn contains(&self, _: Epsilon) -> bool {
        self.0.chunks.borrow()[FLAGS_CHUNK] & EPSILON_FLAG != 0
    }
}

pub trait IntersectOp<T> {
    fn intersects(&self, other: T) -> bool;
}

impl IntersectOp<&u8> for Transition<'_> {
    #[inline]
    fn intersects(&self, symbol: &u8) -> bool {
        let chunks = self.0.chunks.borrow();
        chunks[*symbol as usize >> 6] & (1 << (symbol & (u8::MAX >> 2))) != 0
    }
}

impl IntersectOp<u8> for Transition<'_> {
    #[inline]
    fn intersects(&self, symbol: u8) -> bool {
        IntersectOp::intersects(self, &symbol)
    }
}

impl IntersectOp<RangeU8> for Transition<'_> {
    #[inline]
    fn intersects(&self, range: RangeU8) -> bool {
        Self::intersects(self, &range)
    }
}

impl IntersectOp<&RangeU8> for Transition<'_> {
    fn intersects(&self, range: &RangeU8) -> bool {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        let chunks = self.0.chunks.borrow();
        unsafe {
            match ms_index - ls_index {
                0 => {
                    let mask = ls_mask & ms_mask;
                    *chunks.get_unchecked(ls_index) & mask != 0
                }
                1 => {
                    *chunks.get_unchecked(ls_index) & ls_mask != 0
                        || *chunks.get_unchecked(ls_index + 1) & ms_mask != 0
                }
                2 => {
                    *chunks.get_unchecked(ls_index) & ls_mask != 0
                        || *chunks.get_unchecked(ls_index + 1) != 0
                        || *chunks.get_unchecked(ls_index + 2) & ms_mask != 0
                }
                3 => {
                    *chunks.get_unchecked(0) & ls_mask != 0
                        || *chunks.get_unchecked(1) != 0
                        || *chunks.get_unchecked(2) != 0
                        || *chunks.get_unchecked(3) & ms_mask != 0
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }
}

impl<'a> IntersectOp<&Transition<'a>> for Transition<'a> {
    fn intersects(&self, other: &Transition<'a>) -> bool {
        let self_chunks = self.0.chunks.borrow();
        let other_chunks = other.0.chunks.borrow();
        self_chunks[0] & other_chunks[0] != 0
            || self_chunks[1] & other_chunks[1] != 0
            || self_chunks[2] & other_chunks[2] != 0
            || self_chunks[3] & other_chunks[3] != 0
            || self_chunks[4] & other_chunks[4] != 0
    }
}

impl<'a> IntersectOp<Transition<'a>> for Transition<'a> {
    #[inline]
    fn intersects(&self, other: Transition<'a>) -> bool {
        IntersectOp::intersects(self, &other)
    }
}

impl<'a> IntersectOp<Epsilon> for Transition<'a> {
    #[inline]
    fn intersects(&self, _: Epsilon) -> bool {
        self.0.chunks.borrow()[FLAGS_CHUNK] & EPSILON_FLAG != 0
    }
}

pub trait MergeOp<T> {
    fn merge(&self, other: T);
}

impl MergeOp<&u8> for Transition<'_> {
    /// Merges a symbol into this transition.
    #[inline]
    fn merge(&self, symbol: &u8) {
        let mut chunks = self.0.chunks.borrow_mut();
        chunks[*symbol as usize >> 6] |= 1 << (symbol & (u8::MAX >> 2));
    }
}

impl MergeOp<u8> for Transition<'_> {
    /// Merges a symbol into this transition.
    #[inline]
    fn merge(&self, symbol: u8) {
        MergeOp::merge(self, &symbol);
    }
}

impl MergeOp<&RangeU8> for Transition<'_> {
    fn merge(&self, range: &RangeU8) {
        let mut ls_mask = 1 << (range.start() & (u8::MAX >> 2));
        ls_mask = !(ls_mask - 1);

        let mut ms_mask = 1 << (range.last() & (u8::MAX >> 2));
        ms_mask |= ms_mask - 1;

        let ls_index = (range.start() >> 6) as usize;
        let ms_index = (range.last() >> 6) as usize;

        let mut chunks = self.0.chunks.borrow_mut();
        unsafe {
            match ms_index - ls_index {
                0 => {
                    *chunks.get_unchecked_mut(ls_index) |= ls_mask & ms_mask;
                }
                1 => {
                    *chunks.get_unchecked_mut(ls_index) |= ls_mask;
                    *chunks.get_unchecked_mut(ls_index + 1) |= ms_mask;
                }
                2 => {
                    *chunks.get_unchecked_mut(ls_index) |= ls_mask;
                    *chunks.get_unchecked_mut(ls_index + 1) |= Chunk::MAX;
                    *chunks.get_unchecked_mut(ls_index + 2) |= ms_mask;
                }
                3 => {
                    *chunks.get_unchecked_mut(0) |= ls_mask;
                    *chunks.get_unchecked_mut(1) |= Chunk::MAX;
                    *chunks.get_unchecked_mut(2) |= Chunk::MAX;
                    *chunks.get_unchecked_mut(3) |= ms_mask;
                }
                _ => std::hint::unreachable_unchecked(),
            }
        }
    }
}

impl MergeOp<RangeU8> for Transition<'_> {
    #[inline]
    fn merge(&self, range: RangeU8) {
        MergeOp::merge(self, &range);
    }
}

impl<'a> MergeOp<&Transition<'a>> for Transition<'a> {
    #[inline]
    fn merge(&self, other: &Transition<'a>) {
        let mut self_chunks = self.0.chunks.borrow_mut();
        let other_chunks = other.0.chunks.borrow();
        self_chunks[0] |= other_chunks[0];
        self_chunks[1] |= other_chunks[1];
        self_chunks[2] |= other_chunks[2];
        self_chunks[3] |= other_chunks[3];
        self_chunks[4] |= other_chunks[4];
    }
}

impl<'a> MergeOp<Transition<'a>> for Transition<'a> {
    #[inline]
    fn merge(&self, other: Transition<'a>) {
        MergeOp::merge(self, &other);
    }
}

impl MergeOp<Epsilon> for Transition<'_> {
    #[inline]
    fn merge(&self, _: Epsilon) {
        self.0.chunks.borrow_mut()[FLAGS_CHUNK] |= 1;
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
        self.0
            .chunks
            .borrow()
            .as_ref()
            .eq(other.0.chunks.borrow().as_ref())
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
    chunks: Ref<'a, [Chunk; BITMAP_LEN]>,
    chunk: Chunk,
    shift: u32,
}

impl<'a> SymbolIter<'a> {
    fn new(chunks: Ref<'a, [Chunk; BITMAP_LEN]>) -> Self {
        let chunk = chunks[0];
        Self {
            chunks,
            chunk,
            shift: 0,
        }
    }
}

impl std::iter::Iterator for SymbolIter<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const SHIFT_OVERFLOW: u32 = (SYM_BITMAP_LEN << 6) as u32;
        while self.shift < SHIFT_OVERFLOW {
            if self.chunk != 0 {
                let trailing_zeros = self.chunk.trailing_zeros();
                self.chunk &= self.chunk.wrapping_sub(1);
                let symbol = trailing_zeros + self.shift;
                return Some(symbol as u8);
            }
            if self.shift < SHIFT_OVERFLOW - 64 {
                self.shift += 64;
                self.chunk = self.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}

pub struct RangeIter<'a> {
    chunks: Ref<'a, [Chunk; BITMAP_LEN]>,
    chunk: Chunk,
    shift: u32,
}

impl<'a> RangeIter<'a> {
    fn new(chunks: Ref<'a, [Chunk; BITMAP_LEN]>) -> Self {
        let chunk = chunks[0];
        Self {
            chunks,
            chunk,
            shift: 0,
        }
    }
}

impl std::iter::Iterator for RangeIter<'_> {
    type Item = RangeU8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const SHIFT_OVERFLOW: u32 = (SYM_BITMAP_LEN << 6) as u32;
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
                self.chunk = self.chunks[self.shift as usize >> 6];
                continue;
            }
            break;
        }
        None
    }
}
