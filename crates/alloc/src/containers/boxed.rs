use std::mem::MaybeUninit;

use crate::{ptr::RelPtr, traits::Allocator};

/// A pointer type for heap allocation.
pub struct Box<T: ?Sized>(RelPtr<T, usize>);

pub struct Owned<'alloc, T, A: Allocator> {
    pub(crate) alloc: &'alloc A,
    pub(crate) inner: T,
}

impl<T> Box<T> {   
    pub fn new_in<A: Allocator>(value: T, alloc: &A) -> Owned<'_, Box<T>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }
    }

    pub fn new_uninit_in<A: Allocator>(alloc: &A) -> Owned<'_, Box<MaybeUninit<T>>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }
    }

    pub fn new_zeroed_in<A: Allocator>(alloc: &A) -> Owned<'_, Box<MaybeUninit<T>>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }
    }

    pub fn new_uninit_slice_in<A: Allocator>(alloc: &A) -> Owned<'_, Box<[MaybeUninit<T>]>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }
    }

    pub fn new_zeroed_slice_in<A: Allocator>(alloc: &A) -> Owned<'_, Box<[MaybeUninit<T>]>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }
    }

}

impl<T, A: Allocator> Owned<'_, Box<T>, A> {
    pub fn into_inner(boxed: Self, value: T) -> T {
        boxed.inner
    }
    
}

impl<T, A: Allocator> Owned<'_, Box<MaybeUninit<T>>, A> {
    pub fn assume_init(self) -> Owned<'_, Box<T>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }
    }

    pub fn write(self, value: T) -> Owned<'_, Box<T>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }
    }
}

impl<T, A: Allocator> Owned<'_, Box<[MaybeUninit<T>]>, A> {
    pub fn assume_init(self) -> Owned<'_, Box<[T]>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }     
    }

    pub fn write(boxed: Self, value: T) -> Owned<'_, Box<[T]>, A> {
        Owned { 
            alloc,
            inner: Box(RelPtr::with_addr(0)),
        }
    }
}