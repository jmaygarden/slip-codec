use super::*;
use std::io::{Read, Write};

#[derive(Debug)]
pub enum Error {
    FramingError,
    OversizedPacket,
    EndOfStream,
    ReadError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::ReadError(err)
    }
}

pub type Result = std::result::Result<usize, self::Error>;

enum State {
    Normal,
    Error,
    Escape,
}

/// SLIP decoding context
pub struct Decoder {
    count: usize,
    state: State,
}

impl Decoder {
    /// Creates a new context with the given maximum buffer size.
    pub fn new() -> Self {
        Self {
            count: 0usize,
            state: State::Normal,
        }
    }

    fn push(self: &mut Self, sink: &mut dyn Write, value: u8) -> self::Result {
        match sink.write(&[value]) {
            Ok(len) => {
                if len != 1 {
                    Err(Error::OversizedPacket)
                } else {
                    self.count += 1;
                    Ok(1usize)
                }
            }
            Err(error) => Err(error.into()),
        }
    }

    /// Attempts to decode a single SLIP frame from the given source.
    ///
    /// # Arguments
    ///
    /// * `source` - Encoded SLIP data source implementing the std::io::Read
    ///              trait
    ///
    /// Returns a Vec<u8> containing a decoded message or an empty Vec<u8> if
    /// of the source data was reached.
    ///
    pub fn decode(self: &mut Self, source: &mut dyn Read, sink: &mut dyn Write) -> self::Result {
        for value in source.bytes() {
            let value = value?;

            match self.state {
                State::Normal => match value {
                    END => {
                        if self.count > 0 {
                            let len = self.count;

                            self.count = 0usize;

                            return Ok(len);
                        }
                    }
                    ESC => {
                        self.state = State::Escape;
                    }
                    _ => {
                        self.push(sink, value)?;
                    }
                },
                State::Error => {
                    if value == END {
                        self.count = 0usize;
                        self.state = State::Normal;
                    }
                }
                State::Escape => match value {
                    ESC_END => {
                        self.push(sink, END)?;
                        self.state = State::Normal;
                    }
                    ESC_ESC => {
                        self.push(sink, ESC)?;
                        self.state = State::Normal;
                    }
                    _ => {
                        self.state = State::Error;

                        return Err(Error::FramingError);
                    }
                },
            }
        }

        Err(Error::EndOfStream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_decode() {
        const INPUT: [u8; 2] = [0xc0, 0xc0];

        let mut slip = Decoder::new();
        let mut buf: Vec<u8> = Vec::new();
        let res = slip.decode(&mut INPUT.as_ref(), &mut buf);
        assert!(res.is_err());
        assert!(buf.is_empty());
    }

    #[test]
    fn simple_decode() {
        const INPUT: [u8; 7] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05, 0xc0];
        const DATA: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];

        let mut slip = Decoder::new();
        let mut buf = [0u8; DATA.len()];
        let len = slip.decode(&mut INPUT.as_ref(), &mut buf.as_mut()).unwrap();
        assert_eq!(DATA.len(), len);
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA, &buf);
    }

    /// Ensure that [ESC, ESC_END] -> [END]
    #[test]
    fn decode_esc_then_esc_end_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdc, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xc0, 0x03];

        let mut slip = Decoder::new();
        let mut buf: Vec<u8> = Vec::new();
        let len = slip.decode(&mut INPUT.as_ref(), &mut buf).unwrap();
        assert_eq!(DATA.len(), len);
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA, buf.as_slice());
    }

    /// Ensure that [ESC, ESC_ESC] -> [ESC]
    #[test]
    fn decode_esc_then_esc_esc_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdd, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xdb, 0x03];

        let mut slip = Decoder::new();
        let mut buf: Vec<u8> = Vec::new();
        let len = slip.decode(&mut INPUT.as_ref(), &mut buf).unwrap();
        assert_eq!(DATA.len(), len);
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA, buf.as_slice());
    }

    #[test]
    fn multi_part_decode() {
        const INPUT_1: [u8; 6] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05];
        const INPUT_2: [u8; 6] = [0x05, 0x06, 0x07, 0x08, 0x09, 0xc0];
        const DATA: [u8; 10] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x05, 0x06, 0x07, 0x08, 0x09];

        let mut slip = Decoder::new();
        let mut buf: Vec<u8> = Vec::new();

        {
            let res = slip.decode(&mut INPUT_1.as_ref(), &mut buf);
            assert!(res.is_err());
            assert_eq!(5, buf.len());
        }

        {
            let len = slip.decode(&mut INPUT_2.as_ref(), &mut buf).unwrap();
            assert_eq!(DATA.len(), len);
            assert_eq!(DATA.len(), buf.len());
            assert_eq!(&DATA, buf.as_slice());
        }
    }
}
