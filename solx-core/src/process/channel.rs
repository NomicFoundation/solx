//!
//! The length-prefixed CBOR frame protocol shared by both sides of the worker channel.
//!

use std::io::BufRead;
use std::io::Write;

/// The size of the frame length prefix.
const LENGTH_PREFIX_SIZE: usize = size_of::<usize>();

///
/// Writes length-prefixed CBOR frames.
///
pub trait FrameWrite: Write {
    ///
    /// Serializes `value` into a length-prefixed CBOR frame and flushes it.
    ///
    fn send<T>(&mut self, value: &T) -> anyhow::Result<()>
    where
        T: serde::Serialize,
    {
        let mut frame = vec![0u8; LENGTH_PREFIX_SIZE];
        ciborium::into_writer(value, &mut frame)
            .map_err(|error| anyhow::anyhow!("Frame serializing error: {error}"))?;
        let body_length = (frame.len() - LENGTH_PREFIX_SIZE).to_le_bytes();
        frame[..LENGTH_PREFIX_SIZE].copy_from_slice(body_length.as_slice());
        self.write_all(frame.as_slice())
            .and_then(|()| self.flush())
            .map_err(|error| anyhow::anyhow!("Frame writing error: {error}"))
    }
}

impl<W: Write + ?Sized> FrameWrite for W {}

///
/// Reads length-prefixed CBOR frames.
///
pub trait FrameRead: BufRead {
    ///
    /// Reads one length-prefixed CBOR frame, or `None` when the stream is at a frame boundary EOF.
    ///
    /// An empty buffer means the stream ended cleanly between frames; once a frame has
    /// started, `read_exact` turns any short read into a truncation error.
    ///
    fn recv<T>(&mut self) -> anyhow::Result<Option<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        if self
            .fill_buf()
            .map_err(|error| anyhow::anyhow!("Frame reading error: {error}"))?
            .is_empty()
        {
            return Ok(None);
        }
        let mut length_bytes = [0u8; LENGTH_PREFIX_SIZE];
        self.read_exact(length_bytes.as_mut_slice())
            .map_err(|error| anyhow::anyhow!("Frame length prefix reading error: {error}"))?;
        let mut body = vec![0u8; usize::from_le_bytes(length_bytes)];
        self.read_exact(body.as_mut_slice())
            .map_err(|error| anyhow::anyhow!("Frame body reading error: {error}"))?;
        ciborium::de::from_reader_with_recursion_limit(body.as_slice(), usize::MAX)
            .map(Some)
            .map_err(|error| anyhow::anyhow!("Frame deserializing error: {error}"))
    }
}

impl<R: BufRead + ?Sized> FrameRead for R {}
