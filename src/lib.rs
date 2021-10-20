mod encoder;
pub use encoder::SlipEncoder;

mod decoder;
pub use decoder::{SlipDecoder, SlipError, SlipResult};

#[cfg(feature = "async-codec")]
pub mod aio;

#[cfg(feature = "tokio-codec")]
pub mod tokio;

const END: u8 = 0xC0;
const ESC: u8 = 0xDB;
const ESC_END: u8 = 0xDC;
const ESC_ESC: u8 = 0xDD;

const MAX_PACKET_SIZE: usize = 1006;
