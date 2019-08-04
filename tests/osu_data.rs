use isoku::osu::OsuData;
use rand::Rng;
use rand::distributions::{Distribution, Standard};

#[test]
fn string() {
    let value = "test string";
    let encoded = value.to_string().encode();
    let decoded = String::decode(&encoded);
    assert_eq!(&decoded, value);
}

#[test]
fn u16() {
    let value: u16 = rand();
    let encoded = value.encode();
    let decoded = u16::decode(&encoded);
    assert_eq!(decoded, value);
}

#[test]
fn i16() {
    let value: i16 = rand();
    let encoded = value.encode();
    let decoded = i16::decode(&encoded);
    assert_eq!(decoded, value);
}

#[test]
fn u32() {
    let value: u32 = rand();
    let encoded = value.encode();
    let decoded = u32::decode(&encoded);
    assert_eq!(decoded, value);
}

#[test]
fn i32() {
    let value: i32 = rand();
    let encoded = value.encode();
    let decoded = i32::decode(&encoded);
    assert_eq!(decoded, value);
}


fn rand<T>() -> T
where Standard: Distribution<T> {
    let mut rng = rand::thread_rng();
    rng.gen()
}