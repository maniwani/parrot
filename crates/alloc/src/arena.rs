use core::{
    alloc::Layout,
    cell::{Cell, UnsafeCell},
    mem, ptr,
};

use bitvec::{bitarr, order::Lsb0, BitArr};
use bytesize::{ByteSize, KIB};
use log::error;

use super::ptr::*;

const OS_PAGE_SIZE: usize = 4 * KIB as usize;
const OS_PAGE_SHIFT: usize = 12; // 4 KiB == 4096 B == 1 << 12
const OS_PAGE_MASK: usize = !(OS_PAGE_SIZE - 1); // masks off the lower bits

// 16 block sizes in each group
// group | step size | min. size
//     0 |       8 B |       0 B
//     1 |       8 B |     128 B  <--  BASE_SIZE
//     2 |      16 B |     256 B
//     3 |      32 B |     512 B
//     4 |      64 B |     1 KiB
//     5 |     128 B |     2 KiB
//     6 |     256 B |     4 KiB
//     7 |     512 B |     8 KiB
//     8 |     1 KiB |    16 KiB
//     9 |     2 KiB |    32 KiB
//    10 |     4 KiB |    64 KiB

const BINS_PER_GROUP: usize = 16;
const BASE_STEP: usize = 8;
const BASE_SIZE: usize = BASE_STEP * BINS_PER_GROUP;

/// Returns the index of the page bin corresponding to the given block size (in bytes).
const fn size_to_bin(mut bytes: usize) -> usize {
    let group = if bytes < BASE_SIZE {
        0
    } else {
        (1 + bytes.log2() - BASE_SIZE.log2()) as usize
    };
    let step = if group == 0 {
        BASE_STEP
    } else {
        BASE_STEP << (group - 1)
    };
    let min_bytes = if group == 0 { 0 } else { BINS_PER_GROUP * step };

    bytes -= min_bytes;
    (group * BINS_PER_GROUP) + (bytes / step) + (((bytes % step) != 0) as usize)
}

/// Returns the block size (in bytes) corresponding to the given bin index.
const fn bin_to_size(index: usize) -> usize {
    let group = index / BINS_PER_GROUP;
    let step = if group == 0 {
        BASE_STEP
    } else {
        BASE_STEP << (group - 1)
    };
    let min_bytes = if group == 0 { 0 } else { BINS_PER_GROUP * step };

    min_bytes + ((index % BINS_PER_GROUP) * step)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AllocInit {
    /// The new memory is left uninitialized.
    Uninitialized,
    /// The new memory is zeroed.
    Zeroed,
}

/// An error with allocating or deallocating memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllocError {
    OutOfMemory,
    RequestTooLarge,
    PointerOutsideRange,
    PointerNotAligned,
    BlockAlreadyFree,
}

/// A (free) block of memory.
#[repr(C)]
struct Block {
    // pointer to next free block (in the same page)
    next: Option<RelPtr<Block, usize>>,
}

/// A contiguous memory region containing blocks of a fixed size.
#[repr(C)]
struct Page {
    index: usize,
    // number of blocks that have been allocated
    used: usize,
    // pointer to next free block (in the page)
    free: Option<RelPtr<Block, usize>>,
    // index of next free page with blocks of the same size
    next: Option<usize>,
    // index of prev free page with blocks of the same size
    prev: Option<usize>,
    // index of bin corresponding to block size
    bin: Option<usize>,
    // 1 bit per block, guards against double-frees (16 KiB / 8 B => max. 2048 blocks per page)
    bitset: BitArr!(for 2048, in Cell<usize>, Lsb0),
}

/// A collection of pages with the same block size.
#[repr(C)]
struct Bin {
    index: usize,
    // number of bytes in each block
    block_size: usize,
    // number of blocks in each page
    block_capacity: usize,
    // next page with free blocks (of size)
    free_page: Option<usize>,
}

/// A non-global allocator that (re)allocates blocks of fixed sizes from a contiguous memory region.
/// Blocks can be individually freed and reused.
///
/// For portability, this allocator returns pointers relative to its memory region's base address.
pub struct Arena {
    buf: UnsafeCell<Box<[u8]>>,
    heap_start: usize,
    page_size: usize,
    page_count: usize,
    bin_count: usize,
}

