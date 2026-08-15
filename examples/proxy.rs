//! Proxy selection: the connect options, for a call site that carries
//! its own proxy configuration.
//!
//! The options default to [`Proxy::System`], which reads `all_proxy` /
//! `https_proxy` from the environment at dial time, honours `no_proxy`
//! and always bypasses loopback. That is the right default for an
//! application, and the wrong one for a call site whose proxy comes
//! from a config file, which is the field set here.
//!
//! Run with:
//! `HOST=imap.example.org PORT=993 PROXY=socks5://127.0.0.1:1080 cargo run --example proxy`

use std::{env, error::Error, io::Read};

use pimalaya_stream::{
    proxy::Proxy,
    stream::{Stream, StreamTlsConnectOptions},
    tls::Tls,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let host = env::var("HOST")?;
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "993".into()).parse()?;

    // NOTE: `socks5://`, `socks5h://`, `socks://`, `http://` and
    // `https://` parse, with optional credentials in the URL. Without
    // the variable this falls back to the ambient one, which is what
    // the options default to.
    let proxy = match env::var("PROXY") {
        Ok(url) => Proxy::from_url(&url)?,
        Err(_) => Proxy::System,
    };

    let opts = StreamTlsConnectOptions {
        tls: Tls::default(),
        proxy,
        ..Default::default()
    };

    let mut stream = Stream::connect_tls(&host, port, opts)?;

    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;

    print!("{}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
