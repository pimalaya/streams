//! The retry strategy over a real socket, which is the only place it
//! can be seen working: the loop is private, so these drive it the way
//! a consumer does, through `Read` and `Write` on a connected stream.
//!
//! A Unix-domain listener is what they connect to. It needs no port, and
//! it reports a peer that hung up as a broken pipe on the first write,
//! where TCP only does once the kernel stops buffering. Each test binds
//! its own socket file so they can run side by side.
#![cfg(unix)]

use std::{
    env, fs,
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    thread,
    time::{Duration, Instant},
};

use pimalaya_stream::{
    retry::StreamRetry,
    stream::{Stream, StreamUnixConnectOptions},
};

/// Opens a stream on `retry` against a throwaway socket file, returning
/// it with the accepted peer to drive the far side from.
///
/// The read deadline is shortened to five milliseconds whatever the
/// strategy says, so a socket with nothing to hand over reports itself
/// promptly instead of parking the test for the whole budget.
fn connected(name: &str, retry: StreamRetry) -> (Stream, UnixStream) {
    let path = env::temp_dir().join(format!("pimalaya-stream-{name}.sock"));
    let _ = fs::remove_file(&path);

    let listener = UnixListener::bind(&path).unwrap();
    let accept = thread::spawn(move || listener.accept().unwrap().0);

    let stream = Stream::connect_unix(&path, StreamUnixConnectOptions { retry }).unwrap();
    let far = accept.join().unwrap();

    fs::remove_file(&path).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_millis(5)))
        .unwrap();

    (stream, far)
}

#[test]
fn a_not_ready_read_is_retried_until_the_bytes_arrive() {
    let (mut stream, mut far) = connected("retried", StreamRetry::Until(Duration::from_secs(5)));

    let peer = thread::spawn(move || {
        thread::sleep(Duration::from_millis(20));
        far.write_all(b"* OK server ready\r\n").unwrap();
    });

    let mut buf = [0u8; 64];
    let n = stream.read(&mut buf).unwrap();
    peer.join().unwrap();

    assert_eq!(&buf[..n], b"* OK server ready\r\n");
}

#[test]
fn a_read_that_never_becomes_ready_gives_up_with_a_timeout() {
    let (mut stream, _far) = connected("timeout", StreamRetry::Until(Duration::from_millis(50)));

    let mut buf = [0u8; 64];
    let err = stream.read(&mut buf).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
    assert_eq!(err.to_string(), "stream stopped responding after 50ms");
}

#[test]
fn never_hands_a_not_ready_read_straight_back() {
    let (mut stream, _far) = connected("never", StreamRetry::Never);

    let mut buf = [0u8; 64];
    let err = stream.read(&mut buf).unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::WouldBlock);
}

#[test]
fn a_broken_stream_is_reported_on_the_spot() {
    let (mut stream, far) = connected("broken", StreamRetry::Until(Duration::from_secs(60)));
    drop(far);

    let start = Instant::now();
    let err = stream.write(b"A1 NOOP\r\n").unwrap_err();

    assert_eq!(err.kind(), std::io::ErrorKind::BrokenPipe);
    // the budget was a minute: anything close to it would mean the write
    // was retried, which a broken pipe never is
    assert!(start.elapsed() < Duration::from_secs(1));
}
