use isoku::packets::{server as p, Id, OsuEncode};
use std::convert::TryFrom;

fn packet_test(buf: &[u8], expected_id: Id, expected_data: &[u8]) {
    let (&id, off) = u16::decode(buf).unwrap();
    let id = Id::try_from(id).unwrap();
    let buf = &buf[off + 1..];
    let (&data_len, off) = i32::decode(buf).unwrap();
    assert_eq!(expected_id as u16, id as u16);
    assert_eq!(expected_data.len() as i32, data_len);
    assert_eq!(expected_data, &buf[off..]);
}

#[test]
fn silence_end() {
    let data = p::silence_end(0);
    packet_test(&data, Id::SilenceEnd, &[0, 0, 0, 0])
}
