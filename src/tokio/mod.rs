mod decoder;
pub use decoder::SlipDecoder;

mod encoder;
pub use encoder::SlipEncoder;

#[derive(Debug)]
pub enum SlipCodecError {
    FramingError,
    OversizedPacket,
    Io(std::io::Error),
}

impl From<std::io::Error> for SlipCodecError {
    fn from(err: std::io::Error) -> Self {
        SlipCodecError::Io(err)
    }
}
