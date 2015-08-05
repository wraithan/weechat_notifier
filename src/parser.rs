use std::io::Cursor;
use std::io::prelude::*;
use byteorder::{ReadBytesExt, BigEndian, Error};
use flate2::read::ZlibDecoder;

pub fn get_length (buffer: &[u8]) -> Result<u32, Error> {
    let mut datum = Cursor::new(buffer);
    datum.read_u32::<BigEndian>()
}

pub fn get_compression (buffer: &[u8]) -> Result<bool, String> {
    if let Some(flag) = buffer.get(4) {
        Ok(flag == &1)
    } else {
        Err("buffer too short".to_owned())
    }
}

pub fn get_raw_data (buffer: &[u8]) -> Result<Vec<u8>, String> {
    let mut datum = Cursor::new(buffer);
    datum.set_position(5);
    let mut decoder = ZlibDecoder::new(datum);
    let mut result = Vec::<u8>::new();
    match decoder.read_to_end(&mut result) {
        Ok(_) => Ok(result),
        Err(_) => Err("decoding bummer".to_owned())
    }
}

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

#[test]
fn parse_test_data() {
    // Data as returned by the test command in weechat:
    let data =  [0, 0, 0, 145, 1, 120, 156, 251, 255, 255, 255, 255, 228, 140,
                 34, 199, 204, 188, 18, 6, 198, 71, 14, 64, 234, 255, 63, 217,
                 3, 57, 249, 121, 92, 134, 70, 198, 38, 166, 102, 230, 22, 150,
                 6, 64, 30, 183, 46, 130, 91, 92, 82, 196, 192, 192, 192, 145,
                 168, 0, 100, 100, 230, 165, 67, 184, 12, 64, 10, 104, 212, 255,
                 164, 210, 52, 32, 135, 13, 72, 165, 165, 22, 1, 73, 144, 88,
                 65, 73, 17, 7, 72, 123, 98, 82, 114, 10, 144, 205, 104, 80, 146,
                 153, 203, 101, 104, 108, 100, 104, 105, 9, 50, 51, 177, 168, 8,
                 98, 6, 19, 16, 51, 3, 21, 129, 152, 41, 169, 64, 97, 144, 163,
                 128, 66, 64, 92, 205, 192, 192, 120, 2, 200, 20, 5, 0, 59, 212,
                 56, 52];
    assert!(get_length(&data).unwrap() == 145);
    assert!(get_compression(&data).unwrap() == true);
    println_stderr!("got data: {:?}", get_raw_data(&data).unwrap())
}
