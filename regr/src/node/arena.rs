use super::nfa::{Node, NodeInner};
use super::NodeId;
use bumpalo::Bump;
use std::cell::RefCell;

pub struct Arena<T> {
    bump: Bump,
    next_id: RefCell<NodeId>,
    _phantom: std::marker::PhantomData<T>,
}

enum NodeWrapper<'a, T> {
    Nfa(NodeInner<'a, T>),
}

impl<T: Sized> Arena<T> {
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            next_id: RefCell::new(0),
            _phantom: Default::default(),
        }
    }

    /// Creates a new NodeBuilder with preallocated memory for at least `capacity`
    /// nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity * std::mem::size_of::<NodeWrapper<T>>()),
            next_id: RefCell::new(0),
            _phantom: Default::default(),
        }
    }

    /// Creates a new NFA node.
    pub fn node_nfa(&self) -> Node<'_, T> {
        let new_id = self
            .next_id
            .replace_with(|v| v.checked_add(1).expect("node id overflow"));
        let wrap_ref = self
            .bump
            .alloc_with(|| NodeWrapper::Nfa(NodeInner::<T>::new(new_id)));

        match wrap_ref {
            NodeWrapper::Nfa(node_ref) => Node::new(node_ref),
        }
    }
}

impl<T> std::ops::Drop for Arena<T> {
    fn drop(&mut self) {
        // number of allocated nodes
        let mut len = *self.next_id.borrow();
        if len == 0 {
            return;
        }

        let alloc_size = std::alloc::Layout::new::<NodeWrapper<T>>().size();

        for (chunk, size) in unsafe { self.bump.iter_allocated_chunks_raw() } {
            let mut ptr = chunk;
            let mut dropped_size = alloc_size;

            loop {
                unsafe { std::ptr::drop_in_place::<NodeWrapper<T>>(ptr as *mut NodeWrapper<T>) };

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

impl<T: Sized> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}
