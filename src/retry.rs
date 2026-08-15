//! What a stream does when a socket is not ready yet.
//!
//! [`Retry`] is the strategy a stream is opened with, and the
//! loop honoring it lives beside it here, as the `retry` method every
//! read, write and flush goes through. The strategy is data saying how
//! long to keep asking; running the operation belongs to the stream,
//! the only thing holding a socket.

use std::{
    io, thread,
    time::{Duration, Instant},
};

use log::debug;

use crate::stream::Stream;

/// How long [`Retry::default`] keeps retrying a stream that is
/// not ready.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Pause before the first retry, doubled on each further one up to
/// [`RETRY_BACKOFF_MAX`].
///
/// It only comes into play when the stream reports not-ready without
/// having waited, a socket read deadline being what otherwise paces the
/// attempts.
const RETRY_BACKOFF_MIN: Duration = Duration::from_millis(1);
/// Longest pause between two retries.
const RETRY_BACKOFF_MAX: Duration = Duration::from_millis(250);

/// What a stream does when a read or a write reports it is not ready.
///
/// A blocking socket is not supposed to report `EAGAIN`, yet callers do
/// see one surface mid-exchange (macOS especially), and any socket
/// carrying a read deadline reports its expiry the same way. Neither
/// says the session is over, so a strategy says how long to keep asking
/// before it is called one.
#[derive(Clone, Copy, Debug)]
pub enum Retry {
    /// Hands every failure back to the caller, untouched.
    ///
    /// For a loop that wants the not-ready failures: a watcher polling
    /// a shutdown flag between IDLE keep-alives, a socket driven from a
    /// poller. Such a caller arms its own read deadline, and connecting
    /// with this strategy arms none.
    Never,
    /// Retries until this long passes without the stream making
    /// progress, then fails with `TimedOut` and a message saying so.
    ///
    /// Each read and each write gets its own budget, so a slow but
    /// progressing transfer never runs out. Connecting with it arms the
    /// socket read deadline to the same value, without which a server
    /// that goes silent would block the caller rather than run the
    /// budget down.
    Until(Duration),
}

impl Default for Retry {
    /// Retries for [`DEFAULT_TIMEOUT`].
    fn default() -> Self {
        Self::Until(DEFAULT_TIMEOUT)
    }
}

impl Stream {
    /// Attempts `op` until it succeeds, fails for a reason other than
    /// "not ready yet", or this stream's [`Retry`] runs out.
    ///
    /// The one place a strategy is honored, shared by the read, the
    /// write and the flush so none of them can drift from the other two.
    /// `op` is handed the stream back rather than closing over it, which
    /// is what lets this loop live here while the socket it touches
    /// stays private to the stream module.
    pub(crate) fn retry<T>(
        &mut self,
        mut op: impl FnMut(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        let Retry::Until(timeout) = self.retry else {
            return op(self);
        };

        let start = Instant::now();
        let mut backoff = RETRY_BACKOFF_MIN;
        let mut retries = 0usize;

        loop {
            let err = match op(self) {
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) if matches!(err.kind(), io::ErrorKind::TimedOut) => err,
                Err(err) if matches!(err.kind(), io::ErrorKind::WouldBlock) => err,
                Err(err) => return Err(err),
                Ok(out) => {
                    if retries > 0 {
                        debug!("stream ready again after {retries} retries");
                    }
                    return Ok(out);
                }
            };

            if start.elapsed() >= timeout {
                debug!("give up on stream after {retries} retries: {err}");
                let kind = io::ErrorKind::TimedOut;
                let msg = format!("stream stopped responding after {timeout:?}");
                return Err(io::Error::new(kind, msg));
            }

            let kind = err.kind();
            let errno = err.raw_os_error();
            debug!("retry stream after transient {kind:?} failure (errno {errno:?}): {err}");

            thread::sleep(backoff);
            backoff = (backoff * 2).min(RETRY_BACKOFF_MAX);
            retries += 1;
        }
    }
}
