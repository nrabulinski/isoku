use tracing::error;

pub trait OsuEncode {
    fn encoded_size(&self) -> usize;
    fn encode(&self, buf: &mut Vec<u8>);
    fn decode<'a>(buf: &'a [u8]) -> Result<(&'a Self, usize), ()>;
}

macro_rules! encode_impl {
    ($t:ty) => {
        impl OsuEncode for $t {
            fn encoded_size(&self) -> usize { std::mem::size_of::<$t>() }

            fn encode(&self, buf: &mut Vec<u8>) {
                let s = unsafe {
                    std::slice::from_raw_parts(self as *const $t as *const u8, std::mem::size_of_val(self))
                };
                buf.extend_from_slice(s);
            }

            fn decode(buf: &[u8]) -> Result<(&Self, usize), ()> {
                if buf.len() < std::mem::size_of::<$t>() { Err(()) }
                else { Ok(unsafe {
                    (&*(buf.as_ptr() as *const $t), std::mem::size_of::<$t>())
                }) }
            }
        }
    };

    ($($t:ty),+) => { $(encode_impl!($t);)+ };
}

encode_impl!(u8, i16, u16, u32, i32, u64, f32);

mod leb128 {
    #[inline]
    pub fn encode(out: &mut Vec<u8>, mut value: u32) {
        loop {
            if value < 0x80 {
                out.push(value as u8);
                break;
            } else {
                out.push(((value & 0x7f) | 0x80) as u8);
                value >>= 7;
            }
        }
    }

    #[inline]
    pub fn decode(slice: &[u8]) -> (u32, usize) {
        let mut result = 0;
        let mut shift = 0;
        let mut position = 0;
        loop {
            let byte = slice[position];
            position += 1;
            if (byte & 0x80) == 0 {
                result |= (byte as u32) << shift;
                return (result, position);
            } else {
                result |= ((byte & 0x7F) as u32) << shift;
            }
            shift += 7;
        }
    }
}

impl OsuEncode for str {
    fn encoded_size(&self) -> usize { 2 + self.as_bytes().len() }

    fn encode(&self, buf: &mut Vec<u8>) {
        if self.is_empty() {
            buf.push(0);
        } else {
            buf.push(0xb);
            leb128::encode(buf, self.len() as u32);
            buf.extend_from_slice(self.as_bytes());
        }
    }

    fn decode(buf: &[u8]) -> Result<(&str, usize), ()> {
        let prefix = unsafe { *buf.as_ptr() };
        match prefix {
            0 => Ok(("", 1)),
            0xb => {
                let buf = &buf[1..];
                let (len, off) = leb128::decode(buf);
                let len = len as usize;
                if buf.len() < off + len {
                    return Err(());
                }
                let buf = &buf[off..off + len];
                let res = unsafe { std::str::from_utf8_unchecked(buf) };
                Ok((res, len + off + 1))
            }
            _ => {
                error!(?buf, "unknown string prefix");
                Err(())
            }
        }
    }
}

impl OsuEncode for bool {
    fn encoded_size(&self) -> usize { 1 }

    fn encode(&self, buf: &mut Vec<u8>) {
        if *self {
            buf.push(1);
        } else {
            buf.push(0);
        }
    }

    fn decode<'a>(buf: &'a [u8]) -> Result<(&'a Self, usize), ()> {
        let (&val, off) = u8::decode(buf)?;
        Ok((if val == 0 { &false } else { &true }, off))
    }
}

impl OsuEncode for [i32] {
    fn encoded_size(&self) -> usize { self.len() * 4 + 2 }

    // EXTREMELY unsafe, make sure the buffer has enough capacity to hold the data
    fn encode(&self, buf: &mut Vec<u8>) {
        //debug_assert!(buf.capacity() - buf.len() >= self.encoded_size());
        (self.len() as u16).encode(buf);
        // unsafe {
        //     let src = self.as_ptr() as *const u8;
        //     let dst = buf.as_mut_ptr().add(buf.len());
        //     std::ptr::copy_nonoverlapping(src, dst, self.len() * 4);
        //     buf.set_len(buf.len() + self.len() * 4);
        // }
        let src = unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 4) };
        buf.extend_from_slice(src);
    }

    fn decode(buf: &[u8]) -> Result<(&[i32], usize), ()> {
        let (&len, off) = u16::decode(buf)?;
        let len = len as usize;
        if buf.len() < off + len * 4 {
            return Err(());
        }
        #[allow(clippy::cast_ptr_alignment)]
        let data = unsafe { std::slice::from_raw_parts(buf.as_ptr().add(off) as *const i32, len) };
        let off = off + len * 4;
        Ok((data, off))
    }
}

impl OsuEncode for [u8] {
    fn encoded_size(&self) -> usize { self.len() }

    fn encode(&self, buf: &mut Vec<u8>) { buf.extend_from_slice(self); }

    fn decode(_buf: &[u8]) -> Result<(&Self, usize), ()> { unimplemented!() }
}

impl<T: OsuEncode, const N: usize> OsuEncode for [T; N] {
    fn encoded_size(&self) -> usize { self.len() * std::mem::size_of::<T>() }

    fn encode(&self, buf: &mut Vec<u8>) {
        let data = unsafe {
            std::slice::from_raw_parts(self.as_ptr() as *const u8, std::mem::size_of_val(self))
        };
        buf.extend_from_slice(data);
    }

    fn decode(buf: &[u8]) -> Result<(&Self, usize), ()> {
        if buf.len() < N {
            return Err(());
        }
        let ret = unsafe { &*(buf.as_ptr() as *const [T; N]) };
        Ok((ret, std::mem::size_of_val(ret)))
    }
}
