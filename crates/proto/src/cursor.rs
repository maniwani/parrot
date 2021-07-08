use std::{io::{self, ErrorKind, SeekFrom}, mem, fmt::Result};

use num_traits::PrimInt;

use super::encoding::{ZigZagEncode, ZigZagDecode};

/// A cursor on an immutable slice of bits.
///
/// `Bits` wraps an `&[u64]` and provides functions for doing sequential operations on it. 
/// The wrapped slice is not copied. Operations return `io::Result` and avoid out-of-bounds indexing.
/// The cursor position will update data is read. Manual seeking is also supported.
/// 
#[derive(Debug, PartialEq)]
pub struct Bits<'a> {
    inner: &'a [u64],
    pos: usize,
}

/// A cursor on a mutable slice of bits.
///
/// `BitsMut` wraps an `&mut [u64]` and provides functions for doing sequential operations on it. 
/// The wrapped slice is not copied. Operations return `io::Result` and avoid out-of-bounds indexing.
/// The cursor position will update as data is read or written. Manual seeking is also supported.
/// 
#[derive(Debug, PartialEq)]
pub struct BitsMut<'a> {
    inner: &'a mut [u64],
    pos: usize,
}

impl<'a> Bits<'a> {
    pub fn new(slice: &[u64]) -> Self {
        Self {
            inner: slice,
            pos: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len() * mem::size_of::<u64>() * 8
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.len().saturating_sub(self.position())
    }

    pub fn seek(&mut self, style: SeekFrom) -> io::Result<usize> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.len(), n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as usize)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as usize)
        };
        match new_pos {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(io::Error::new_const(
                ErrorKind::InvalidInput,
                &"invalid seek to a negative or overflowing position",
            )),
        }
    }

    #[inline]
    pub fn advance(&self, n: usize) -> io::Result<usize> {
        // if n > isize::MAX as usize
        self.seek(SeekFrom::Position(n as isize))
    }

    /// Copies the contents of the referenced slice into a new [`Vec`].
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
    
    pub(crate) unsafe fn peek_unchecked(&mut self, len: usize) -> u64 {
        let block = self.pos / (u64::BITS as usize);
        let bit = self.pos % (u64::BITS as usize);
        let read = (u64::BITS as usize) - bit;
        
        let mask = !0 >> (u64::BITS as usize - len);
        let x = self.inner.get_unchecked(block);
        let mut value = (*x & (mask << bit)) >> bit;
        
        if len > read {
            let x = self.inner.get_unchecked(block + 1);
            value |= (*x & (mask >> read)) << read;
        }
        
        value
    }

    pub fn peek(&mut self, len: usize) -> Result<u64, String> {
        if (len > self.remaining()) || (len > u64::BITS) {
            return Err(format!(""));
        }

        let value = unsafe { self.peek_unchecked(len) };
        Ok(value)
    }
    
    pub fn read(&mut self, len: usize) -> Result<u64, String> {
        let result = self.peek(len);
        if result.is_ok() {
            self.pos += len;
        }
        result
    }

    pub fn read_varint(&mut self) -> Result<i64, String> { 
        let len = self.read(6)?;
        let encoded = self.read((len + 1) as usize)?;
        let value = encoded.zig_zag_decode();
        Ok(value)
    }
}

