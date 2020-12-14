#[cfg(not(feature = "tokio"))]
mod default;

#[cfg(not(feature = "tokio"))]
pub use crate::default::{encode, Decoder, Encoder, Error, Result};

#[cfg(feature = "tokio")]
mod tokio;

#[cfg(feature = "tokio")]
pub use crate::tokio::{SlipCodecError, SlipDecoder, SlipEncoder};

const END: u8 = 0xC0;
const ESC: u8 = 0xDB;
const ESC_END: u8 = 0xDC;
const ESC_ESC: u8 = 0xDD;
