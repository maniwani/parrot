pub trait Allocator {}

pub trait AllocClone {}
pub trait AllocDrop {}

pub(crate) trait Borrow {
    type Ref<'a>
    where
        Self: 'a;
    type Mut<'a>
    where
        Self: 'a;

    fn as_ref<A: Allocator>(&self, alloc: &A) -> Self::Ref<'_>;
    fn as_mut<A: Allocator>(&mut self, alloc: &A) -> Self::Mut<'_>;
}

impl<T: Copy> Borrow for T {
    type Ref<'a> = &'a Self;
    type Mut<'a> = &'a mut Self;

    fn as_ref<A: Allocator>(&self, alloc: &A) -> Self::Ref<'_> {
        self
    }

    fn as_mut<A: Allocator>(&mut self, alloc: &A) -> Self::Mut<'_> {
        self
    }
}

/// Emulates `&T` for non-global allocator.
pub(crate) struct Ref<'alloc, 'scope, T, A>
where
    T: Drop,
    A: Allocator,
    'alloc: 'scope,
{
    alloc: &'alloc A,
    inner: &'scope T,
}

/// Emulates `&mut T` for non-global allocator.
pub(crate) struct Mut<'alloc, 'scope, T, A>
where
    T: Drop,
    A: Allocator,
    'alloc: 'scope,
{
    alloc: &'alloc A,
    inner: &'scope mut T,
}
