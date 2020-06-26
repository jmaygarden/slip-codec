mod decoder;
mod encoder;

pub use crate::decoder::{Decoder, Error, Result};
pub use crate::encoder::Encoder;

const END: u8 = 0xC0;
const ESC: u8 = 0xDB;
const ESC_END: u8 = 0xDC;
const ESC_ESC: u8 = 0xDD;
