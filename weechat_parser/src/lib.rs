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
        if let Err(x) = writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            panic!("Unable to write to stderr: {}", x);
        }
    )
);

pub struct WeechatMessage {
    pub id: String,
    pub data: Vec<WeechatData>
}


#[derive(PartialEq, Eq, Clone, Debug)]
pub enum WeechatData {
    Char(char),
    Int(i32),
    Long(i64),
    String(String),
    StringNull,
    Buffer(String),
    BufferNull,
    Pointer(String),
    Time(String),
    Array(Vec<WeechatData>)
}

impl WeechatMessage {
    pub fn from_raw_message (buffer: &[u8]) -> Result<WeechatMessage, WeechatParseError> {
        let raw_data = try!(get_raw_data(&buffer));
        let (len, id) = try!(get_message_type(&raw_data));
        let length = raw_data.len() - len;
        let name;
        let data = match id {
            None => {
                name = "test".to_owned();
                try!(parse_test_data(&raw_data[len..], length))
            },
            Some(message_type) => fail!((UnknownId, "Got an unfamiliar ID", message_type))
        };
        Ok(WeechatMessage{id: name, data: data})
    }
}

fn parse_test_data (buffer: &[u8], length: usize) -> Result<Vec<WeechatData>, WeechatParseError> {
    let mut acc = vec![];
    let mut position = 0;
    while position < length {
        let element_type = get_element_type(&buffer[position..]);
        position += 3;
        match element_type.as_str() {
            "chr" => {
                let value = try!(read_u8(&buffer[position..]));
                let input_char = try!(char::from_u32(value as u32).ok_or((MalformedBinaryParse, "Couldn't read char data")));
                acc.push(WeechatData::Char(input_char));
                position += 1;
            },
            "int" => {
                acc.push(WeechatData::Int(try!(read_i32(&buffer[position..]))));
                position += 4;
            },
            "lon" => {
                let (len, value) = try!(read_long(&buffer[position..]));
                acc.push(WeechatData::Long(value));
                position += len;
            },
            "str" => {
                let (len, value) = try!(read_string_32bit_length(&buffer[position..]));
                match value {
                    Some(string) => acc.push(WeechatData::String(string)),
                    None => acc.push(WeechatData::StringNull)
                }
                position += len;
            },
            "buf" => {
                let (len, value) = try!(read_string_32bit_length(&buffer[position..]));
                match value {
                    Some(string) => acc.push(WeechatData::Buffer(string)),
                    None => acc.push(WeechatData::BufferNull)
                }
                position += len;
            },
            "ptr" => {
                let (len, value) = try!(read_pointer(&buffer[position..]));
                acc.push(WeechatData::Pointer(value));
                position += len;
            },
            "tim" => {
                let (len, value) = try!(read_time(&buffer[position..]));
                acc.push(WeechatData::Time(value));
                position += len;
            },
            "htb" => break,
            "hda" => break,
            "inf" => break,
            "inl" => break,
            "arr" => {
                let (len, value) = try!(read_array(&buffer[position..]));
                acc.push(WeechatData::Array(value));
                position += len;
            },
            _ => fail!((UnknownType, "Got unfamiliar type", element_type.to_owned()))
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

fn read_long(buffer: &[u8]) -> Result<(usize, i64), WeechatParseError> {
    let (end, value) = try!(read_string_8bit_length(&buffer));
    let long = try!(i64::from_str_radix(value.as_str(), 10));
    Ok((end, long))
}

fn read_pointer(buffer: &[u8]) -> Result<(usize, String), WeechatParseError> {
    let (end, mut value) = try!(read_string_8bit_length(&buffer));
    // Pointers should have 0x at the start.
    value.insert(0, '0');
    value.insert(1, 'x');
    Ok((end, value))
}

fn read_time(buffer: &[u8]) -> Result<(usize, String), WeechatParseError> {
    read_string_8bit_length(&buffer)
}

fn read_string_8bit_length(buffer: &[u8]) -> Result<(usize, String), WeechatParseError> {
    let length = try!(read_u8(&buffer)) as usize;
    let end = length + 1;
    let value = String::from_utf8_lossy(&buffer[1..end]).into_owned();
    Ok((end, value))
}

fn read_string_32bit_length(buffer: &[u8]) -> Result<(usize, Option<String>), WeechatParseError> {
    match read_i32(buffer) {
        Ok(size) => {
            if size == 0 {
                return Ok((4, Some("".to_owned())))
            }
            if size == -1  {
                return Ok((4, None))
            }
            let end = (size + 4) as usize;
            let raw_string = &buffer[4..end];
            let value = String::from_utf8_lossy(raw_string);
            Ok((end, Some(value.into_owned())))
        },
        Err(error) => fail!(error)
    }
}

fn read_array(buffer: &[u8]) -> Result<(usize, Vec<WeechatData>), WeechatParseError> {
    let array_type = get_element_type(&buffer);
    let mut position = 3;
    let count = try!(read_i32(&buffer[position..]));
    position += 4;
    let mut acc = Vec::<WeechatData>::with_capacity(count as usize);
    match array_type.as_str() {
        "str" => {
            for _ in 0..count {
                let (len, value) = try!(read_string_32bit_length(&buffer[position..]));
                match value {
                    Some(string) => acc.push(WeechatData::String(string)),
                    None => acc.push(WeechatData::StringNull)
                }
                position += len;
            }
        }
        "int" => {
            for _ in 0..count {
                let value = try!(read_i32(&buffer[position..]));
                acc.push(WeechatData::Int(value));
                position += 4;
            }
        }
        _ => fail!((UnknownType, "array isn't implemented for type", format!("found array type {:?}", array_type)))
    };
    Ok((position, acc))
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

pub fn get_message_type (buffer: &[u8]) -> Result<(usize, Option<String>), WeechatParseError> {
    read_string_32bit_length(&buffer)
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
    assert_eq!(message.id, "test".to_owned());
    assert_eq!(message.data.get(0), Some(&WeechatData::Char('A')));
    assert_eq!(message.data.get(1), Some(&WeechatData::Int(123456)));
    assert_eq!(message.data.get(2), Some(&WeechatData::Int(-123456)));
    assert_eq!(message.data.get(3), Some(&WeechatData::Long(1234567890)));
    assert_eq!(message.data.get(4), Some(&WeechatData::Long(-1234567890)));
    assert_eq!(message.data.get(5), Some(&WeechatData::String("a string".to_owned())));
    assert_eq!(message.data.get(6), Some(&WeechatData::String("".to_owned())));
    assert_eq!(message.data.get(7), Some(&WeechatData::StringNull));
    assert_eq!(message.data.get(8), Some(&WeechatData::Buffer("buffer".to_owned())));
    assert_eq!(message.data.get(9), Some(&WeechatData::BufferNull));
    assert_eq!(message.data.get(10), Some(&WeechatData::Pointer("0x1234abcd".to_owned())));
    assert_eq!(message.data.get(11), Some(&WeechatData::Pointer("0x0".to_owned())));
    assert_eq!(message.data.get(12), Some(&WeechatData::Time("1321993456".to_owned())));
    if let &WeechatData::Array(ref test_string_array) = message.data.get(13).unwrap() {
        assert_eq!(test_string_array.len(), 2);
        assert_eq!(test_string_array.get(0), Some(&WeechatData::String("abc".to_owned())));
        assert_eq!(test_string_array.get(1), Some(&WeechatData::String("de".to_owned())));
    } else {
        panic!("got wrong type in test element 13 (expected Array)");
    }
    if let &WeechatData::Array(ref test_string_array) = message.data.get(14).unwrap() {
        assert_eq!(test_string_array.len(), 3);
        assert_eq!(test_string_array.get(0), Some(&WeechatData::Int(123)));
        assert_eq!(test_string_array.get(1), Some(&WeechatData::Int(456)));
        assert_eq!(test_string_array.get(2), Some(&WeechatData::Int(789)));
    } else {
        panic!("got wrong type in test element 14 (expected Array)");
    }
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
    assert_eq!(message_type, None);
    assert_eq!(get_element_type(&raw_data[type_jump..]), "chr".to_owned());
}