impl<'a> BitsMut<'a> {
    pub fn new(slice: &mut [u64]) -> Self {
        Self {
            inner: slice,
            pos: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len() * mem::size_of::<u64>() * 8
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.len().saturating_sub(self.position())
    }

    pub fn seek(&mut self, style: SeekFrom) -> io::Result<usize> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.len(), n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as usize)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as usize)
        };
        match new_pos {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(io::Error::new_const(
                ErrorKind::InvalidInput,
                &"invalid seek to a negative or overflowing position",
            )),
        }
    }

    #[inline]
    pub fn advance(&self, n: usize) -> io::Result<usize> {
        // if n > isize::MAX as usize
        self.seek(SeekFrom::Position(n as isize))
    }

    /// Copies the contents of the referenced slice into a new [`Vec`].
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }

    pub(crate) unsafe fn peek_unchecked(&mut self, len: usize) -> u64 {
        let block = self.pos / (u64::BITS as usize);
        let bit = self.pos % (u64::BITS as usize);
        let read = (u64::BITS as usize) - bit;
        
        let mask = !0 >> (u64::BITS as usize - len);
        let x = self.inner.get_unchecked(block);
        let mut value = (*x & (mask << bit)) >> bit;
        
        if len > read {
            let x = self.inner.get_unchecked(block + 1);
            value |= (*x & (mask >> read)) << read;
        }
        
        value
    }

    pub fn peek(&mut self, len: usize) -> Result<u64, String> {
        if (len > self.remaining()) || (len > u64::BITS) {
            return Err(format!(""));
        }

        let value = unsafe { self.peek_unchecked(len) };
        Ok(value)
    }
    
    pub fn read(&mut self, len: usize) -> Result<u64, String> {
        let result = self.peek(len);
        if result.is_ok() {
            self.pos += len;
        }
        result
    }
    
    pub(crate) unsafe fn write_unchecked(&mut self, value: u64, len: usize) {
        let block = self.pos / (u64::BITS as usize);
        let bit = self.pos % (u64::BITS as usize);
        let written = u64::BITS as usize - bit;
        
        let mask = !0 >> (u64::BITS as usize - len);
        value &= mask;
        let x = self.inner.get_unchecked_mut(block);
        *x &= !(mask << bit);
        *x |= value << bit;
        
        if len > written {
            let x = self.inner.get_unchecked_mut(block + 1);
            *x &= !(mask >> written);
            *x |= value >> written;
        }
        
        self.pos += len;
    }

    pub fn write(&mut self, value: u64, len: usize) -> Result<(), String> {
        if (len > self.remaining()) || (len > u64::BITS) {
            return Err(format!(""));
        }

        unsafe { self.write_unchecked(value, len) };
        Ok(())
    }
    
    pub fn read_varint(&mut self) -> Result<i64, String> { 
        let len = self.read(6)?;
        let encoded = self.read((len + 1) as usize)?;
        let value = encoded.zig_zag_decode();
        Ok(value)
    }
    
    pub fn write_varint(&mut self, value: i64) -> Result<(), String> {      
        let encoded = value.zig_zag_encode();
        let len = (u64::BITS - encoded.leading_zeros()).max(1) as usize;
        self.write((len - 1) as u64, 6)?;
        self.write(encoded, len)?;
        Ok(())
    }
}

/// A cursor on an immutable slice of bytes.
///
/// `Bytes` wraps an `&[u8]` and provides functions for doing sequential operations on it. 
/// The wrapped slice is not copied. Operations return `io::Result` and avoid out-of-bounds indexing.
/// The cursor position will update data is read. Manual seeking is also supported.
/// 
#[derive(Debug, PartialEq)]
pub struct Bytes<'a> {
    inner: &'a [u8],
    pos: usize,
}

/// A cursor on a mutable slice of bytes.
///
/// `BytesMut` wraps an `&mut [u8]` and provides functions for doing sequential operations on it. 
/// The wrapped slice is not copied. Operations return `io::Result` and avoid out-of-bounds indexing.
/// The cursor position will update as data is read or written. Manual seeking is also supported.
/// 
#[derive(Debug, PartialEq)]
pub struct BytesMut<'a> {
    inner: &'a mut [u8],
    pos: usize,
}

impl<'a> Bytes<'a> {
    pub fn new(slice: &[u8]) -> Self {
        Self {
            inner: slice,
            pos: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the current cursor position.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns number of bytes remaining from the cursor position. 
    #[inline]
    pub fn remaining(&self) -> usize {
        self.len().saturating_sub(self.position())
    }

    pub fn seek(&mut self, style: SeekFrom) -> io::Result<usize> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.len(), n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as usize)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as usize)
        };
        match new_pos {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(io::Error::new_const(
                ErrorKind::InvalidInput,
                &"invalid seek to a negative or overflowing position",
            )),
        }
    }

    #[inline]
    pub fn advance(&self, n: usize) -> io::Result<usize> {
        // if n > isize::MAX as usize
        self.seek(SeekFrom::Position(n as isize))
    }

    /// Divides one `Bytes` into two `Bytes` at an index.
    ///
    /// The first will contain all bytes from `[0, mid)` (excluding the index `mid` itself) and
    /// the second will contain all bytes from `[mid, len)` (excluding the index `len` itself).
    pub fn split_at(&self, mid: usize) -> io::Result<(Bytes, Bytes)> {
        let (left, right) = self.inner.split_at(mid);
        Ok(Bytes { inner: left, pos: 0 }, Bytes { inner: right, pos: 0 })
    }

    /// Copies the contents of the referenced slice into a new [`Vec`].
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
}

