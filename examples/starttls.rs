//! STARTTLS: connect in the clear, exchange the upgrade command with
//! the server, then swap the plain socket for a TLS session.
//!
//! [`Stream::upgrade_tls`] consumes the plain stream and returns the
//! encrypted one, so a caller cannot keep writing to the socket it just
//! upgraded. What it does not do is speak the protocol: the `STARTTLS`
//! command below is SMTP's, and deciding when to send it belongs to the
//! protocol crate (io-smtp's session coroutine yields the upgrade
//! request at the right moment, and refuses it when the server appended
//! bytes to its reply).
//!
//! Run with: `HOST=smtp.example.org PORT=587 cargo run --example starttls`

use std::{
    env,
    error::Error,
    io::{Read, Write},
};

use pimalaya_stream::{
    std::stream::{Stream, StreamTcpConnectOptions},
    tls::Tls,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let host = env::var("HOST")?;
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "587".into()).parse()?;

    let tls = Tls::default();
    let opts = StreamTcpConnectOptions::default();
    let mut stream = Stream::connect_tcp(&host, port, opts)?;
    let mut buf = [0u8; 4096];

    // NOTE: greeting, then EHLO, then the upgrade command. A real
    // client reads these through a parser rather than by eye.
    let n = stream.read(&mut buf)?;
    print!("{}", String::from_utf8_lossy(&buf[..n]));

    stream.write_all(format!("EHLO {host}\r\n").as_bytes())?;
    let n = stream.read(&mut buf)?;
    print!("{}", String::from_utf8_lossy(&buf[..n]));

    stream.write_all(b"STARTTLS\r\n")?;
    let n = stream.read(&mut buf)?;
    print!("{}", String::from_utf8_lossy(&buf[..n]));

    let mut stream = stream.upgrade_tls(&tls)?;

    stream.write_all(format!("EHLO {host}\r\n").as_bytes())?;
    let n = stream.read(&mut buf)?;
    print!("{}", String::from_utf8_lossy(&buf[..n]));

    stream.write_all(b"QUIT\r\n")?;

    Ok(())
}