impl Arena {
    /// Constructs a new `Arena` with the specified page size and page count.
    ///
    /// # Panics
    ///
    /// Panics if
    /// - `page_size` is not a power of 2
    /// - `page_size` is smaller than the operating system page size
    /// - `page_size` * `page_count` (plus metadata) exceeds `isize::MAX` bytes
    pub fn new(page_size: usize, page_count: usize) -> Self {
        assert!(page_size.is_power_of_two());
        assert!(page_size >= OS_PAGE_SIZE);
        let bin_count = size_to_bin(page_size) + 1;
        let meta_size = mem::size_of::<Option<usize>>()
            + (mem::size_of::<Page>() * page_count)
            + (mem::size_of::<Bin>() * bin_count);
        let heap_size = page_size * page_count;

        let mut buf = vec![0u8; meta_size + heap_size];
        let (meta, heap) = buf.split_at_mut(meta_size);

        // SAFETY: allocated enough space
        unsafe {
            let ptr = meta.as_mut_ptr();
            // write bin metadata
            let mut ptr = ptr.cast::<Bin>();
            for i in 0..bin_count {
                let block_size = bin_to_size(i);
                ptr.write(Bin {
                    index: i,
                    block_size,
                    block_capacity: if block_size == 0 {
                        usize::MAX
                    } else {
                        page_size / block_size
                    },
                    free_page: None,
                });
                ptr = ptr.add(1);
            }
            // write page metadata
            let mut ptr = ptr.cast::<Page>();
            for i in 0..page_count {
                ptr.write(Page {
                    index: i,
                    free: None,
                    next: Some(i + 1),
                    prev: if i == 0 { None } else { Some(i - 1) },
                    bin: None,
                    used: 0,
                    bitset: bitarr![Cell<usize>, Lsb0; 0; 2048],
                });
                ptr = ptr.add(1);
            }
            // write head of free page list
            let ptr = ptr.cast::<Option<usize>>();
            ptr.write(Some(0));
            assert_eq!(ptr.add(1) as usize, heap.as_ptr() as usize);
        }

        Self {
            buf: UnsafeCell::new(buf.into_boxed_slice()),
            heap_start: meta_size,
            page_size,
            page_count,
            bin_count,
        }
    }

    /// Returns a pointer from a [`RelPtr`].
    ///
    /// ## Safety
    /// - Pointee must have been allocated and intialized.
    pub unsafe fn get<T>(&self, rel_ptr: RelPtr<T, usize>) -> Option<*mut T> {
        self.get_ptr::<T>(rel_ptr.addr())
    }

    /// Returns a pointer to the data at `addr`.
    unsafe fn get_ptr<T>(&self, addr: usize) -> Option<*mut T> {
        assert!(
            mem::size_of::<T>() != 0,
            "we aren't ready to handle zero-sized types"
        );
        if addr >= (*self.buf.get())[self.heap_start..].len() {
            // outside heap
            return None;
        }

        // SAFETY: the addr is inside heap, so page is valid
        let page = self.get_page_unchecked(self.get_page_index(addr));

        if let Some(index) = (*page).bin {
            let bin = self.get_bin(index);
            if mem::size_of::<T>() > (*bin).block_size {
                // size of T > size of block
                return None;
            }

            let addr_in_page = self.get_addr_in_page(addr);
            if (addr_in_page % (*bin).block_size) != 0 {
                // addr does not point to the start of a block
                return None;
            }

            let block_index = addr_in_page / (*bin).block_size;
            if !(*page).bitset.get_unchecked(block_index) {
                // block not in use
                return None;
            }

            // the pointer is valid, block is in use, and size of T <= size of block
            return Some(self.get_ptr_unchecked(addr));
        }

        // page not in use
        None
    }

    /// Returns a pointer to the data at `addr`.
    ///
    /// This does not check if `addr` points to an actual block, if that block is in use,
    /// or if the block is large enough to hold a `T`.
    unsafe fn get_ptr_unchecked<T>(&self, addr: usize) -> *mut T {
        (*self.buf.get())[self.heap_start..]
            .as_mut_ptr()
            .add(addr)
            .cast()
    }

