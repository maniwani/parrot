use crate::{ptr::RelPtr, traits::Allocator};

/// A contiguous growable array type, written as `Vec<T>`, short for 'vector'.
pub struct Vec<T> {
    ptr: RelPtr<T, usize>,
    len: usize,
    cap: usize,
}


pub struct Owned<'alloc, T, A: Allocator> {
    alloc: &'alloc A,
    inner: T,
}


impl<T> Vec<T> {
    pub fn new_in<A: Allocator>(alloc: &A) -> Owned<'_, Vec<T>, A> {
        Owned { 
            alloc,
            inner: Vec {
                ptr: RelPtr::with_addr(0),
                len: 0,
                cap: 0,
            },
        }
    }

    pub fn with_capacity_in<A: Allocator>(capacity: usize, alloc: &A) -> Owned<'_, Vec<T>, A> {
        // alloc.allocate(capacity * mem::size_of::<T>())
        Owned { 
            alloc,
            inner: Vec {
                ptr: RelPtr::with_addr(0),
                len: 0,
                cap: 0,
            },
        }
    }
}

impl<T, A: Allocator> Owned<'_, Vec<T>, A> {
    // append(&mut self, other: &mut ???)
    // as_mut_slice(&mut self) -> &mut [T]
    // as_slice(&self) -> &[T]
    // clear
    // is_empty
    // capacity
    // len
    // insert
    // push
    // pop
    // remove
    // swap_remove
    // truncate
    // into_boxed_slice
    // extend_from_slice
    // extend_from_within
    // as_ptr
    // as_mut_ptr

    // reserve
    // reserve_exact
    // resize
    // resize_with
    // retain
    // retain_mut
    // set_len
    // shrink_to
    // shrink_to_fit
    // spare_capacity_mut
    // splice
    // split_at_spare_mut
    // split_off
}

// Deref<Target=[T]>
// DerefMut<Target=[T]>
