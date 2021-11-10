use crate::{END, ESC, ESC_END, ESC_ESC};
use std::io::{Read, Write};

/// SLIP decoder error type
#[derive(Debug)]
pub enum SlipError {
    FramingError,
    OversizedPacket,
    EndOfStream,
    ReadError(std::io::Error),
}

impl From<SlipError> for std::io::Error {
    fn from(err: SlipError) -> std::io::Error {
        match err {
            SlipError::FramingError => {
                std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err))
            }
            SlipError::OversizedPacket => {
                std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err))
            }
            SlipError::EndOfStream => {
                std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", err))
            }
            SlipError::ReadError(err) => err,
        }
    }
}

impl From<std::io::Error> for SlipError {
    fn from(err: std::io::Error) -> Self {
        SlipError::ReadError(err)
    }
}

pub type SlipResult = std::result::Result<usize, self::SlipError>;

enum State {
    Normal,
    Error,
    Escape,
}

/// SLIP decoder context
pub struct SlipDecoder {
    count: usize,
    state: State,
}

impl SlipDecoder {
    /// Creates a new context with the given maximum buffer size.
    pub fn new() -> Self {
        Self {
            count: 0usize,
            state: State::Normal,
        }
    }

    fn push(&mut self, sink: &mut dyn Write, value: u8) -> self::SlipResult {
        match sink.write(&[value]) {
            Ok(len) => {
                if len != 1 {
                    Err(SlipError::OversizedPacket)
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
    pub fn decode(&mut self, source: &mut dyn Read, sink: &mut dyn Write) -> self::SlipResult {
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

                        return Err(SlipError::FramingError);
                    }
                },
            }
        }

        Err(SlipError::EndOfStream)
    }
}

impl Default for SlipDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_decode() {
        const INPUT: [u8; 2] = [0xc0, 0xc0];

        let mut slip = SlipDecoder::new();
        let mut buf: Vec<u8> = Vec::new();
        let res = slip.decode(&mut INPUT.as_ref(), &mut buf);
        assert!(res.is_err());
        assert!(buf.is_empty());
    }

    #[test]
    fn simple_decode() {
        const INPUT: [u8; 7] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05, 0xc0];
        const DATA: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];

        let mut slip = SlipDecoder::new();
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

        let mut slip = SlipDecoder::new();
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

        let mut slip = SlipDecoder::new();
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

        let mut slip = SlipDecoder::new();
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
