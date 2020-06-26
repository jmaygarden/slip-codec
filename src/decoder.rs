use super::*;
use std::io::Read;

#[derive(Debug)]
pub enum Error {
    FramingError,
    OversizedPacket,
    ReadError(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::ReadError(err)
    }
}

pub type Result = std::result::Result<Vec<u8>, self::Error>;

enum State {
    Normal,
    Error,
    Escape,
}

/// SLIP decoding context
pub struct Decoder {
    buffer: Vec<u8>,
    state: State,
}

impl Decoder {
    /// Creates a new context with the given maximum buffer size.
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            state: State::Normal,
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
    pub fn decode<T>(self: &mut Self, source: &mut T) -> self::Result
    where
        T: Read,
    {
        for value in source.bytes() {
            let value = value?;

            if self.buffer.len() == self.buffer.capacity() {
                self.buffer.clear();
                self.state = State::Error;

                return Err(Error::OversizedPacket);
            }

            match self.state {
                State::Normal => match value {
                    END => {
                        if !self.buffer.is_empty() {
                            let mut buffer = Vec::<u8>::new();

                            buffer.extend(self.buffer.iter().clone());
                            self.buffer.clear();

                            return Ok(buffer);
                        }
                    }
                    ESC => {
                        self.state = State::Escape;
                    }
                    _ => {
                        self.buffer.push(value);
                    }
                },
                State::Error => {
                    if value == END {
                        self.state = State::Normal;
                    }
                }
                State::Escape => match value {
                    ESC_END => {
                        self.buffer.push(END);
                        self.state = State::Normal;
                    }
                    ESC_ESC => {
                        self.buffer.push(ESC);
                        self.state = State::Normal;
                    }
                    _ => {
                        self.buffer.clear();
                        self.state = State::Error;

                        return Err(Error::FramingError);
                    }
                },
            }
        }

        Ok(Vec::<u8>::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_decode() {
        const INPUT: [u8; 2] = [0xc0, 0xc0];

        let mut slip = Decoder::new(32);
        let res = slip.decode(&mut INPUT.as_ref());
        assert!(res.is_ok());
        let buf = res.unwrap();
        assert!(buf.is_empty());
    }

    #[test]
    fn simple_decode() {
        const INPUT: [u8; 7] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05, 0xc0];
        const DATA: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];

        let mut slip = Decoder::new(32);
        let buf = slip.decode(&mut INPUT.as_ref()).unwrap();
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA, buf.as_slice());
    }

    /// Ensure that [ESC, ESC_END] -> [END]
    #[test]
    fn decode_esc_then_esc_end_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdc, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xc0, 0x03];

        let mut slip = Decoder::new(200);
        let buf = slip.decode(&mut INPUT.as_ref()).unwrap();
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA, buf.as_slice());
    }

    /// Ensure that [ESC, ESC_ESC] -> [ESC]
    #[test]
    fn decode_esc_then_esc_esc_sequence() {
        const INPUT: [u8; 6] = [0xc0, 0x01, 0xdb, 0xdd, 0x03, 0xc0];
        const DATA: [u8; 3] = [0x01, 0xdb, 0x03];

        let mut slip = Decoder::new(200);
        let buf = slip.decode(&mut INPUT.as_ref()).unwrap();
        assert_eq!(DATA.len(), buf.len());
        assert_eq!(&DATA, buf.as_slice());
    }

    #[test]
    fn multi_part_decode() {
        const INPUT_1: [u8; 6] = [0xc0, 0x01, 0x02, 0x03, 0x04, 0x05];
        const INPUT_2: [u8; 6] = [0x05, 0x06, 0x07, 0x08, 0x09, 0xc0];
        const DATA: [u8; 10] = [0x01, 0x02, 0x03, 0x04, 0x05, 0x05, 0x06, 0x07, 0x08, 0x09];

        let mut slip = Decoder::new(200);
        {
            let buf = slip.decode(&mut INPUT_1.as_ref()).unwrap();
            assert!(buf.is_empty());
        }
        {
            let buf = slip.decode(&mut INPUT_2.as_ref()).unwrap();
            assert_eq!(DATA.len(), buf.len());
            assert_eq!(&DATA, buf.as_slice());
        }
    }
}
