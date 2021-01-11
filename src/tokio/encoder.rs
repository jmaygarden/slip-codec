use crate::{SlipCodecError, END, ESC, ESC_END, ESC_ESC};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::Encoder;

/// SLIP encoder context
pub struct SlipEncoder {}

impl SlipEncoder {
    /// Creates a new encoder context
    pub fn new() -> Self {
        Self {}
    }
}

impl Encoder<Bytes> for SlipEncoder {
    type Error = SlipCodecError;

    fn encode(&mut self, mut item: Bytes, dst: &mut BytesMut) -> Result<(), SlipCodecError> {
        dst.reserve(item.len());

        dst.put_u8(END);

        while item.has_remaining() {
            let value = item.get_u8();

            match value {
                END => {
                    dst.put_u8(ESC);
                    dst.put_u8(ESC_END);
                }
                ESC => {
                    dst.put_u8(ESC);
                    dst.put_u8(ESC_ESC);
                }
                _ => {
                    dst.put_u8(value);
                }
            }
        }

        dst.put_u8(END);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_encode() {
        const EXPECTED: [u8; 2] = [0xc0, 0xc0];
        let mut output = BytesMut::new();

        let mut slip = SlipEncoder::new();
        slip.encode(BytesMut::new(), &mut output).unwrap();
        assert_eq!(&EXPECTED[..], &output);
    }

    #[test]
    fn encode_esc_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, ESC, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_ESC, 0x03, 0xc0];
        let mut output = BytesMut::new();

        let mut slip = SlipEncoder::new();
        slip.encode(BytesMut::from(&INPUT[..]), &mut output)
            .unwrap();
        assert_eq!(&EXPECTED[..], &output);
    }

    #[test]
    fn encode_end_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, END, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_END, 0x03, 0xc0];
        let mut output = BytesMut::new();

        let mut slip = SlipEncoder::new();
        slip.encode(BytesMut::from(&INPUT[..]), &mut output)
            .unwrap();
        assert_eq!(&EXPECTED[..], &output);
    }
}
