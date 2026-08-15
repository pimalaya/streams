//! Implicit TLS: one connect, one handshake, one `Read + Write`
//! handle.
//!
//! [`Stream::connect_tls`] resolves the ambient proxy, opens the TCP
//! connection and runs the handshake with the provider the enabled
//! cargo feature selected. What comes back is an ordinary blocking
//! stream, which is exactly what an io- protocol client asks for.
//!
//! Run with: `HOST=imap.example.org PORT=993 cargo run --example tls`

use std::{
    env,
    error::Error,
    io::{Read, Write},
};

use pimalaya_stream::{
    stream::{Stream, StreamTlsConnectOptions},
    tls::Tls,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let host = env::var("HOST")?;
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "993".into()).parse()?;

    // NOTE: the defaults pick the first enabled provider and the
    // platform trust anchors; `alpn` and `cert` are the two fields a
    // protocol client usually sets. The other options default to the
    // ambient proxy and to retrying a not-ready stream for a minute.
    let opts = StreamTlsConnectOptions {
        tls: Tls::default(),
        ..Default::default()
    };

    let mut stream = Stream::connect_tls(&host, port, opts)?;
    let mut buf = [0u8; 4096];

    let n = stream.read(&mut buf)?;
    print!("{}", String::from_utf8_lossy(&buf[..n]));

    stream.write_all(b"a LOGOUT\r\n")?;

    let n = stream.read(&mut buf)?;
    print!("{}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
