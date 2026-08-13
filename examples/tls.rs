//! Implicit TLS: one connect, one handshake, one `Read + Write`
//! handle.
//!
//! [`StreamStd::connect_tls`] resolves the ambient proxy, opens the TCP
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

use pimalaya_stream::{std::stream::StreamStd, tls::Tls};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let host = env::var("HOST")?;
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "993".into()).parse()?;

    // NOTE: the defaults pick the first enabled provider and the
    // platform trust anchors; `alpn` and `cert` are the two fields a
    // protocol client usually sets.
    let tls = Tls::default();

    let mut stream = StreamStd::connect_tls(&host, port, &tls)?;
    let mut buf = [0u8; 4096];

    let n = stream.read(&mut buf)?;
    print!("{}", String::from_utf8_lossy(&buf[..n]));

    stream.write_all(b"a LOGOUT\r\n")?;

    let n = stream.read(&mut buf)?;
    print!("{}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
