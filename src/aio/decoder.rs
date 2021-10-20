use crate::{SlipError, MAX_PACKET_SIZE};
use bytes::{Buf, BufMut, BytesMut};
use asynchronous_codec::Decoder;

/// SLIP decoding context
pub struct SlipDecoder {
    buf: BytesMut,
    capacity: usize,
    inner: crate::SlipDecoder,
}

impl SlipDecoder {
    /// Creates a new context with the given maximum buffer size.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: BytesMut::with_capacity(capacity),
            capacity,
            inner: Default::default(),
        }
    }
}

impl Decoder for SlipDecoder {
    type Item = BytesMut;
    type Error = SlipError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let src = &mut src.reader();
        let dst = {
            self.buf.reserve(self.capacity);
            &mut (&mut self.buf).limit(self.capacity).writer()
        };

        match self.inner.decode(src, dst) {
            Ok(len) => Ok(Some(self.buf.split_to(len))),
            Err(SlipError::EndOfStream) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

impl Default for SlipDecoder {
    fn default() -> Self {
        Self::with_capacity(MAX_PACKET_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_decode() {
        const INPUT: [u8; 2] = [0xc0, 0xc0];

        let mut slip = SlipDecoder::default();
        let mut buf = BytesMut::from(&INPUT[..]);
        let res = slip.decode(&mut buf).unwrap();
        assert!(res.is_none());
        assert!(buf.is_empty());
    }

    #[test]
    fn simple_decode() {
        const INPUT: [u8; 7] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05, 0xc0];
        const DATA: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];

        let mut slip = SlipDecoder::default();
        let mut buf = BytesMut::from(&INPUT[..]);
        let buf = slip.decode(&mut buf).unwrap().unwrap();
        eprintln!("{}:{}: {:?}", file!(), line!(), buf);
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA[..], &buf);
    }

    /// Ensure that [ESC, ESC_END] -> [END]
    #[test]
    fn decode_esc_then_esc_end_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdc, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xc0, 0x03];

        let mut slip = SlipDecoder::default();
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

        let mut slip = SlipDecoder::default();
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

        let mut slip = SlipDecoder::default();
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
