extern crate weechat_parser;

use std::io::prelude::*;
use std::fs::File;

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

    weechat_parser::WeechatMessage::from_raw_message(&buffer[..length]).unwrap();
}
