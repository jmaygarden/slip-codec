use futures::{ready, SinkExt, StreamExt};
use serialport::SerialPort;
use slip_codec::tokio::SlipCodec;
use std::io::{Read, Write};
use std::task::Context;
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::macros::support::{Pin, Poll};

type Result<T> = std::result::Result<T, std::io::Error>;

struct AsyncTTYPort {
    inner: AsyncFd<serialport::TTYPort>,
}

impl AsyncTTYPort {
    pub fn new(mut inner: serialport::TTYPort) -> Result<Self> {
        const ZERO: std::time::Duration = std::time::Duration::from_secs(0);

        inner.set_timeout(ZERO)?;

        Ok(Self {
            inner: AsyncFd::new(inner)?,
        })
    }

    pub fn pair() -> std::io::Result<(AsyncTTYPort, AsyncTTYPort)> {
        let (a, b) = serialport::TTYPort::pair()?;
        let a = AsyncTTYPort::new(a)?;
        let b = AsyncTTYPort::new(b)?;

        Ok((a, b))
    }
}

impl AsyncRead for AsyncTTYPort {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        let mut guard = ready!(self.inner.poll_read_ready_mut(cx))?;

        match guard.try_io(|inner| {
            let read = inner.get_mut().read(buf.initialize_unfilled())?;
            buf.advance(read);
            Ok(())
        }) {
            Ok(value) => Poll::Ready(value),
            Err(_) => Poll::Pending,
        }
    }
}

impl AsyncWrite for AsyncTTYPort {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<tokio::io::Result<usize>> {
        let mut guard = ready!(self.inner.poll_write_ready_mut(cx))?;

        match guard.try_io(|io| io.get_mut().write(buf)) {
            Ok(value) => Poll::Ready(value),
            Err(_) => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
        let mut guard = ready!(self.inner.poll_write_ready_mut(cx))?;

        match guard.try_io(|io| io.get_mut().flush()) {
            Ok(value) => Poll::Ready(value),
            Err(_) => Poll::Pending,
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

async fn run_source(port: AsyncTTYPort) {
    let mut sink = tokio_util::codec::Framed::new(port, SlipCodec::new());

    for message in ["foo", "bar", "baz"].iter() {
        let message = message.to_string().into();

        println!("send {:?}", message);
        sink.send(message).await.unwrap();
    }
}

async fn run_sink(port: AsyncTTYPort) {
    let mut source = tokio_util::codec::Framed::new(port, SlipCodec::new());

    loop {
        if let Some(result)  = source.next().await {
            match result {
                Ok(message) => println!("recv {:?}", message),
                Err(_) => break,
            }
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let (source, sink) = AsyncTTYPort::pair()?;
    let source_handle = tokio::spawn(run_source(source));
    let sink_handle = tokio::spawn(run_sink(sink));

    source_handle.await?;
    sink_handle.await?;

    Ok(())
}
