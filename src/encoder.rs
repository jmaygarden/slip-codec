use crate::{END, ESC, ESC_END, ESC_ESC};

/// SLIP encoder context
pub struct SlipEncoder {
    begin_with_end: bool,
}

impl SlipEncoder {
    /// Creates a new encoder context
    pub fn new(begin_with_end: bool) -> Self {
        Self { begin_with_end }
    }

    /// Encodes the given buffer in a SLIP frame and forwards it to the sink.
    ///
    /// # Arguments
    ///
    /// * `buf` - input data buffer for encoding
    /// * `sink` - output object implementing the std::io::Write trait
    ///
    /// Returns the number of bytes written to the sink.
    ///
    pub fn encode(&mut self, buf: &[u8], sink: &mut dyn std::io::Write) -> std::io::Result<usize> {
        let mut len = if self.begin_with_end {
            sink.write(&[END])?
        } else {
            0
        };

        for value in buf.iter() {
            match *value {
                END => {
                    len += sink.write(&[ESC, ESC_END])?;
                }
                ESC => {
                    len += sink.write(&[ESC, ESC_ESC])?;
                }
                _ => {
                    len += sink.write(&[*value])?;
                }
            }
        }

        len += sink.write(&[END])?;

        sink.flush()?;

        Ok(len)
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

    #[test]
    fn empty_encode() {
        const EXPECTED: [u8; 2] = [0xc0, 0xc0];
        let mut output = Vec::<u8>::new();

        let mut slip = SlipEncoder::new(true);
        let len = slip.encode(&[0; 0], &mut output).unwrap();
        assert_eq!(EXPECTED.len(), len);
        assert_eq!(&EXPECTED, output.as_slice());
    }

    #[test]
    fn encode_esc_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, ESC, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_ESC, 0x03, 0xc0];
        let mut output = Vec::<u8>::new();

        let mut slip = SlipEncoder::new(true);
        let len = slip.encode(&INPUT, &mut output).unwrap();
        assert_eq!(EXPECTED.len(), len);
        assert_eq!(&EXPECTED, output.as_slice());
    }

    #[test]
    fn encode_end_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, END, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_END, 0x03, 0xc0];
        let mut output = Vec::<u8>::new();

        let mut slip = SlipEncoder::new(true);
        let len = slip.encode(&INPUT, &mut output).unwrap();
        assert_eq!(EXPECTED.len(), len);
        assert_eq!(&EXPECTED, output.as_slice());
    }
}
