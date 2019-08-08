use isoku::osu::OsuData;
use rand::Rng;
use rand::distributions::{Distribution, Standard};
use std::mem::size_of;
use isoku::cursor::Cursor;
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::clone::Clone;

fn basic_test<T: OsuData + PartialEq + Debug + Clone>(value: T) {
    let encoded = value.clone().encode();
    let mut cursor = Cursor::new(&encoded);
    let decoded = <T as OsuData>::decode(&mut cursor);
    assert_eq!(decoded, value);
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
    let encoded = value.encode();
    let mut cursor = Cursor::new(&encoded);
    let decoded: &[i32] = OsuData::decode(&mut cursor);
    assert_eq!(encoded.len(), value.len() * size_of::<i32>() + size_of::<u16>());
    assert_eq!(decoded, &value);
}

#[test]
fn multiple_values() {
    type Test = (i32, u32, i16, u16, String);
    let (a,b,c,d,e): Test = {
        let mut rng = rand::thread_rng();
        (rng.gen(),rng.gen(),rng.gen(),rng.gen(),"osu test".to_string())
    };
        let encoded = [
        a.encode(),
        b.encode(),
        c.encode(),
        d.encode(),
        e.clone().encode()
    ].concat();
    let mut cursor = Cursor::new(&encoded);
    let (x,y,z,i,j): Test = (
        OsuData::decode(&mut cursor),
        OsuData::decode(&mut cursor),
        OsuData::decode(&mut cursor),
        OsuData::decode(&mut cursor),
        OsuData::decode(&mut cursor),
    );
    assert_eq!(x, a);
    assert_eq!(y, b);
    assert_eq!(z, c);
    assert_eq!(i, d);
    assert_eq!(j, e);
}


fn rand<T>() -> T
where Standard: Distribution<T> {
    let mut rng = rand::thread_rng();
    rng.gen()
}