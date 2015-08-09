#![feature(convert)]

extern crate byteorder;
extern crate flate2;

#[macro_use]
pub mod errors;

use std::char;
use std::io::Cursor;
use std::io::prelude::*;
use std::string::String;
use byteorder::{ReadBytesExt, BigEndian};
use flate2::read::ZlibDecoder;
use errors::WeechatParseError;
use errors::ErrorKind::{MalformedBinaryParse, UnknownId, UnknownType};

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);

pub struct WeechatMessage {
    pub id: String,
    pub data: Vec<WeechatData>
}


#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum WeechatData {
    Char(char),
    Int(i32)
}

impl WeechatMessage {
    pub fn from_raw_message (buffer: &[u8]) -> Result<WeechatMessage, WeechatParseError> {
        let raw_data = try!(get_raw_data(&buffer));
        let length = raw_data.len();
        let (len, id) = try!(get_message_type(&raw_data));
        let data = match id.as_str() {
            "" => try!(parse_test_data(&raw_data[len..], length)),
            _ => fail!((UnknownId, "Got an unfamiliar ID", id.to_owned()))
        };
        Ok(WeechatMessage{id: id, data: data})
    }
}

fn parse_test_data (buffer: &[u8], length: usize) -> Result<Vec<WeechatData>, WeechatParseError> {
    let mut acc = vec![];
    let mut position = 0;
    while position < length {
        let element_type = get_element_type(&buffer[position..]);
        println_stderr!("called");
        match element_type.as_str() {
            "chr" => {
                let value = try!(read_u8(&buffer[position + 3..]));
                let input_char = try!(char::from_u32(value as u32).ok_or((MalformedBinaryParse, "Couldn't read char data")));
                acc.push(WeechatData::Char(input_char));
                position += 4;
            },
            "int" => {
                acc.push(WeechatData::Int(try!(read_i32(&buffer[position + 3..]))));
                position += 7
            },
            _ => break//fail!((UnknownType, "Got unfamiliar type", element_type.to_owned()))
        }
    }
    Ok(acc)
}

fn get_element_type (buffer: &[u8]) -> String {
    String::from_utf8_lossy(&buffer[..3]).into_owned()
}

fn read_u32(buffer: &[u8]) -> Result<u32, WeechatParseError> {
    let mut datum = Cursor::new(buffer);
    try_result!(datum.read_u32::<BigEndian>())
}

fn read_u8(buffer: &[u8]) -> Result<u8, WeechatParseError> {
    let mut datum = Cursor::new(buffer);
    try_result!(datum.read_u8())
}

fn read_i32(buffer: &[u8]) -> Result<i32, WeechatParseError> {
    let mut datum = Cursor::new(buffer);
    try_result!(datum.read_i32::<BigEndian>())
}

fn read_string(buffer: &[u8]) -> Result<(usize, String), WeechatParseError> {
    match read_i32(buffer) {
        Ok(size) => {
            if size == -1 {
                return Ok((4, "".to_owned()))
            }
            let length = size as usize;
            let raw_string = &buffer[5..length];
            let value = String::from_utf8_lossy(raw_string);
            Ok((size as usize, value.into_owned()))
        },
        Err(error) => fail!(error)
    }
}

pub fn get_length (buffer: &[u8]) -> Result<u32, WeechatParseError> {
    read_u32(buffer)
}

pub fn get_compression (buffer: &[u8]) -> Result<bool, WeechatParseError> {
    if let Some(flag) = buffer.get(4) {
        Ok(flag == &1)
    } else {
        fail!((MalformedBinaryParse, "Could not find compression flag"))
    }
}

pub fn get_message_type (buffer: &[u8]) -> Result<(usize, String), WeechatParseError> {
    read_string(&buffer)
}

pub fn get_raw_data (buffer: &[u8]) -> Result<Vec<u8>, WeechatParseError> {
    let mut datum = Cursor::new(buffer);
    datum.set_position(5);
    let mut decoder = ZlibDecoder::new(datum);
    let mut result = Vec::<u8>::new();
    match decoder.read_to_end(&mut result) {
        Ok(_) => Ok(result),
        Err(error) => fail!(error)
    }
}

#[test]
fn test_parse_test_data() {
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

    let message = WeechatMessage::from_raw_message(&data).unwrap();
    assert_eq!(message.id, "".to_owned());
    assert_eq!(message.data.get(0), Some(&WeechatData::Char('A')));
    assert_eq!(message.data.get(1), Some(&WeechatData::Int(123456)));
    assert_eq!(message.data.get(2), Some(&WeechatData::Int(-123456)));
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
    let (type_jump, message_type) = get_message_type(&raw_data).unwrap();
    assert_eq!(type_jump, 4);
    assert_eq!(message_type, "".to_owned());
    assert_eq!(get_element_type(&raw_data[type_jump..]), "chr".to_owned());
}
