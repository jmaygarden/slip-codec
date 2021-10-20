use bytes::{BufMut, Bytes, BytesMut};
use asynchronous_codec::Encoder;

/// SLIP encoder context
pub struct SlipEncoder {
    inner: crate::SlipEncoder,
}

impl SlipEncoder {
    /// Creates a new encoder context
    pub fn new(begin_with_end: bool) -> Self {
        Self {
            inner: crate::SlipEncoder::new(begin_with_end),
        }
    }
}

impl Encoder for SlipEncoder {
    type Item = Bytes;
    type Error = std::io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.inner
            .encode(item.as_ref(), &mut dst.writer())
            .map(|_| ())
    }
}

impl Default for SlipEncoder {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{END, ESC, ESC_END, ESC_ESC};

    #[test]
    fn empty_encode() {
        const EXPECTED: [u8; 2] = [0xc0, 0xc0];

        // default is to begin and end with END tokens
        let mut output = BytesMut::new();
        let mut slip = SlipEncoder::default();
        slip.encode(Bytes::new(), &mut output).unwrap();
        assert_eq!(&EXPECTED[..], &output);

        // override to only use END token to terminate packet
        let mut output = BytesMut::new();
        let mut slip = SlipEncoder::new(false);
        slip.encode(Bytes::new(), &mut output).unwrap();
        assert_eq!(&EXPECTED[..1], &output);
    }

    #[test]
    fn encode_esc_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, ESC, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_ESC, 0x03, 0xc0];
        let mut output = BytesMut::new();

        let mut slip = SlipEncoder::default();
        slip.encode(Bytes::from(&INPUT[..]), &mut output).unwrap();
        assert_eq!(&EXPECTED[..], &output);
    }

    #[test]
    fn encode_end_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, END, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_END, 0x03, 0xc0];
        let mut output = BytesMut::new();

        let mut slip = SlipEncoder::default();
        slip.encode(Bytes::from(&INPUT[..]), &mut output).unwrap();
        assert_eq!(&EXPECTED[..], &output);
    }
}
