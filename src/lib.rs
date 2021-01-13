#[cfg(not(feature = "async"))]
mod default;

#[cfg(not(feature = "async"))]
pub use crate::default::{encode, Decoder, Encoder, Error, Result};

#[cfg(feature = "async")]
mod tokio;

#[cfg(feature = "async")]
pub use crate::tokio::{SlipCodec, SlipCodecError, SlipDecoder, SlipEncoder};

const END: u8 = 0xC0;
const ESC: u8 = 0xDB;
const ESC_END: u8 = 0xDC;
const ESC_ESC: u8 = 0xDD;
