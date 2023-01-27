use mio::{Events, Poll, Interest, Token};
use mio::net::TcpListener;
use nix::sys::socket::{AddressFamily, connect, MsgFlags, send, socket, SockaddrIn, SockFlag, SockType};

use std::io::IoSlice;
use std::str::FromStr;
use std::thread;

fn main() {
    thread::spawn(move || {

        let localhost = SockaddrIn::from_str("127.0.0.1:8081").unwrap();
        let fd = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();
        connect(fd, &localhost).expect("connect");

        std::thread::sleep(std::time::Duration::from_millis(1000));
        let iov = IoSlice::new(b"hello");
        let iov2 = IoSlice::new(b"x");
        // send something normal
        send(fd, &iov, MsgFlags::empty()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        // send something with msg_oob which should cause POLLPRI to be set on the receiver
        send(fd, &iov2, MsgFlags::MSG_OOB).unwrap();
    });

    let mut listener = TcpListener::bind("127.0.0.1:8081".parse().unwrap()).unwrap();
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(128);
    poll.registry().register(&mut listener, Token(0), Interest::READABLE).unwrap();
    poll.poll(&mut events, Some(std::time::Duration::from_millis(100))).unwrap();

    // wait for the client to be connected
    events.iter().next().unwrap();

    let (mut stream, _) = listener.accept().unwrap();

    // Register the socket with `Poll`
    poll.registry().register(&mut stream, Token(1), Interest::READABLE).unwrap();
    loop {
        poll.poll(&mut events, None).unwrap();

        for event in &events {
            dbg!(event);
        }
    }
}
