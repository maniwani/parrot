use core::{marker::PhantomData, mem};
use num_traits::{PrimInt, Unsigned};

/// An error where the distance between two memory locations cannot be represented by the offset type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddressError {
    /// The offset overflowed the range of `isize`.
    IsizeOverflow,
}

fn offset_between(from: usize, to: usize) -> Result<isize, AddressError> {
    let (result, overflow) = to.overflowing_sub(from);
    if (!overflow && result <= (isize::MAX as usize))
        || (overflow && result >= (isize::MIN as usize))
    {
        Ok(result as isize)
    } else {
        Err(AddressError::IsizeOverflow)
    }
}

pub trait Address: PrimInt + Unsigned {
    fn from_usize(addr: usize) -> Self;
    fn to_usize(self) -> usize;
}

macro_rules! impl_address {
    ($ty:ty) => {
        impl Address for $ty {
            fn from_usize(addr: usize) -> Self {
                addr as Self
            }

            fn to_usize(self) -> usize {
                self as usize
            }
        }
    };
}

impl_address!(usize);
#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl_address!(u32);
#[cfg(target_pointer_width = "64")]
impl_address!(u64);

pub type RelPtrUsize<T> = RelPtr<T, usize>;
pub type RelPtrU32<T> = RelPtr<T, u32>;
pub type RelPtrU64<T> = RelPtr<T, u64>;

/// A strongly-typed pointer to a memory address, relative to some base address.
#[repr(transparent)]
pub struct RelPtr<T: ?Sized, P: Address> {
    addr: P,
    _marker: PhantomData<*mut T>,
}

impl<T: ?Sized, P: Address> Copy for RelPtr<T, P> {}

impl<T: ?Sized, P: Address> Clone for RelPtr<T, P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized, P: Address> RelPtr<T, P> {
    pub(crate) fn with_addr(addr: usize) -> Self {
        Self {
            addr: P::from_usize(addr),
            _marker: PhantomData,
        }
    }

    pub fn addr(self) -> usize {
        self.addr.to_usize()
    }

    pub fn cast<U>(self) -> RelPtr<U, P> {
        RelPtr::with_addr(self.addr.to_usize())
    }
}

// pointer arithmetic
impl<T: Sized, P: Address> RelPtr<T, P> {
    unsafe fn add(self, count: usize) -> Self {
        let addr = self
            .addr
            .to_usize()
            .wrapping_add(count * mem::size_of::<T>());
        Self {
            addr: P::from_usize(addr),
            _marker: PhantomData,
        }
    }

    unsafe fn sub(self, count: usize) -> Self {
        let addr = self
            .addr
            .to_usize()
            .wrapping_sub(count * mem::size_of::<T>());
        Self {
            addr: P::from_usize(addr),
            _marker: PhantomData,
        }
    }
}

// read
impl<T: Sized, P: Address> RelPtr<T, P> {
    unsafe fn read<A>(self, alloc: A) -> () {}

    unsafe fn read_volatile<A>(self, alloc: A) -> () {}

    unsafe fn read_unaligned<A>(self, alloc: A) -> () {}

    unsafe fn copy_from<A>(self, src: Self, count: usize, alloc: A) {}

    unsafe fn copy_from_nonoverlapping<A>(self, src: Self, count: usize, alloc: A) {}
}

// write
impl<T: Sized, P: Address> RelPtr<T, P> {
    unsafe fn write<A>(self, val: T, alloc: A) {}

    unsafe fn write_bytes<A>(self, val: u8, count: usize, alloc: A) {}

    unsafe fn write_volatile<A>(self, val: T, alloc: A) {}

    unsafe fn write_unaligned<A>(self, val: T, alloc: A) {}

    unsafe fn replace<A>(self, val: T, alloc: A) -> () {}

    unsafe fn swap<A>(self, with: Self, alloc: A) {}
}

// drop
impl<T: ?Sized, P: Address> RelPtr<T, P> {
    unsafe fn drop_in_place<A>(self, alloc: A) {}
}