impl<'a> BytesMut<'a> {
    pub fn new(slice: &mut [u8]) -> Self {
        Self {
            inner: slice,
            pos: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the current cursor position.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns number of bytes remaining from the cursor position. 
    #[inline]
    pub fn remaining(&self) -> usize {
        self.len().saturating_sub(self.position())
    }

    pub fn seek(&mut self, style: SeekFrom) -> io::Result<usize> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.len(), n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        let new_pos = if offset >= 0 {
            base_pos.checked_add(offset as usize)
        } else {
            base_pos.checked_sub((offset.wrapping_neg()) as usize)
        };
        match new_pos {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(io::Error::new_const(
                ErrorKind::InvalidInput,
                &"invalid seek to a negative or overflowing position",
            )),
        }
    }

    #[inline]
    pub fn advance(&self, n: usize) -> io::Result<usize> {
        self.seek(SeekFrom::Position(n as isize))
    }

    /// Divides one `BytesMut` into two `Bytes` at an index.
    ///
    /// The first will contain all bytes from `[0, mid)` (excluding the index `mid` itself) and
    /// the second will contain all bytes from `[mid, len)` (excluding the index `len` itself).
    pub fn split_at(&self, mid: usize) -> io::Result<(Bytes, Bytes)> {
        let (left, right) = self.inner.split_at(mid);
        Ok(Bytes { inner: left, pos: 0 }, Bytes { inner: right, pos: 0 })
    }

    /// Divides one `BytesMut` into two `BytesMut` at an index.
    ///
    /// The first will contain all bytes from `[0, mid)` (excluding the index `mid` itself) and
    /// the second will contain all bytes from `[mid, len)` (excluding the index `len` itself).
    pub fn split_at_mut(&mut self, mid: usize) -> io::Result<(BytesMut, BytesMut)> {
        let (left, right) = self.inner.split_at_mut(mid);
        Ok(BytesMut { inner: left, pos: 0 }, BytesMut { inner: right, pos: 0 })
    }

    /// Copies the contents of the referenced slice into a new [`Vec`].
    pub fn to_vec(&self) -> Vec<u8> {
        self.as_ref().to_vec()
    }
}

impl<'a> AsRef<[u8]> for Bytes<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[self.pos..]
    }
}

impl<'a> AsRef<[u8]> for BytesMut<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[self.pos..]
    }
}

impl<'a> AsMut<[u8]> for BytesMut<'a> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buf[self.pos..]
    }
}

impl<'a> Bytes<'a> {
    /// Reads a native endian integer from the current cursor position,
    /// without advancing the cursor.
    pub fn peek<T: PrimInt>(&self) -> io::Result<T> {
        let src = self.as_ref();
        if src.len() < mem::size_of::<T>() {
            return Err(io::Error::new_const(ErrorKind::InvalidInput, &"buffer too short"));
        }
        let mut dst = [0; mem::size_of::<T>()];
        dst.copy_from_slice(&src);
        Ok(T::from_be_bytes(dst))
    }

    /// Reads a native endian integer from the current cursor position,
    /// advancing the cursor by [`mem::size_of::<T>()`] bytes.
    pub fn read<T: PrimInt>(&mut self) -> io::Result<T> {
        let val = self.peek::<T>();
        self.pos += mem::size_of::<T>();   
        val
    }
}

impl<'a> BytesMut<'a> {
    /// Reads a native endian integer from the current cursor position,
    /// without advancing the cursor.
    pub fn peek<T: PrimInt>(&self) -> io::Result<T> {
        let src = self.as_ref();
        if src.len() < mem::size_of::<T>() {
            return Err(io::Error::new_const(ErrorKind::InvalidInput, &"buffer too short"));
        }
        let mut dst = [0; mem::size_of::<T>()];
        dst.copy_from_slice(&src);
        Ok(T::from_be_bytes(dst))
    }

    /// Reads a native endian integer from the current cursor position,
    /// advancing the cursor by [`mem::size_of::<T>()`] bytes.
    pub fn read<T: PrimInt>(&mut self) -> io::Result<T> {
        let val = self.peek::<T>()?;
        self.pos += mem::size_of::<T>();   
        Ok(val)
    }

