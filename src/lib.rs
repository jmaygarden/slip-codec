//! Serial Line Internet Protocol (SLIP) encoder/decoder
//! 
//! [`SlipEncoder`] and [`SlipDecoder`] facilitate encoder and decoding of SLIP
//! data streams with `std::io::Read` and `std::io::Write` interfaces.
//! 
//! Enabling the `tokio-codec` feature makes a codec available for use with
//! the tokio runtime (see [`tokio::SlipCodec`]). If a different asynchronous
//! runtime is used, then the `async-codec` feature provides a runtime agnostic
//! SLIP codec based on the `asynchronous-codec` crate (see [`aio::SlipCodec`]).
//! 
//! [`SlipEncoder`]: crate::SlipEncoder
//! [`SlipDecoder`]: crate::SlipDecoder
//! [`tokio::SlipCodec`]: crate::tokio::SlipCodec
//! [`aio::SlipCodec`]: crate::aio::SlipCodec

mod encoder;
pub use encoder::SlipEncoder;

mod decoder;
pub use decoder::{SlipDecoder, SlipError, SlipResult};

#[cfg(feature = "async-codec")]
pub mod aio;

#[cfg(feature = "tokio-codec")]
pub mod tokio;

/// SLIP end of packet token
const END: u8 = 0xC0;

/// SLIP escape token
const ESC: u8 = 0xDB;

/// SLIP escaped 0xC0 token
const ESC_END: u8 = 0xDC;

/// SLIP escaped 0xDB token
const ESC_ESC: u8 = 0xDD;

/// Recommended maximum SLIP packet size per RFC 1055
#[allow(dead_code)]
const MAX_PACKET_SIZE: usize = 1006;
