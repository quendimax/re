use bumpalo::Bump;
use smallvec::SmallVec;
use std::cell::Cell;

pub(crate) struct Arena<T> {
    bump: Bump,
    len: Cell<usize>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Sized> Arena<T> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity * std::mem::size_of::<T>()),
            len: Cell::new(0),
            _phantom: Default::default(),
        }
    }

    pub(crate) fn alloc_with<F>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        let refer = self.bump.alloc_with(f);
        self.len.replace(self.len.get() + 1);
        refer
    }
}

impl<T> std::ops::Drop for Arena<T> {
    fn drop(&mut self) {
        // number of allocated nodes
        let mut len = self.len.get();
        if len == 0 {
            return;
        }

        let alloc_size = std::alloc::Layout::new::<T>().size();

        for (chunk, size) in unsafe { self.bump.iter_allocated_chunks_raw() } {
            let mut ptr = chunk;
            let mut dropped_size = alloc_size;

            loop {
                unsafe { std::ptr::drop_in_place::<T>(ptr as *mut T) };

                len -= 1;
                if len == 0 {
                    return;
                }

                dropped_size += alloc_size;
                if dropped_size > size {
                    break;
                }

                ptr = unsafe { ptr.add(alloc_size) };
            }
        }
    }
}

impl<T> Arena<T> {
    /// Returns an iterator over the items in the arena.
    ///
    /// Because of the arena is a bump allocator, it is possible to add new
    /// items to the arena during iteration. But the iteration won't be affected
    /// by the new items, i.e. if at the moment of creation the iterator the
    /// arena contains `n` items, then the iterator will iterate over `n` items.
    pub(crate) fn iter(&self) -> Iter<'_, T> {
        Iter::new(self)
    }
}

pub(crate) struct Iter<'a, T> {
    // number of items at current the iterator creating moment
    len: usize,
    chunks: SmallVec<[(*mut u8, usize); 4]>,
    chunk_start: *const u8,
    chunk_size: usize,
    cur_ptr: *const u8,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iter<'a, T> {
    const ALLOC_SIZE: usize = std::alloc::Layout::new::<T>().size();

    fn new(arena: &'a Arena<T>) -> Self {
        let chunk_iter = unsafe { arena.bump.iter_allocated_chunks_raw() };
        Self {
            len: arena.len.get(),
            chunks: chunk_iter.collect::<SmallVec<[(*mut u8, usize); 4]>>(),
            chunk_start: std::ptr::null(),
            chunk_size: 0,
            cur_ptr: std::ptr::null(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, T> std::iter::Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;

        if self.chunk_start == self.cur_ptr {
            (self.chunk_start, self.chunk_size) = self.chunks.pop().unwrap();
            // set cur_ptr to the last chunk's element because the elements
            // order is reversed
            self.cur_ptr = unsafe { self.chunk_start.add(self.chunk_size).sub(Self::ALLOC_SIZE) };
        } else {
            self.cur_ptr = unsafe { self.cur_ptr.sub(Self::ALLOC_SIZE) };
        }
        assert!(self.chunk_start <= self.cur_ptr);

        Some(unsafe { &*(self.cur_ptr as *mut T as *const T) })
    }
}

#[cfg(test)]
mod utest {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn arena_iter() {
        // if right order for some chunks inside of the arena
        let arena = Arena::with_capacity(10);
        let mut items = Vec::new();
        for i in 0..1000 {
            items.push(i);
            arena.alloc_with(|| i);
        }

        let iter = arena.iter();
        let collect = iter.copied().collect::<Vec<_>>();
        assert_eq!(collect, items);

        let arena = Arena::<u32>::with_capacity(10);
        assert_eq!(arena.iter().collect::<Vec<_>>(), Vec::<&u32>::new());
    }

    #[test]
    fn arena_alloc_during_iteration() {
        let arena = Arena::with_capacity(0);
        let mut items = vec![1, 2, 3, 4, 5];
        for item in &items {
            arena.alloc_with(|| *item);
        }

        for item in arena.iter() {
            let _ = arena.alloc_with(|| 2 * *item);
            items.push(2 * *item);
        }

        assert_eq!(items.len(), 10);
        assert_eq!(items, arena.iter().copied().collect::<Vec<_>>());
    }
}
