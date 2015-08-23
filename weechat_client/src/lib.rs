#![feature(socket_timeout)]

extern crate weechat_parser;

use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use weechat_parser::WeechatData;

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
    let (tx, rx) = weechat_parser::new();
    loop {
        let mut buffer = Vec::<u8>::with_capacity(150);
        match in_stream.read_to_end(&mut buffer) {
            Ok(count) =>{
                if count > 0 {
                    println_stderr!("sending {}", buffer.len());
                }
            },
            Err(e) => {}//println_stderr!("error reading {:?}", e)
        }
        println_stderr!("received chunk of size: {:?}", buffer.len());
        tx.send(buffer).unwrap();
        loop {
            match rx.try_recv() {
                Ok(res) => match res {
                    Ok(message) => {
                        if message.id == "_buffer_line_added" {
                            if let &WeechatData::Hdata(ref name, ref data) = message.data.get(0).unwrap() {
                                let body = data.get(0).unwrap();
                                if let &WeechatData::Char(ref highlight) = body.get("highlight").unwrap() {
                                    if highlight == &'\u{1}' {
                                        println_stderr!("Got message: {:?}", body.get("message").unwrap());
                                    }
                                }
                            }
                        }
                    },
                    Err(e) => {
                        println_stderr!("error parsing {:?}", e);
                        break;
                    }
                },
                Err(e) => {
                    break;
                }
            }
        }
    }

    println_stderr!("done reading");
    out_stream.write("quit\n".as_bytes()).unwrap();
}


#[test]
fn it_works() {
    WeechatRelay::connect("127.0.0.1:9000").unwrap();
    decode();
}
