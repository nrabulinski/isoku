use isoku::packets::encoding::OsuEncode;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::fmt::Debug;

fn basic_test<T>(value: &T)
where
    T: OsuEncode + PartialEq + Debug + ?Sized,
{
    let mut buf = Vec::with_capacity(value.encoded_size());
    value.encode(&mut buf);
    let (decoded, _) = T::decode(&buf).unwrap();
    assert_eq!(value, decoded);
}

fn rand<T>() -> T
where
    Standard: Distribution<T>,
{
    let mut rng = rand::thread_rng();
    rng.gen()
}

#[test]
fn string() { basic_test("#osu") }

#[test]
fn u16() { basic_test(&rand::<u16>()); }

#[test]
fn i16() { basic_test(&rand::<i16>()); }

#[test]
fn u32() { basic_test(&rand::<u32>()); }

#[test]
fn i32() { basic_test(&rand::<i32>()); }
