//! Proxy selection: the builder, for a call site that carries its own
//! proxy configuration.
//!
//! The plain constructors funnel through [`Proxy::System`], which reads
//! `all_proxy` / `https_proxy` from the environment at dial time,
//! honours `no_proxy` and always bypasses loopback. That is the right
//! default for an application, and the wrong one for a call site whose
//! proxy comes from a config file, which is what
//! [`StreamStd::builder`] is for.
//!
//! Run with:
//! `HOST=imap.example.org PORT=993 PROXY=socks5://127.0.0.1:1080 cargo run --example proxy`

use std::{env, error::Error, io::Read};

use pimalaya_stream::{
    std::{proxy::Proxy, stream::StreamStd},
    tls::Tls,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let host = env::var("HOST")?;
    let port: u16 = env::var("PORT").unwrap_or_else(|_| "993".into()).parse()?;

    // NOTE: `socks5://`, `socks5h://`, `socks://`, `http://` and
    // `https://` parse, with optional credentials in the URL. Without
    // the variable this falls back to the ambient one, which is what
    // the plain constructors do.
    let proxy = match env::var("PROXY") {
        Ok(url) => Proxy::from_url(&url)?,
        Err(_) => Proxy::System,
    };

    let mut stream = StreamStd::builder(&host, port)
        .tls(Tls::default())
        .proxy(proxy)
        .connect()?;

    let mut buf = [0u8; 4096];
    let n = stream.read(&mut buf)?;

    print!("{}", String::from_utf8_lossy(&buf[..n]));

    Ok(())
}
