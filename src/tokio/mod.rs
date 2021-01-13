use bytes::{Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

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

pub struct SlipCodec {
    decoder: SlipDecoder,
    encoder: SlipEncoder,
}

impl SlipCodec {
    pub fn new() -> Self {
        Self {
            decoder: SlipDecoder::new(),
            encoder: SlipEncoder::new(),
        }
    }
}

impl Decoder for SlipCodec {
    type Item = BytesMut;
    type Error = SlipCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<BytesMut>, SlipCodecError> {
        self.decoder.decode(src)
    }
}

impl Encoder<Bytes> for SlipCodec {
    type Error = SlipCodecError;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), SlipCodecError> {
        self.encoder.encode(item, dst)
    }
}
