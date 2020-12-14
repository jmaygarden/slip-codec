use crate::{SlipCodecError, END, ESC, ESC_END, ESC_ESC};

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::Decoder;

enum State {
    Normal,
    Error,
    Escape,
}

/// SLIP decoding context
pub struct SlipDecoder {
    buf: BytesMut,
    capacity: Option<usize>,
    state: State,
}

const INITIAL_CAPACITY: usize = 1024;

impl SlipDecoder {
    /// Creates a new context without a maximum buffer size.
    pub fn new() -> Self {
        Self {
            buf: BytesMut::with_capacity(INITIAL_CAPACITY),
            capacity: None,
            state: State::Normal,
        }
    }

    /// Creates a new context with the given maximum buffer size.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(capacity),
            capacity: Some(capacity),
            state: State::Normal,
        }
    }

    fn push(self: &mut Self, value: u8) -> Result<(), SlipCodecError> {
        if let Some(capacity) = self.capacity {
            if self.buf.len() == capacity {
                return Err(SlipCodecError::OversizedPacket);
            }
        }

        self.buf.put_u8(value);

        Ok(())
    }
}

impl Decoder for SlipDecoder {
    type Item = BytesMut;
    type Error = SlipCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<BytesMut>, SlipCodecError> {
        while src.has_remaining() {
            let value = src.get_u8();

            match self.state {
                State::Normal => match value {
                    END => {
                        if self.buf.has_remaining() {
                            return Ok(Some(self.buf.split()));
                        }
                    }
                    ESC => {
                        self.state = State::Escape;
                    }
                    _ => {
                        self.push(value)?;
                    }
                },
                State::Error => {
                    if value == END {
                        self.buf.clear();
                        self.state = State::Normal;
                    }
                }
                State::Escape => match value {
                    ESC_END => {
                        self.push(END)?;
                        self.state = State::Normal;
                    }
                    ESC_ESC => {
                        self.push(ESC)?;
                        self.state = State::Normal;
                    }
                    _ => {
                        self.state = State::Error;

                        return Err(SlipCodecError::FramingError);
                    }
                },
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_decode() {
        const INPUT: [u8; 2] = [0xc0, 0xc0];

        let mut slip = SlipDecoder::new();
        let mut buf = BytesMut::from(&INPUT[..]);
        let res = slip.decode(&mut buf).unwrap();
        assert!(res.is_none());
        assert!(buf.is_empty());
    }

    #[test]
    fn simple_decode() {
        const INPUT: [u8; 7] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05, 0xc0];
        const DATA: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];

        let mut slip = SlipDecoder::new();
        let mut buf = BytesMut::from(&INPUT[..]);
        let buf = slip.decode(&mut buf).unwrap().unwrap();
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA[..], &buf);
    }

    /// Ensure that [ESC, ESC_END] -> [END]
    #[test]
    fn decode_esc_then_esc_end_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdc, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xc0, 0x03];

        let mut slip = SlipDecoder::new();
        let mut buf = BytesMut::from(&INPUT[..]);
        let buf = slip.decode(&mut buf).unwrap().unwrap();
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA[..], &buf);
    }

    /// Ensure that [ESC, ESC_ESC] -> [ESC]
    #[test]
    fn decode_esc_then_esc_esc_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdd, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xdb, 0x03];

        let mut slip = SlipDecoder::new();
        let mut buf = BytesMut::from(&INPUT[..]);
        let buf = slip.decode(&mut buf).unwrap().unwrap();
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA[..], &buf);
    }

    #[test]
    fn multi_part_decode() {
        const INPUT_1: [u8; 6] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05];
        const INPUT_2: [u8; 6] = [0x05, 0x06, 0x07, 0x08, 0x09, 0xc0];
        const DATA: [u8; 10] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x05, 0x06, 0x07, 0x08, 0x09];

        let mut slip = SlipDecoder::new();
        let mut buf = BytesMut::from(&INPUT_1[..]);

        {
            let res = slip.decode(&mut buf).unwrap();
            assert!(res.is_none());
            assert_eq!(0, buf.len());
        }

        buf.extend_from_slice(&INPUT_2[..]);

        {
            let buf = slip.decode(&mut buf).unwrap().unwrap();
            assert_eq!(DATA.len(), buf.len());
            assert_eq!(&DATA[..], &buf);
        }
    }
}