    /// Allocates memory.
    ///
    /// Returns a [`RelPtr`] to an unitialized block that meets the size and alignment required by
    /// `layout`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if there is no memory available that meets the requirements.
    pub fn allocate(&self, layout: Layout) -> Result<RelPtr<[u8], usize>, AllocError> {
        let size = layout.size();
        assert!(size != 0, "we aren't ready to handle zero-sized types");

        if size > self.page_size {
            error!(
                "size requested is larger than the maximum block size: {} > {}",
                ByteSize::b(size as u64).to_string_as(true),
                ByteSize::b(self.page_size as u64).to_string_as(true)
            );
            return Err(AllocError::RequestTooLarge);
        }

        unsafe {
            let bin = self.get_bin_unchecked(size_to_bin(size));
            let page = match (*bin).free_page {
                Some(index) => {
                    let page = self.get_page_unchecked(index);
                    assert_eq!((*page).bin, Some((*bin).index));
                    page
                }
                None => {
                    match self.pop_page(self.free_page()) {
                        Some(page) => {
                            (*page).bin = Some((*bin).index);
                            // construct block freelist
                            let mut addr = (*page).index * self.page_size;
                            for i in 0..(*bin).block_capacity {
                                let block = self.get_ptr_unchecked::<Block>(addr);
                                addr += (*bin).block_size;
                                let next = {
                                    if i == ((*bin).block_capacity - 1) {
                                        None
                                    } else {
                                        Some(RelPtr::with_addr(addr))
                                    }
                                };
                                block.write(Block { next });
                            }
                            self.push_page(ptr::addr_of_mut!((*bin).free_page), page);
                            page
                        }
                        None => {
                            // TODO: Try from the larger bins?
                            return Err(AllocError::OutOfMemory);
                        }
                    }
                }
            };

            assert!((*page).free.is_some());
            let ptr = (*page).free.unwrap();
            let addr = ptr.addr();
            assert_eq!(self.get_page_index(addr), (*page).index);

            let block_index = self.get_addr_in_page(addr) / (*bin).block_size;
            assert!(!(*page).bitset.get_unchecked(block_index));

            let block = self.get_ptr_unchecked::<Block>(addr);
            (*page).free = (*block).next;
            (*block).next = None;
            (*page).bitset.set_aliased_unchecked(block_index, true);
            (*page).used += 1;

            if (*page).used == (*bin).block_capacity {
                self.remove_page(ptr::addr_of_mut!((*bin).free_page), page);
            }

            // ptr::write_bytes(block as *mut u8, 0, bin.block_size);

            Ok(RelPtr::with_addr(addr))
        }
    }

    /// Frees allocated memory.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the pointer is invalid or the pointee block was not in use.
    pub fn deallocate(&self, rel_ptr: RelPtr<u8, usize>) -> Result<(), AllocError> {
        unsafe {
            let addr = rel_ptr.addr();
            if addr >= (*self.buf.get())[self.heap_start..].len() {
                return Err(AllocError::PointerOutsideRange);
            }

            let page = self.get_page_unchecked(self.get_page_index(addr));
            assert!((*page).bin.is_some());
            let bin = self.get_bin((*page).bin.unwrap());

            let addr_in_page = self.get_addr_in_page(addr);
            if (addr_in_page % (*bin).block_size) != 0 {
                return Err(AllocError::PointerNotAligned);
            }

            let block_index = addr_in_page / (*bin).block_size;
            if !(*page).bitset.get_unchecked(block_index) {
                return Err(AllocError::BlockAlreadyFree);
            }

            if (*page).used == (*bin).block_capacity {
                self.push_page(ptr::addr_of_mut!((*bin).free_page), page);
            }

            let block = self.get_ptr_unchecked::<Block>(addr);
            block.write(Block { next: (*page).free });
            (*page).free = Some(rel_ptr.cast());
            (*page).bitset.set_aliased_unchecked(block_index, false);
            (*page).used -= 1;

            if (*page).used == 0 {
                self.remove_page(ptr::addr_of_mut!((*bin).free_page), page);
                self.push_page(self.free_page(), page);
                (*page).bin = None;
            }

            Ok(())
        }
    }

    /// Reallocates allocated memory.
    ///
    /// If the new layout maps to the same block size, this function returns the same pointer.
    /// Otherwise, this function will allocate a new block, copy `min(old, new)` bytes from the old block
    /// to the new block, then free the old block.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the new layout is too large, the pointer is invalid, or the pointee is
    /// already free.
    ///
    /// If `rel_ptr` was valid, its pointee's contents remain unaltered.
    pub fn reallocate(
        &self,
        rel_ptr: RelPtr<[u8], usize>,
        new_layout: Layout,
    ) -> Result<RelPtr<[u8], usize>, AllocError> {
        unsafe {
            if rel_ptr.addr() >= (*self.buf.get())[self.heap_start..].len() {
                return Err(AllocError::PointerOutsideRange);
            }

            let old_page = self.get_page_unchecked(self.get_page_index(rel_ptr.addr()));
            assert!((*old_page).bin.is_some());
            let old_bin = self.get_bin((*old_page).bin.unwrap());
            let old_size = (*old_bin).block_size;

            let addr_in_page = self.get_addr_in_page(rel_ptr.addr());
            if (addr_in_page % old_size) != 0 {
                return Err(AllocError::PointerNotAligned);
            }

            let block_index = addr_in_page / old_size;
            if !(*old_page).bitset.get_unchecked(block_index) {
                return Err(AllocError::BlockAlreadyFree);
            }

            let new_size = new_layout.size();
            let new_bin = self.get_bin(size_to_bin(new_size));

            if (*new_bin).index != (*old_bin).index {
                self.allocate(new_layout).and_then(|new_ptr| {
                    // copy
                    let old = self.get_ptr_unchecked::<u8>(rel_ptr.addr());
                    let new = self.get_ptr_unchecked::<u8>(new_ptr.addr());
                    new.copy_from_nonoverlapping(old, old_size.min(new_size));

                    // deallocate
                    if (*old_page).used == (*old_bin).block_capacity {
                        self.push_page(ptr::addr_of_mut!((*old_bin).free_page), old_page);
                    }

                    let block = self.get_ptr_unchecked::<Block>(rel_ptr.addr());
                    block.write(Block {
                        next: (*old_page).free,
                    });
                    (*old_page).free = Some(rel_ptr.cast());
                    (*old_page).bitset.set_aliased_unchecked(block_index, false);
                    (*old_page).used -= 1;

                    if (*old_page).used == 0 {
                        self.remove_page(ptr::addr_of_mut!((*old_bin).free_page), old_page);
                        self.push_page(self.free_page(), old_page);
                        (*old_page).bin = None;
                    }
                    Ok(new_ptr)
                })
            } else {
                Ok(rel_ptr)
            }
        }
    }

