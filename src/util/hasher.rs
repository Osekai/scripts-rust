use std::hash::{BuildHasher, Hasher};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct IntHasher;

impl BuildHasher for IntHasher {
    type Hasher = IntHash;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        IntHash(0)
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
/// Efficient hasher for integers
pub struct IntHash(u64);

#[rustfmt::skip]
impl Hasher for IntHash {
    fn write(&mut self, _: &[u8])     { unreachable!(); }
    fn write_u128(&mut self, _: u128) { unreachable!(); }
    fn write_i128(&mut self, _: i128) { unreachable!(); }

    #[inline] fn write_u8(&mut self, n: u8)       { self.0 = u64::from(n) }
    #[inline] fn write_u16(&mut self, n: u16)     { self.0 = u64::from(n) }
    #[inline] fn write_u32(&mut self, n: u32)     { self.0 = u64::from(n) }
    #[inline] fn write_u64(&mut self, n: u64)     { self.0 = n }
    #[inline] fn write_usize(&mut self, n: usize) { self.0 = n as u64 }

    #[inline] fn write_i8(&mut self, n: i8)       { self.0 = n as u64 }
    #[inline] fn write_i16(&mut self, n: i16)     { self.0 = n as u64 }
    #[inline] fn write_i32(&mut self, n: i32)     { self.0 = n as u64 }
    #[inline] fn write_i64(&mut self, n: i64)     { self.0 = n as u64 }
    #[inline] fn write_isize(&mut self, n: isize) { self.0 = n as u64 }

    #[inline] fn finish(&self) -> u64 { self.0 }
}
