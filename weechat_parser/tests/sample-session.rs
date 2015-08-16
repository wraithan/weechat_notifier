extern crate weechat_parser;

use std::io::prelude::*;
use std::fs::File;
use weechat_parser::{WeechatData, WeechatMessage};

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        if let Err(x) = writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            panic!("Unable to write to stderr: {}", x);
        }
    )
);

#[test]
fn simple_session_test () {
    // Load in session data
    let mut f = File::open("./tests/fodder/simple.dat").unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();

    let length = weechat_parser::get_length(&buffer).unwrap() as usize;
    println_stderr!("got length of {}", length);

    let message = WeechatMessage::from_raw_message(&buffer[..length]).unwrap();
    assert_eq!(message.id, "_buffer_line_added");
    let datum = message.data.get(0).unwrap();
    match datum {
        &WeechatData::Hdata(ref name, ref data) => {
            assert_eq!(name, "line_data");
            assert_eq!(data.len(), 1);
            let hdata = data.get(0).unwrap();
            assert_eq!(hdata.get("buffer").unwrap(), &WeechatData::Pointer("0x7fcab15936d0".to_owned()));
            assert_eq!(hdata.get("date").unwrap(), &WeechatData::Time("1439651878".to_owned()));
            assert_eq!(hdata.get("date_printed").unwrap(), &WeechatData::Time("1439651878".to_owned()));
            assert_eq!(hdata.get("displayed").unwrap(), &WeechatData::Char('\u{1}'));
            assert_eq!(hdata.get("highlight").unwrap(), &WeechatData::Char('\u{0}'));
        },
        _ => panic!("unexpected type: {:?}", datum)
    }
}
