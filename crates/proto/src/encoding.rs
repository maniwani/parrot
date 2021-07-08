use num_traits::{Float, PrimInt, Signed, Unsigned};

macro_rules! impl_zig_zag_decode {
    ($n:expr, $signed:ty) => {
        (($n >> 1) ^ (!($n & 1)).wrapping_add(1)) as $signed
    };
}

macro_rules! impl_zig_zag_encode {
    ($n:expr, $signed:ty, $unsigned:ty) => {
        (($n << 1) ^ ($n >> (<$signed>::BITS - 1))) as $unsigned
    };
}

pub(crate) trait ZigZagDecode<S: PrimInt + Signed>: PrimInt + Unsigned {
    fn zig_zag_decode(self) -> S {}
}

impl ZigZagDecode<i8> for u8 {
    fn zig_zag_decode(self) -> i8 {
        impl_zig_zag_decode!(self, i8)
    }
}

impl ZigZagDecode<i16> for u16 {
    fn zig_zag_decode(self) -> i16 {
        impl_zig_zag_decode!(self, i16)
    }
}

impl ZigZagDecode<i32> for u32 {
    fn zig_zag_decode(self) -> i32 {
        impl_zig_zag_decode!(self, i32)
    }
}

impl ZigZagDecode<i64> for u64 {
    fn zig_zag_decode(self) -> i64 {
        impl_zig_zag_decode!(self, i64)
    }
}

impl ZigZagDecode<i128> for u128 {
    fn zig_zag_decode(self) -> i128 {
        impl_zig_zag_decode!(self, i128)
    }
}

impl ZigZagDecode<isize> for usize {
    fn zig_zag_decode(self) -> isize {
        impl_zig_zag_decode!(self, isize)
    }
}

pub(crate) trait ZigZagEncode<U: PrimInt + Unsigned>: PrimInt + Signed {
    fn zig_zag_encode(self) -> U {}
}

impl ZigZagEncode<u8> for i8 {
    fn zig_zag_encode(self) -> u8 {
        impl_zig_zag_encode!(self, i8, u8)
    }
}

impl ZigZagEncode<u16> for i16 {
    fn zig_zag_encode(self) -> u16 {
        impl_zig_zag_encode!(self, i16, u16)
    }
}

impl ZigZagEncode<u32> for i32 {
    fn zig_zag_encode(self) -> u32 {
        impl_zig_zag_encode!(self, i32, u32)
    }
}

impl ZigZagEncode<u64> for i64 {
    fn zig_zag_encode(self) -> u64 {
        impl_zig_zag_encode!(self, i64, u64)
    }
}

impl ZigZagEncode<u128> for i128 {
    fn zig_zag_encode(self) -> u128 {
        impl_zig_zag_encode!(self, i128, u128)
    }
}

impl ZigZagEncode<usize> for isize {
    fn zig_zag_encode(self) -> usize {
        impl_zig_zag_encode!(self, isize, usize)
    }
}

pub trait RadixEncode<T: PrimInt>: Float {
    fn radix_encode(self) -> T {}
}

impl RadixEncode<u32> for f32 {
    fn radix_encode(self) -> u32 {
        todo!()
    }
}

impl RadixEncode<u64> for f64 {
    fn radix_encode(self) -> u64 {
        todo!()
    }
}
