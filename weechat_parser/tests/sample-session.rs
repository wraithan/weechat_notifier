extern crate weechat_parser;

use std::io::prelude::*;
use std::fs::File;
use std::sync::mpsc::Receiver;
use weechat_parser::{WeechatData, WeechatMessage};
use weechat_parser::errors::WeechatParseError;

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        if let Err(x) = writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            panic!("Unable to write to stderr: {}", x);
        }
    )
);

#[test]
fn simple_session_test() {
    // Load in session data
    let mut f = File::open("./tests/fodder/simple.dat").unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();

    let length = weechat_parser::get_length(&buffer).unwrap() as usize;

    let message = WeechatMessage::from_raw_message(&buffer[..length]).unwrap();
    assert_eq!(message.id, "_buffer_line_added");
    if let &WeechatData::Hdata(ref name, _, ref data) = message.data.get(0).unwrap() {
        assert_eq!(name, "line_data");
        assert_eq!(data.len(), 1);
        let hdata = data.get(0).unwrap();
        assert_eq!(hdata.get("buffer").unwrap(), &WeechatData::Pointer("0x7fcab15936d0".to_owned()));
        assert_eq!(hdata.get("date").unwrap(), &WeechatData::Time("1439651878".to_owned()));
        assert_eq!(hdata.get("date_printed").unwrap(), &WeechatData::Time("1439651878".to_owned()));
        assert_eq!(hdata.get("displayed").unwrap(), &WeechatData::Char('\u{1}'));
        assert_eq!(hdata.get("highlight").unwrap(), &WeechatData::Char('\u{0}'));
        let tags = WeechatData::Array(vec![WeechatData::String("irc_privmsg".to_owned()),
                                           WeechatData::String("notify_message".to_owned()),
                                           WeechatData::String("prefix_nick_cyan".to_owned()),
                                           WeechatData::String("nick_Wraithan".to_owned()),
                                           WeechatData::String("host_~wraithan@104.236.142.65".to_owned()),
                                           WeechatData::String("log1".to_owned())]);
        assert_eq!(hdata.get("tags_array").unwrap(), &tags);
        assert_eq!(hdata.get("prefix").unwrap(), &WeechatData::String("\u{19}F10\u{19}F13Wraithan".to_owned()));
        assert_eq!(hdata.get("message").unwrap(), &WeechatData::String("Hey".to_owned()));
    } else {
        panic!("unexpected type for first message");
    }
}

#[test]
fn blob_session() {
    let mut f = File::open("./tests/fodder/simple.dat").unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();

    let (tx, rx) = weechat_parser::new();
    tx.send(buffer).unwrap();

    validate_session(rx)
}

#[test]
fn single_byte_session() {
    let mut f = File::open("./tests/fodder/simple.dat").unwrap();
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();

    let (tx, rx) = weechat_parser::new();
    for item in buffer {
        tx.send(vec![item]).unwrap();
    }

    validate_session(rx)
}

