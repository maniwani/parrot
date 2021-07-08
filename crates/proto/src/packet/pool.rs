use std::mem::MaybeUninit;

type ConnectionId = u64;
type ChannelId = u64;

pub struct BufferPool {
    bufs: Vec<Box<[MaybeUninit<u8>]>>,
    meta: Vec<BufferMetadata>,
    buffer_size: usize,
    capacity: usize,
    capacity_remaining: usize,
}

pub struct BufferMetadata {
    pub(super) holder: Option<(ConnectionId, ChannelId)>,
    pub(super) generation: u32,
    pub(super) prev: Option<usize>,
    pub(super) next: Option<usize>,
}

pub struct BufferHandle {
    generation: u32,
    index: u32,
}

impl BufferPool {
    pub fn new(buffer_size: usize, capacity: usize) -> Self {
        let mut meta = Vec::with_capacity(capacity);        
        let mut bufs = Vec::with_capacity(capacity);       
        
        for i in 0..capacity {
            let metadata = {
                BufferMetadata {
                    holder: None,
                    generation: 0,
                    prev: if i == 0 { None } else { Some(i - 1) },
                    next: if i == (capacity - 1) { None } else { Some(i + 1) },
                }
            };
            
            meta.push(metadata);
            bufs.push(Box::<[u8]>::new_uninit_slice(buffer_size));
        }
        
        Self {
            bufs,
            meta,
            buffer_size,
            capacity,
            capacity_remaining: capacity,
        }
    }

    pub fn capacity_remaining(&self) -> usize {
        self.capacity_remaining
    }
    
    pub fn get(&self, handle: BufferHandle) -> Option<&[MaybeUninit<u8>]> {
        self.meta
            .get(handle.index as usize)
            .and_then(|metadata| {
                if handle.generation != metadata.generation {
                    return None;
                }
                let buf = self.bufs.get(handle.index as usize).unwrap();
                Some(buf.as_ref())
            })
    }

    pub fn get_mut(&self, handle: BufferHandle) -> Option<&mut [MaybeUninit<u8>]> {
        self.meta
            .get(handle.index as usize)
            .and_then(|metadata| {
                if handle.generation != metadata.generation {
                    return None;
                }
                let buf = self.bufs.get_mut(handle.index as usize).unwrap();
                Some(buf.as_mut())
            })
    }
    
    pub fn acquire(&mut self) -> Result<BufferHandle, ()> {
        if self.capacity_remaining > 0 {
            // TODO: pop freelist
            self.capacity_remaining -= 1;

            let handle = {
                BufferHandle { 
                    index: 0,
                    generation: 0
                }
            };

            Ok(handle)
        } else {
            Err(())
        }
    }
    
    pub fn release(&mut self, handle: BufferHandle) -> Result<(), ()> {
        self.meta
            .get_mut(handle.index)
            .and_then(|metadata| {
                metadata.holder = None;
                metadata.generation += 1;
                // TODO: push freelist
                self.capacity_remaining += 1;
            })
            .ok_or(0);
    }
}


fn main() {
    // capacity = max connections * (2 * max packets per tick)
    let mut pool = BufferPool::new(1232, 1024);
    let handle = pool.acquire().unwrap();
    let buf = pool.get_mut(handle).unwrap();
}