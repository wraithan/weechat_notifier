extern crate byteorder;
extern crate flate2;

#[macro_use]
pub mod errors;

use std::io::Cursor;
use std::io::prelude::*;
use std::string::String;
use byteorder::{ReadBytesExt, BigEndian};
use flate2::read::ZlibDecoder;
use errors::WeechatParseError;

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

pub fn read_u32(buffer: &[u8]) -> Result<u32, WeechatParseError> {
    let mut datum = Cursor::new(buffer);
    match datum.read_u32::<BigEndian>() {
        Ok(value) => Ok(value),
        Err(error) => fail!(error)
    }
}

pub fn read_i32(buffer: &[u8]) -> Result<i32, WeechatParseError> {
    let mut datum = Cursor::new(buffer);
    match datum.read_i32::<BigEndian>() {
        Ok(value) => Ok(value),
        Err(error) => fail!(error)
    }
}

pub fn read_string(buffer: &[u8]) -> Result<String, WeechatParseError> {
    match read_u32(buffer) {
        Ok(size) => {
            println_stderr!("size {}", size);
            println_stderr!("const {}", 0xFFFFFFFF as u32);
            println_stderr!("raw {:?}", &buffer[1..5]);
            if size == 0xFFFFFFFF as u32 {
                return Ok("".to_owned())
            }
            let length = size as usize;
            let raw_string = &buffer[5..length];
            let value = String::from_utf8_lossy(raw_string);
            Ok(value.into_owned())
        },
        Err(error) => fail!(error)
    }
}

pub fn get_length (buffer: &[u8]) -> Result<u32, WeechatParseError> {
    read_u32(buffer)
}

pub fn get_compression (buffer: &[u8]) -> Result<bool, String> {
    if let Some(flag) = buffer.get(4) {
        Ok(flag == &1)
    } else {
        Err("buffer too short".to_owned())
    }
}

pub fn get_message_type (buffer: &[u8]) -> Result<String, WeechatParseError> {
    read_string(&buffer)
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

    // uncompressed data blob.
    // [255, 255, 255, 255, 99, 104, 114, 65, 105, 110, 116, 0, 1, 226, 64, 105,
    //  110, 116, 255, 254, 29, 192, 108, 111, 110, 10, 49, 50, 51, 52, 53, 54,
    //  55, 56, 57, 48, 108, 111, 110, 11, 45, 49, 50, 51, 52, 53, 54, 55, 56, 57,
    //  48, 115, 116, 114, 0, 0, 0, 8, 97, 32, 115, 116, 114, 105, 110, 103, 115,
    //  116, 114, 0, 0, 0, 0, 115, 116, 114, 255, 255, 255, 255, 98, 117, 102, 0,
    //  0, 0, 6, 98, 117, 102, 102, 101, 114, 98, 117, 102, 255, 255, 255, 255, 112,
    //  116, 114, 8, 49, 50, 51, 52, 97, 98, 99, 100, 112, 116, 114, 1, 48, 116,
    //  105, 109, 10, 49, 51, 50, 49, 57, 57, 51, 52, 53, 54, 97, 114, 114, 115,
    //  116, 114, 0, 0, 0, 2, 0, 0, 0, 3, 97, 98, 99, 0, 0, 0, 2, 100, 101, 97, 114,
    //  114, 105, 110, 116, 0, 0, 0, 3, 0, 0, 0, 123, 0, 0, 1, 200, 0, 0, 3, 21]
    assert_eq!(get_length(&data).unwrap(), 145);
    assert_eq!(get_compression(&data).unwrap(), true);
    let raw_data = get_raw_data(&data).unwrap();
    assert_eq!(get_message_type(&raw_data).unwrap(), "".to_owned());
}
