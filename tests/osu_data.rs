//use isoku::osu::OsuData;
use isoku::bytes::{AsBuf, Bytes, Cursor};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::clone::Clone;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::mem::size_of;

fn basic_test<T: AsBuf + PartialEq + Debug + Clone>(value: T) {
    let mut buf = Vec::new();
    buf.put(value.clone());
    let mut cursor = Cursor::new(&buf);
    let decoded: T = cursor.get().unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn str_len() {
    let strings = [
        "a very very very very long string",
        "looooooooong string",
        "osu test",
        "osu",
        "",
    ];
    let mut buf = Vec::new();
    for s in strings.into_iter() {
        buf.clear();
        let value = s.to_string();
        buf.put(value.clone());
        assert_eq!(value.size(), buf.len());
    }
}

#[test]
fn string() {
    basic_test("#osu".to_string());
}

#[test]
fn u16() {
    basic_test(rand::<u16>());
}

#[test]
fn i16() {
    basic_test(rand::<i16>());
}

#[test]
fn u32() {
    basic_test(rand::<u32>());
}

#[test]
fn i32() {
    basic_test(rand::<i32>());
}

#[test]
fn i32_slice() {
    let value = [1_i32];
    let mut buf = Vec::new();
    buf.put(value.as_ref());
    let mut cursor = Cursor::new(&buf);
    let decoded: &[i32] = cursor.get().unwrap();
    assert_eq!(buf.len(), value.len() * size_of::<i32>() + size_of::<u16>());
    assert_eq!(decoded, &value);
}

#[test]
fn multiple_values() {
    type Test = (i32, u32, i16, u16, String);
    let mut buf = Vec::new();
    let (a, b, c, d, e): Test = {
        let mut rng = rand::thread_rng();
        (
            rng.gen(),
            rng.gen(),
            rng.gen(),
            rng.gen(),
            "osu test".to_string(),
        )
    };
    buf.put(a);
    buf.put(b);
    buf.put(c);
    buf.put(d);
    buf.put(e.clone());
    let mut cursor = Cursor::new(&buf);
    let (x, y, z, i, j): Test = (
        cursor.get().unwrap(),
        cursor.get().unwrap(),
        cursor.get().unwrap(),
        cursor.get().unwrap(),
        cursor.get().unwrap(),
    );
    assert_eq!(x, a);
    assert_eq!(y, b);
    assert_eq!(z, c);
    assert_eq!(i, d);
    assert_eq!(j, e);
}

fn rand<T>() -> T
where
    Standard: Distribution<T>,
{
    let mut rng = rand::thread_rng();
    rng.gen()
}
