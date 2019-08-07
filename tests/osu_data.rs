use isoku::osu::OsuData;
use rand::Rng;
use rand::distributions::{Distribution, Standard};
use std::mem::size_of;

#[test]
fn string() {
    let value = "#osu";
    let encoded = value.to_string().encode();
    let (_, decoded) = String::decode(&encoded);
    println!("{:x?}", encoded.as_ref());
    assert_eq!(&decoded, value);
}

#[test]
fn u16() {
    let value: u16 = rand();
    let encoded = value.encode();
    let (_, decoded) = u16::decode(&encoded);
    assert_eq!(decoded, value);
}

#[test]
fn i16() {
    let value: i16 = rand();
    let encoded = value.encode();
    let (_, decoded) = i16::decode(&encoded);
    assert_eq!(decoded, value);
}

#[test]
fn u32() {
    let value: u32 = rand();
    let encoded = value.encode();
    let (_, decoded) = u32::decode(&encoded);
    assert_eq!(decoded, value);
}

#[test]
fn i32() {
    let value: i32 = rand();
    let encoded = value.encode();
    let (_, decoded) = i32::decode(&encoded);
    assert_eq!(decoded, value);
}

#[test]
fn i32_slice() {
    let value = [1_i32, 1, 1, 1, 1, 1];
    let encoded = value.encode();
    let (_, decoded): (_, &[i32]) = OsuData::decode(&encoded);
    println!("{:?}\n{:?}\n{:?}\n", value, encoded.as_ref(), decoded);
    assert_eq!(encoded.len(), value.len() * size_of::<i32>() + size_of::<u16>());
    assert_eq!(decoded, &value);
}


fn rand<T>() -> T
where Standard: Distribution<T> {
    let mut rng = rand::thread_rng();
    rng.gen()
}