fn validate_session(rx: Receiver<Result<WeechatMessage, WeechatParseError>>) {
    let message = rx.recv().unwrap().unwrap();
    assert_eq!(message.id, "_buffer_line_added");
    if let &WeechatData::Hdata(ref name, _, ref data) = message.data.get(0).unwrap() {
        assert_eq!(name, "line_data");
        assert_eq!(data.len(), 1);
        let hdata = data.get(0).unwrap();
        assert_eq!(hdata.get("buffer").unwrap(), &WeechatData::Pointer("0x7fcab15936d0".to_owned()));
        assert_eq!(hdata.get("date").unwrap(), &WeechatData::Time("1439651878".to_owned()));
        assert_eq!(hdata.get("date_printed").unwrap(), &WeechatData::Time("1439651878".to_owned()));
        assert_eq!(hdata.get("displayed").unwrap(), &WeechatData::Char('\u{1}'));
        assert_eq!(hdata.get("highlight").unwrap(), &WeechatData::Char('\u{0}'));
        let tags = WeechatData::Array(vec![WeechatData::String("irc_privmsg".to_owned()),
                                           WeechatData::String("notify_message".to_owned()),
                                           WeechatData::String("prefix_nick_cyan".to_owned()),
                                           WeechatData::String("nick_Wraithan".to_owned()),
                                           WeechatData::String("host_~wraithan@104.236.142.65".to_owned()),
                                           WeechatData::String("log1".to_owned())]);
        assert_eq!(hdata.get("tags_array").unwrap(), &tags);
        assert_eq!(hdata.get("prefix").unwrap(), &WeechatData::String("\u{19}F10\u{19}F13Wraithan".to_owned()));
        assert_eq!(hdata.get("message").unwrap(), &WeechatData::String("Hey".to_owned()));
    } else {
        panic!("unexpected type for first message");
    }

    let message2 = rx.recv().unwrap().unwrap();
    assert_eq!(message2.id, "_buffer_line_added");
    if let &WeechatData::Hdata(ref name, _, ref data) = message2.data.get(0).unwrap() {
        assert_eq!(name, "line_data");
        assert_eq!(data.len(), 1);
        let hdata = data.get(0).unwrap();
        assert_eq!(hdata.get("buffer").unwrap(), &WeechatData::Pointer("0x7fcab15936d0".to_owned()));
        assert_eq!(hdata.get("date").unwrap(), &WeechatData::Time("1439651883".to_owned()));
        assert_eq!(hdata.get("date_printed").unwrap(), &WeechatData::Time("1439651883".to_owned()));
        assert_eq!(hdata.get("displayed").unwrap(), &WeechatData::Char('\u{1}'));
        assert_eq!(hdata.get("highlight").unwrap(), &WeechatData::Char('\u{1}'));
        let tags = WeechatData::Array(vec![WeechatData::String("irc_privmsg".to_owned()),
                                           WeechatData::String("notify_message".to_owned()),
                                           WeechatData::String("prefix_nick_cyan".to_owned()),
                                           WeechatData::String("nick_Wraithan".to_owned()),
                                           WeechatData::String("host_~wraithan@104.236.142.65".to_owned()),
                                           WeechatData::String("log1".to_owned())]);
        assert_eq!(hdata.get("tags_array").unwrap(), &tags);
        assert_eq!(hdata.get("prefix").unwrap(), &WeechatData::String("\u{19}F10\u{19}F13Wraithan".to_owned()));
        assert_eq!(hdata.get("message").unwrap(), &WeechatData::String("test_bot: Hey".to_owned()));
    } else {
        panic!("unexpected type for second message");
    }

    let message3 = rx.recv().unwrap().unwrap();
    assert_eq!(message3.id, "_buffer_line_added");
    if let &WeechatData::Hdata(ref name, _, ref data) = message3.data.get(0).unwrap() {
        assert_eq!(name, "line_data");
        assert_eq!(data.len(), 1);
        let hdata = data.get(0).unwrap();
        assert_eq!(hdata.get("buffer").unwrap(), &WeechatData::Pointer("0x7fcab15936d0".to_owned()));
        assert_eq!(hdata.get("date").unwrap(), &WeechatData::Time("1439651900".to_owned()));
        assert_eq!(hdata.get("date_printed").unwrap(), &WeechatData::Time("1439651900".to_owned()));
        assert_eq!(hdata.get("displayed").unwrap(), &WeechatData::Char('\u{1}'));
        assert_eq!(hdata.get("highlight").unwrap(), &WeechatData::Char('\u{0}'));
        let tags = WeechatData::Array(vec![WeechatData::String("irc_privmsg".to_owned()),
                                           WeechatData::String("notify_none".to_owned()),
                                           WeechatData::String("no_highlight".to_owned()),
                                           WeechatData::String("prefix_nick_white".to_owned()),
                                           WeechatData::String("nick_test_bot".to_owned()),
                                           WeechatData::String("log1".to_owned())]);
        assert_eq!(hdata.get("tags_array").unwrap(), &tags);
        assert_eq!(hdata.get("message").unwrap(), &WeechatData::String("Hey".to_owned()));
        assert_eq!(hdata.get("prefix").unwrap(), &WeechatData::String("\u{19}F10\u{19}15test_bot".to_owned()));
    } else {
        panic!("unexpected type for third message");
    }

    // Two more messages that don't get processed by tests because they are basically the same as the above.
    rx.recv().unwrap().unwrap();
    rx.recv().unwrap().unwrap();
}
