use super::{SlipDecoder, SlipEncoder};
use crate::{SlipError, MAX_PACKET_SIZE};
use bytes::{Bytes, BytesMut};
use asynchronous_codec::{Decoder, Encoder};

pub struct SlipCodec {
    decoder: SlipDecoder,
    encoder: SlipEncoder,
}

pub struct SlipCodecBuilder {
    begin_with_end: bool,
    capacity: usize,
}

impl SlipCodec {
    pub fn new() -> Self {
        Self {
            decoder: SlipDecoder::default(),
            encoder: SlipEncoder::default(),
        }
    }

    pub fn builder() -> SlipCodecBuilder {
        SlipCodecBuilder {
            begin_with_end: true,
            capacity: MAX_PACKET_SIZE,
        }
    }
}

impl Decoder for SlipCodec {
    type Item = BytesMut;
    type Error = SlipError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<BytesMut>, Self::Error> {
        self.decoder.decode(src)
    }
}

impl Encoder for SlipCodec {
    type Item = Bytes;
    type Error = SlipError;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.encoder.encode(item, dst).map_err(SlipError::ReadError)
    }
}

impl Default for SlipCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl SlipCodecBuilder {
    pub fn begin_with_end(self, begin_with_end: bool) -> Self {
        Self {
            begin_with_end,
            ..self
        }
    }

    pub fn capacity(self, capacity: usize) -> Self {
        Self { capacity, ..self }
    }

    pub fn build(self) -> SlipCodec {
        SlipCodec {
            decoder: SlipDecoder::with_capacity(self.capacity),
            encoder: SlipEncoder::new(self.begin_with_end),
        }
    }
}
