use bytes::{BufMut, BytesMut};
use std::sync::{RwLock, Arc};
use std::collections::HashMap;

pub mod packets;
pub mod token;
pub mod channel;
//pub use token::TokenList;

#[derive(Debug)]
pub struct List<T> {
    list: RwLock<HashMap<String, Arc<T>>>
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            list: RwLock::new(HashMap::new())
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<T>> {
        match self.list.read().unwrap().get(key) {
            Some(val) => Some((*val).clone()),
            None => None
        }
    }

    pub fn entries(&self) -> Vec<Arc<T>> {
        self.list.read().unwrap().values().map(|t| t.clone()).collect()
    }

    pub fn remove(&self, key: &str) -> Option<Arc<T>> {
        self.list.write().unwrap().remove(key)
    }

    fn insert(&self, key: String, val: Arc<T>) {
        self.list.write().unwrap().insert(key, val);
    }
}

pub trait OsuData {
    fn encode(self) -> BytesMut;
    //TODO: Make the return type Self again
    fn decode(buf: &[u8]) -> (usize, Self);
}

impl OsuData for u16 {
    fn encode(self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u16_le(self);
        buf
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        unsafe {
            (2, *(buf.as_ptr() as *mut u16))
        }
    }
}

impl OsuData for i16 {
    fn encode(self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_i16_le(self);
        buf
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        unsafe {
            (2, *(buf.as_ptr() as *mut i16))
        }
    }
}

impl OsuData for u32 {
    fn encode(self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_u32_le(self);
        buf
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        unsafe {
            (4, *(buf.as_ptr() as *mut u32))
        }
    }
}

impl OsuData for i32 {
    fn encode(self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_i32_le(self);
        buf
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        unsafe {
            (4, *(buf.as_ptr() as *mut i32))
        }
    }
}

impl OsuData for BytesMut {
    fn encode(self) -> BytesMut {
        self
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        (buf.len(), BytesMut::from(buf))
    }
}

impl OsuData for Vec<i32> {
    fn encode(self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(self.len() + 2);
        buf.put_u16_le(self.len() as u16);
        // if self.len() > 0 {
        //     unsafe {
        //         buf.set_len(self.len() + 2);
        //         std::ptr::copy(self.as_ptr(), buf.as_mut_ptr().offset(2) as *mut i32, self.len())
        //     };
        // };
        for val in self {
            buf.put_i32_le(val);
        }
        buf
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        use bytes::Buf;
        use std::io::Cursor;
        let mut buf = Cursor::new(buf);
        let len = buf.get_i16_le();//i16::decode(buf);
        if len > 0 {
            // let ptr = (&buf[2..]).as_ptr() as *const i32;
            // let mut data = Vec::with_capacity(len as usize);
            // unsafe {
            //     data.set_len(len as usize);
            //     std::ptr::copy(ptr, data.as_mut_ptr(), len as usize)
            // };
            // data
            let mut data = Vec::with_capacity(len as usize);
            for _ in 0..len {
                data.push(buf.get_i32_le());
            }
            (2 + data.len() * 4, data)
        } else {
            (2, Vec::with_capacity(0))
        }
    }
}

impl OsuData for &[i32] {
    fn encode(self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(self.len() * 4 + 2);
        buf.put_u16_le(self.len() as u16);
        unsafe {
            buf.set_len(self.len() * 4 + 2);
            let dst = buf.as_mut_ptr().offset(2) as *mut i32;
            std::ptr::copy_nonoverlapping(self.as_ptr(), dst, self.len());
        }
        buf
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        let (_, len) = i16::decode(buf);
        unsafe {
            let ptr = buf.as_ptr().offset(2) as *const i32;
            let slice = std::slice::from_raw_parts(ptr, len as usize);
            (2 + slice.len() * 4, slice)
        }
    }
}

impl OsuData for Vec<u8> {
    fn encode(self) -> BytesMut {
        BytesMut::from(self)
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        (buf.len(), Vec::from(buf))
    }
}

fn leb_encode(buf: &mut BytesMut, value: u32) {
    let mut value = value;
    while {
        let mut byte = (value & !(1<<7)) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 1<<7;
        }
        buf.put_u8(byte);
        value != 0
    } {}
}

fn leb_decode(buf: &[u8]) -> (u32, usize) {
    let mut result = 0u32;
    //let mut shift = 0usize;
    let mut len = 0usize;
    for byte in buf {
        result |= ((byte & !(1<<7)) as u32) << (len * 7);
        len += 1;
        if byte & 1<<7 == 0 {
            break
        }
        //shift += 7;
    }
    (result, len)
}

fn encode_str(text: &str) -> BytesMut {
    let mut buf = BytesMut::with_capacity(1);
    if text.len() > 0 { 
        buf.put_u8(11);
        let bytes = text.as_bytes();
        leb_encode(&mut buf, bytes.len() as u32);
        buf.reserve(bytes.len());
        buf.put(bytes); 
    } else { buf.put_u8(0) };
    buf
}

impl OsuData for String {
    fn encode(self) -> BytesMut {
        encode_str(&self)
    }

    fn decode(buf: &[u8]) -> (usize, Self) {
        if buf[0] == 0 {
            (1, "".to_string())
        } else if buf[0] == 0xb {
            let buf = &buf[1..];
            let (len, start) = leb_decode(buf);
            (1 + start + len as usize, String::from_utf8_lossy(&buf[start..start + len as usize]).to_string())
        } else {
            eprint!("Unknown string prefix!\n{:x?}", buf);
            (1, "".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    #[test]
    fn leb() {
        use super::{leb_decode, leb_encode};
        use rand::Rng;

        let mut buf = BytesMut::new();
        let value: u32 = { 
            let mut rng = rand::thread_rng();
            rng.gen()
        };
        leb_encode(&mut buf, value);
        let (decoded, _) = leb_decode(&buf);
        assert_eq!(decoded, value);
    }
}
