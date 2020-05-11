use isoku::packets::OsuEncode;
use std::fmt::Debug;

fn basic_test<T>(value: &T, expected_buf: &[u8])
where
    T: OsuEncode + PartialEq + Debug + ?Sized,
{
    let mut buf = Vec::with_capacity(value.encoded_size());
    value.encode(&mut buf);
    assert_eq!(expected_buf, &buf[..]);
    let (decoded, _) = T::decode(&buf).unwrap();
    assert_eq!(value, decoded);
}

#[test]
fn string() { basic_test("#osu", &[0xb, 4, 0x23, 0x6f, 0x73, 0x75]) }

#[test]
fn u16() { basic_test(&2137u16, &[0x59, 0x8]); }

#[test]
fn i16() { basic_test(&666i16, &[0x9a, 0x2]); }

#[test]
fn u32() { basic_test(&9727u32, &[0xff, 0x25, 0, 0]); }

#[test]
fn i32() { basic_test(&1337i32, &[0x39, 0x5, 0, 0]); }

#[test]
#[should_panic]
fn u8_slice_decode() { <[u8] as OsuEncode>::decode(&[0u8]).unwrap(); }

#[test]
fn bool() { basic_test(&true, &[1]); }

#[test]
fn empty_string() { basic_test("", &[0]) }

#[test]
#[should_panic]
fn str_unknown_prefix() {
    let buf = [0xc, 0u8];
    <str as OsuEncode>::decode(&buf).unwrap();
}

#[test]
#[should_panic]
fn str_invalid_len() {
    let buf = [0xb, 2, 0u8];
    <str as OsuEncode>::decode(&buf).unwrap();
}

#[test]
fn i32_slice() {
    let val = [10i32, 2, 1, 3, 7, 0];
    basic_test(
        &val[..],
        &[
            6, 0, 10, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0,
        ],
    );
}