    /// Returns `true` if the allocator contains the pointer address.
    #[inline]
    pub fn contains(&self, ptr: *const u8) -> bool {
        unsafe {
            (*self.buf.get())[self.heap_start..]
                .as_ptr_range()
                .contains(&ptr)
        }
    }

    /// Returns the index of the page containing the given addr.
    #[inline]
    fn get_page_index(&self, addr: usize) -> usize {
        addr >> self.page_size.log2()
    }

    /// Returns the given addr modulo the allocator's page size.
    #[inline]
    fn get_addr_in_page(&self, addr: usize) -> usize {
        addr & (self.page_size - 1)
    }

    /// Returns a pointer to the metadata for the specified page bin.
    #[inline]
    unsafe fn get_bin_unchecked(&self, index: usize) -> *mut Bin {
        (*self.buf.get())[..self.heap_start]
            .as_mut_ptr()
            .cast::<Bin>()
            .add(index)
    }

    /// Returns a pointer to the metadata for the specified page.
    #[inline]
    unsafe fn get_page_unchecked(&self, index: usize) -> *mut Page {
        (*self.buf.get())[..self.heap_start]
            .as_mut_ptr()
            .cast::<Bin>()
            .add(self.bin_count)
            .cast::<Page>()
            .add(index)
    }

    /// Returns a pointer to the metadata for the specified page bin.
    ///
    /// ## Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    fn get_bin(&self, index: usize) -> *mut Bin {
        assert!(index < self.bin_count);
        unsafe { self.get_bin_unchecked(index) }
    }

    /// Returns a pointer to the metadata for the specified page.
    ///
    /// ## Panics
    ///
    /// Panics if `index` is out of bounds.
    #[inline]
    fn get_page(&self, index: usize) -> *mut Page {
        assert!(index < self.page_count);
        unsafe { self.get_page_unchecked(index) }
    }

    /// Returns a pointer to the index of the next free page.
    #[inline]
    fn free_page(&self) -> *mut Option<usize> {
        // SAFETY: fixed address
        unsafe {
            (*self.buf.get())[..self.heap_start]
                .as_mut_ptr()
                .cast::<Bin>()
                .add(self.bin_count)
                .cast::<Page>()
                .add(self.page_count)
                .cast::<_>()
        }
    }

    // TODO: Replace these with linked list struct.
    unsafe fn pop_page(&self, list: *mut Option<usize>) -> Option<*mut Page> {
        (*list).and_then(|index| {
            let page = self.get_page(index);

            if let Some(prev_index) = (*page).prev {
                let prev_page = self.get_page(prev_index);
                (*prev_page).next = (*page).next;
                (*page).next = None;
            }

            if let Some(next_index) = (*page).next {
                let next_page = self.get_page(next_index);
                (*next_page).prev = (*page).prev;
                (*page).prev = None;
            }

            Some(page)
        })
    }

    unsafe fn push_page(&self, list: *mut Option<usize>, page: *mut Page) {
        assert!((*page).prev.is_none());
        assert!((*page).next.is_none());
        if let Some(index) = *list {
            let head = self.get_page(index);
            (*head).prev = Some((*page).index);
            (*page).next = Some((*head).index);
        }

        *list = Some((*page).index);
    }

    unsafe fn remove_page(&self, list: *mut Option<usize>, page: *mut Page) {
        if *list == Some((*page).index) {
            *list = (*page).next;
        }

        if let Some(prev_index) = (*page).prev {
            let prev_page = self.get_page(prev_index);
            (*prev_page).next = (*page).next;
            (*page).next = None;
        }

        if let Some(next_index) = (*page).next {
            let next_page = self.get_page(next_index);
            (*next_page).prev = (*page).prev;
            (*page).prev = None;
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
