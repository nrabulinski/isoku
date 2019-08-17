pub struct Cursor<'a> {
    pos: usize,
    data: &'a [u8],
}

type Result<T> = std::result::Result<T, ()>;

type Buffer = Vec<u8>;

pub trait AsBuf: Sized {
    fn encode(self, buf: &mut Buffer);
    fn decode(buf: &mut Cursor) -> Result<Self>;
    fn size(&self) -> usize;
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Cursor<'a> {
        Cursor { pos: 0, data }
    }

    pub fn data(&self) -> &'a [u8] {
        if self.pos >= self.data.len() {
            &[]
        } else {
            &(self.data)[self.pos..]
        }
    }

    pub fn remaining(&self) -> usize {
        if self.pos >= self.data.len() {
            0
        } else {
            self.data.len() - self.pos
        }
    }

    pub fn read(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.pos >= self.data.len() {
            return Err(());
        }
        let res = &(self.data)[self.pos..self.pos + len];
        self.pos += len;
        Ok(res)
    }

    pub fn advance(&mut self, len: usize) {
        self.pos += len;
        if self.pos >= self.data.len() {
            warn!("advanced the cursor beyond data length");
        }
    }

    pub fn get<T: AsBuf>(&mut self) -> Result<T> {
        T::decode(self)
    }
}

impl AsBuf for u8 {
    fn encode(self, buf: &mut Buffer) {
        buf.push(self);
    }

    fn decode(buf: &mut Cursor) -> Result<Self> {
        Ok(buf.read(1)?[0])
    }

    fn size(&self) -> usize {
        1
    }
}

impl AsBuf for Vec<u32> {
    fn encode(self, buf: &mut Buffer) {
        let slice = unsafe {
            let data = self.as_ptr() as *const u8;
            std::slice::from_raw_parts(data, self.len() * 4)
        };
        buf.put(self.len() as u16);
        buf.extend_from_slice(slice);
    }

    fn decode(buf: &mut Cursor) -> Result<Self> {
        let len = buf.get::<u16>()? as usize;
        #[allow(clippy::cast_ptr_alignment)]
        let data = buf.read(len * 4)?.as_ptr() as *const u32;
        unsafe { Ok(std::slice::from_raw_parts(data, len).to_vec()) }
    }

    fn size(&self) -> usize {
        self.len() * 4 + 2
    }
}

impl AsBuf for &[i32] {
    fn encode(self, buf: &mut Buffer) {
        let slice = unsafe {
            let data = self.as_ptr() as *const u8;
            std::slice::from_raw_parts(data, self.len() * 4)
        };
        buf.put(self.len() as u16);
        buf.extend_from_slice(slice);
    }

    fn decode(buf: &mut Cursor) -> Result<Self> {
        let len = buf.get::<u16>()? as usize;
        #[allow(clippy::cast_ptr_alignment)]
        let data = buf.read(len * 4)?.as_ptr() as *const i32;
        unsafe { Ok(std::slice::from_raw_parts(data, len)) }
    }

    fn size(&self) -> usize {
        self.len() * 4 + 2
    }
}

impl AsBuf for Vec<u8> {
    fn encode(self, buf: &mut Buffer) {
        buf.extend_from_slice(&self)
    }

    fn decode(buf: &mut Cursor) -> Result<Self> {
        Ok(buf.read(buf.remaining())?.to_vec())
    }

    fn size(&self) -> usize {
        self.len()
    }
}

impl AsBuf for &[u8] {
    fn encode(self, buf: &mut Buffer) {
        buf.extend_from_slice(self)
    }

    fn decode(buf: &mut Cursor) -> Result<Self> {
        unsafe {
            let data = buf.read(buf.remaining())?;
            Ok(std::slice::from_raw_parts(data.as_ptr(), data.len()))
        }
    }

    fn size(&self) -> usize {
        self.len()
    }
}

impl AsBuf for () {
    fn encode(self, _: &mut Buffer) {}
    fn decode(_: &mut Cursor) -> Result<Self> {
        Ok(())
    }
    fn size(&self) -> usize {
        0
    }
}

mod leb {
    use super::{Bytes, Cursor};

    pub fn encode(buf: &mut Vec<u8>, value: u32) {
        let mut value = value;
        while {
            let mut byte = (value & !(1 << 7)) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 1 << 7;
            }
            buf.put(byte);
            value != 0
        } {}
    }

    pub fn decode(buf: &mut Cursor<'_>) -> u32 {
        let mut result = 0u32;
        let mut len = 0usize;
        for byte in buf.data() {
            result |= u32::from(byte & !(1 << 7)) << (len * 7);
            len += 1;
            if byte & 1 << 7 == 0 {
                break;
            }
        }
        buf.advance(len);
        result
    }
}

impl AsBuf for String {
    fn encode(self, buf: &mut Buffer) {
        if !self.is_empty() {
            buf.push(0xb);
            leb::encode(buf, self.len() as u32);
            buf.extend_from_slice(self.as_bytes());
        } else {
            buf.push(0)
        }
    }

    fn decode(buf: &mut Cursor) -> Result<Self> {
        let prefix: u8 = buf.get()?;
        match prefix {
            0 => Ok("".to_string()),
            0xb => {
                let len = leb::decode(buf) as usize;
                Ok(String::from_utf8_lossy(buf.read(len)?).to_string())
            }
            _ => {
                warn!("unknown string prefix {}; {:x?}", prefix, buf.data());
                Err(())
            }
        }
    }

    fn size(&self) -> usize {
        use std::convert::TryInto;
        let mut result = self.as_bytes().len();
        if !self.is_empty() {
            let mut len: u32 = self.as_bytes().len().try_into().unwrap();
            while len != 0 {
                len >>= 7;
                result += 1;
            }
        }
        result + 1
    }
}

macro_rules! auto_asbuf {
    ($($type:ty),+) => {
        use std::mem::size_of;
        $(
            impl AsBuf for $type {
                fn encode(self, buf: &mut Buffer) {
                    let slice = unsafe {
                        let data = &self as *const _ as *const u8;
                        std::slice::from_raw_parts(data, size_of::<$type>())
                    };
                    buf.extend_from_slice(slice);
                }

                fn decode(buf: &mut Cursor) -> Result<Self> {
                    let src = buf.read(size_of::<$type>())?.as_ptr() as *const $type;
                    unsafe {
                        Ok(*src)
                    }
                }

                fn size(&self) -> usize {
                    size_of::<$type>()
                }
            }
        )+
    };
}

auto_asbuf!(i16, u16, i32, u32, u64, f32);

pub trait Bytes {
    fn put(&mut self, data: impl AsBuf);
}

impl Bytes for Buffer {
    fn put(&mut self, data: impl AsBuf) {
        data.encode(self);
    }
}

#[cfg(test)]
mod tests {
    use super::Cursor;

    #[test]
    fn leb() {
        use super::leb;
        use rand::Rng;

        let mut buf = Vec::new();
        let value: u32 = {
            let mut rng = rand::thread_rng();
            rng.gen()
        };
        leb::encode(&mut buf, value);
        let mut c = Cursor::new(&buf);
        let decoded = leb::decode(&mut c);
        assert_eq!(decoded, value);
    }
}
