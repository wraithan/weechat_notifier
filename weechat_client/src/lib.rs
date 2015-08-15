#![feature(socket_timeout)]

extern crate weechat_parser;

use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        if let Err(x) = writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            panic!("Unable to write to stderr: {}", x);
        }
    )
);

pub struct WeechatRelay {
    in_stream: TcpStream,
    out_stream: TcpStream,
}

impl WeechatRelay {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<WeechatRelay, String> {
        if let Ok(out_stream) = TcpStream::connect(addr) {
            out_stream.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();
            if let Ok(in_stream) = out_stream.try_clone() {
                return Ok(WeechatRelay{
                    in_stream: in_stream,
                    out_stream: out_stream
                })
            }
        } else {
            println_stderr!("couldn't connect");
        }
        return Err("mooooo".to_owned())
    }
}

pub fn decode() {
    let mut out_stream = TcpStream::connect("127.0.0.1:9000").unwrap();
    let mut in_stream = out_stream.try_clone().unwrap();
    in_stream.set_read_timeout(Some(Duration::from_millis(1000))).unwrap();
    out_stream.write("init\n".as_bytes()).unwrap();
    out_stream.write("sync\n".as_bytes()).unwrap();
    let mut buffer = Vec::<u8>::with_capacity(150);
    println_stderr!("about to read");

    while buffer.len() < 1000 {
        println_stderr!("buffer contains {}", buffer.len());
        match in_stream.read_to_end(&mut buffer) {
            Ok(count) => println_stderr!("got {}", count),
            Err(_) => println_stderr!("nothing")
        }
    }
    println_stderr!("full data: {:?}", buffer);
    if let Ok(length) = weechat_parser::get_length(&buffer) {
        println_stderr!("got length of {}", length);
        assert!(length == 145);
    } else {
        println_stderr!("parser error");
    }

    println_stderr!("done reading");
    out_stream.write("quit\n".as_bytes()).unwrap();
}


#[test]
fn it_works() {
    WeechatRelay::connect("127.0.0.1:9000").unwrap();
    decode();
}
