use bumpalo::Bump;
use std::cell::RefCell;

pub struct Arena<T> {
    bump: Bump,
    len: RefCell<usize>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Sized> Arena<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            bump: Bump::with_capacity(capacity * std::mem::size_of::<T>()),
            len: RefCell::new(0),
            _phantom: Default::default(),
        }
    }

    pub fn alloc_with<F>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        let refer = self.bump.alloc_with(f);
        *self.len.borrow_mut() += 1;
        refer
    }
}

impl<T> std::ops::Drop for Arena<T> {
    fn drop(&mut self) {
        // number of allocated nodes
        let mut len = *self.len.borrow();
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