    /// Writes a native endian integer at the current cursor position,
    /// advancing the cursor by [`mem::size_of::<T>()`] bytes.
    pub fn write<T: PrimInt>(&mut self, val: T) -> io::Result<()> {
        let dst = self.as_mut();
        if dst.len() <  mem::size_of::<T>() {
            return Err(io::Error::new_const(ErrorKind::InvalidInput, &"buffer too short"));        
        }
        dst[..mem::size_of::<T>()].copy_from_slice(&T::to_be_bytes(val));
        self.pos += mem::size_of::<T>();
        Ok(())    
    }

    /// Sets `count` bytes of the wrapped slice, starting at the cursor position, to `val`.
    /// Advances the cursor by `count` bytes.
    pub fn write_bytes(&mut self, val: u8, count: usize) -> io::Result<()> {
        if count > self.remaining() {
            return Err(io::Error::new_const(ErrorKind::InvalidInput, &"buffer too short"));
        }
        let dst = self.as_mut()[..count];
        dst.fill(val);
        self.pos += count;
        Ok(())
    }

    /// Copies all bytes from `src`, starting at the cursor position, using a memcpy.
    /// Advances the cursor by the length of the slice.
    pub fn copy_from_slice(&mut self, src: &[u8]) -> io::Result<()> {
        if src.len() > self.remaining() {
            return Err(io::Error::new_const(ErrorKind::InvalidInput, &"buffer too short"));
        }
        let dst = self.as_mut()[..src.len()];
        dst.copy_from_slice(src);
        self.pos += src.len();
        Ok(())
    }
}

impl<'a> Bytes<'a> {
    pub fn peek_varint(&self) -> io::Result<u64> {
        let cursor = Bytes::new(&self.as_ref());
        let desc = cursor.read_u8()?;        
        
        if desc == 0 {
            return Err(io::Error::new_const(
            ErrorKind::InvalidData,
            &"variable-size integer has invalid encoded length"))
        }
    
        let len = (desc.trailing_zeros() + 1) as usize;
        
        if cursor.remaining() < len {
            return Err(io::Error::new_const(
                ErrorKind::InvalidData,
                &"variable-size integer len greater than remaining len"))
        }
    
        let val = cursor.read_uint(len)?;
        Ok(val)
    }

    pub fn read_varint(&mut self) -> io::Result<u64> {
        let cursor = Bytes::new(&self.as_ref());
        let desc = cursor.read_u8()?;        
        
        if desc == 0 {
            return Err(io::Error::new_const(
            ErrorKind::InvalidData,
            &"variable-size integer has invalid encoded length"))
        }
    
        let len = (desc.trailing_zeros() + 1) as usize;
        
        if cursor.remaining() < len {
            return Err(io::Error::new_const(
                ErrorKind::InvalidData,
                &"variable-size integer len greater than remaining len"))
        }
    
        let val = cursor.read_uint(len)?;
        self.pos += len;
        Ok(val)
    }
}

impl<'a> BytesMut<'a> {
    pub fn peek_varint(&self) -> io::Result<u64> {
        let cursor = Bytes::new(&self.as_ref());
        let desc = cursor.read::<u8>()?;        
        
        if desc == 0 {
            return Err(io::Error::new_const(
            ErrorKind::InvalidData,
            &"variable-size integer has invalid encoded length"))
        }
    
        let len = (desc.trailing_zeros() + 1) as usize;
        
        if cursor.remaining() < len {
            return Err(io::Error::new_const(
                ErrorKind::InvalidData,
                &"variable-size integer len greater than remaining len"))
        }
    
        let val = u64::from_be_bytes(cursor.as_ref()[..len]);
        Ok(val)
    }

    pub fn read_varint(&mut self) -> io::Result<u64> {
        let cursor = Bytes::new(&self.as_ref());
        let desc = cursor.read::<u8>()?;        
        
        if desc == 0 {
            return Err(io::Error::new_const(
            ErrorKind::InvalidData,
            &"variable-size integer has invalid encoded length"))
        }
    
        let len = (desc.trailing_zeros() + 1) as usize;
        
        if cursor.remaining() < len {
            return Err(io::Error::new_const(
                ErrorKind::InvalidData,
                &"variable-size integer len greater than remaining len"))
        }
    
        let val = u64::from_be_bytes(cursor.as_ref()[..len]);
        self.pos += len;
        Ok(val)
    }

    pub fn write_varint(&mut self, val: u64) -> io::Result<()> {
        todo!()
    }
}
