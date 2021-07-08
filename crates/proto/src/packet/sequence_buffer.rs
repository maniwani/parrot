use core::ops::Range;

pub type SequenceNumber = u64;

pub(crate) struct SequenceBuffer<T> {
    sequences: Box<[Option<SequenceNumber>]>,
    data: Box<[Option<T>]>,
}

impl<T> SequenceBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            sequences: vec![None; capacity].into_boxed_slice(),
            data: vec![None; capacity].into_boxed_slice(),
        }
    }

    pub fn capacity(&self) -> usize {
        self.sequences.capacity()
    }

    #[inline]
    pub fn index_of(&self, sequence: SequenceNumber) -> usize {
        sequence as usize % self.data.len()
    }

    pub fn contains(&self, sequence: SequenceNumber) -> bool {
        self.sequences[self.index_of(sequence)] == Some(sequence)
    }

    #[allow(dead_code)]
    pub fn get(&self, sequence: SequenceNumber) -> Option<&Option<T>> {
        let index = self.index_of(sequence);
        if self.sequences[index] == Some(sequence) {
            Some(&self.data[index])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, sequence: SequenceNumber) -> Option<&mut Option<T>> {
        let index = self.index_of(sequence);
        if self.sequences[index] == Some(sequence) {
            Some(&mut self.data[index])
        } else {
            None
        }
    }

    pub fn get_index(&self, index: usize) -> (&Option<SequenceNumber>, &Option<T>) {
        (&self.sequences[index], &self.data[index])
    }

    pub fn get_index_mut(&mut self, index: usize) -> (&mut Option<SequenceNumber>, &mut Option<T>) {
        (&mut self.sequences[index], &mut self.data[index])
    }

    pub fn get_or_insert(&mut self, sequence: SequenceNumber, data: T) -> &mut T {
        if self.contains(sequence) {
            self.get_mut(sequence).as_mut().unwrap()
        } else {
            self.insert(sequence, data).unwrap()
        }
    }

    pub fn get_or_insert_with<F: FnOnce() -> T>(&mut self, sequence: SequenceNumber, f: F) -> &mut T {
        if self.contains(sequence) {
            self.get_mut(sequence).as_mut().unwrap()
        } else {
            self.insert(sequence, f()).unwrap()
        }
    }

    pub fn insert(&mut self, sequence: SequenceNumber, data: T) -> &mut T {
        let index = self.index_of(sequence);
        *self.sequences[index] = Some(sequence);
        *self.data[index] = Some(data);
        self.data[index].as_mut().unwrap()
    }

    pub fn remove(&mut self, sequence: SequenceNumber) -> Option<T> {
        let index = self.index_of(sequence);
        self.sequences[index].take();
        self.data[index].take()
    }

    pub fn remove_index(&mut self, index: usize) -> (Option<SequenceNumber>, Option<T>) {
        (self.sequences[index].take(), self.data[index].take())
    }

    pub fn remove_range(&mut self, range: Range<SequenceNumber>) {
        let start_idx = self.index_of(range.start);
        let end_idx = self.index_of(range.end);

        if end_idx < start_idx {
            self.sequences[..end_idx].fill(None);
            self.sequences[start_idx..].fill(None);
            self.entries[..end_idx].fill(None);
            self.entries[start_idx..].fill(None);
        } else {
            self.sequences[start_idx..end_idx].fill(None);
            self.entries[start_idx..end_idx].fill(None);
        }
    }

}