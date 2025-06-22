use crate::AutomatonKind;
use crate::node::{Node, NodeInner};
use bumpalo::Bump;
use smallvec::SmallVec;
use std::cell::Cell;
use std::iter::Iterator;

pub struct Arena {
    node_bump: Bump,
    nodes_len: Cell<usize>,
    gr_data: Cell<Option<(u64, AutomatonKind)>>,
}

/// Public API
impl Arena {
    #[inline]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            node_bump: Bump::with_capacity(capacity * std::mem::size_of::<NodeInner>()),
            nodes_len: Cell::new(0),
            gr_data: Cell::new(None), // bound graph id
        }
    }

    pub fn alloc_node(&self) -> Node<'_> {
        let (gid, kind) = self
            .gr_data
            .get()
            .expect("you can't create a node until arena is bound to a graph");

        let nid: u32 = self.nodes_len.get().try_into().expect("node id overflow");
        self.nodes_len.replace(self.nodes_len.get() + 1);

        let uid: u64 = nid as u64 | (gid << Node::ID_BITS);

        let node_mut = self
            .node_bump
            .alloc_with(|| Node::new_inner(uid, self, kind));
        Node::from(node_mut)
    }

    /// Returns an iterator over the items in the arena.
    ///
    /// Because of the arena is a bump allocator, it is possible to add new
    /// items to the arena during iteration. But the iteration won't be affected
    /// by the new items, i.e. if at the moment of creation the iterator the
    /// arena contains `n` items, then the iterator will iterate over `n` items.
    pub fn nodes(&self) -> impl Iterator<Item = Node<'_>> {
        BumpIter::new(&self.node_bump, self.nodes_len.get()).map(|ptr| Node::from(unsafe { &*ptr }))
    }
}

/// Crate API
impl Arena {
    /// Binds this arena with a graph. Should be run by the graph constructor.
    ///
    /// We run nodes dropping here because I can't save mutable referance to the
    /// arena in the graph, but can have it in the graph constructor.
    pub(crate) fn set_graph_data(&mut self, gid: u64, kind: AutomatonKind) {
        if let Some((gid, _)) = self.gr_data.get() {
            panic!("this arena is already bound to a graph(gid={})", gid);
        }
        self.gr_data.set(Some((gid, kind)));
        self.drop_nodes();
        self.drop_nodes();
        self.drop_nodes();
        self.drop_nodes();
    }

    /// Unbinds this arena from a graph. Should be run by the graph destructor.
    ///
    /// Dispite expectations, this doesn't run nodes dropping, because the graph
    /// can't hold a mutable reference to the arena. So the dropping is run by
    /// either a new graph constructor or the arena destructor.
    pub(crate) fn reset_graph_data(&self) {
        self.gr_data.set(None);
    }
}

/// Private API
impl Arena {
    fn drop_nodes(&mut self) {
        if self.nodes_len.get() != 0 {
            let iter = BumpIter::<NodeInner>::new(&self.node_bump, self.nodes_len.get());
            for (i, ptr) in iter.enumerate() {
                // check layout within the bump allocator; each next node should have next node id
                assert_eq!(Node::from(unsafe { &*ptr }).nid() as usize, i);
                unsafe { std::ptr::drop_in_place::<NodeInner>(ptr) };
            }
            self.nodes_len.set(0);
            self.node_bump.reset();
        }
    }
}

impl std::default::Default for Arena {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Drop for Arena {
    fn drop(&mut self) {
        self.drop_nodes();
    }
}

/// Iterator over the one type items within the Bump arena.
struct BumpIter<T> {
    // number of items at the moment when this iterator was created
    len: usize,
    chunks: SmallVec<[(*mut u8, usize); 4]>,
    chunk_start: *const u8,
    chunk_size: usize,
    cur_ptr: *const u8,
    _phantom: std::marker::PhantomData<*mut T>,
}

impl<T> BumpIter<T> {
    const ALLOC_SIZE: usize = std::alloc::Layout::new::<T>().size();

    fn new(bump: &Bump, len: usize) -> Self {
        let chunk_iter = unsafe { bump.iter_allocated_chunks_raw() };
        Self {
            len,
            chunks: chunk_iter.collect::<SmallVec<[(*mut u8, usize); 4]>>(),
            chunk_start: std::ptr::null(),
            chunk_size: 0,
            cur_ptr: std::ptr::null(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> std::iter::Iterator for BumpIter<T> {
    type Item = *mut T;

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

        Some(self.cur_ptr as *mut T)
    }
}

#[cfg(test)]
mod utest {
    // use super::*;
    // use pretty_assertions::assert_eq;

    // #[test]
    // fn arena_iter() {
    //     // if right order for some chunks inside of the arena
    //     let arena = Arena::with_capacity(10);
    //     let mut items = Vec::new();
    //     for i in 0..1000 {
    //         items.push(i);
    //         arena.alloc_with(|| i);
    //     }

    //     let iter = arena.iter();
    //     let collect = iter.copied().collect::<Vec<_>>();
    //     assert_eq!(collect, items);

    //     let arena = Arena::<u32>::with_capacity(10);
    //     assert_eq!(arena.iter().collect::<Vec<_>>(), Vec::<&u32>::new());
    // }

    // #[test]
    // fn arena_alloc_during_iteration() {
    //     let arena = Arena::with_capacity(0);
    //     let mut items = vec![1, 2, 3, 4, 5];
    //     for item in &items {
    //         arena.alloc_with(|| *item);
    //     }

    //     for item in arena.iter() {
    //         let _ = arena.alloc_with(|| 2 * *item);
    //         items.push(2 * *item);
    //     }

    //     assert_eq!(items.len(), 10);
    //     assert_eq!(items, arena.iter().copied().collect::<Vec<_>>());
    // }
}
