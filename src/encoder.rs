use super::*;

/// SLIP encoder context
pub struct Encoder {}

impl Encoder {
    /// Creates a new encoder context
    pub fn new() -> Self {
        Self {}
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
        let mut len = sink.write(&[END])?;

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

        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_encode() {
        const EXPECTED: [u8; 2] = [0xc0, 0xc0];
        let mut output = Vec::<u8>::new();

        let mut slip = Encoder::new();
        let len = slip.encode(&[0; 0], &mut output).unwrap();
        assert_eq!(EXPECTED.len(), len);
        assert_eq!(&EXPECTED, output.as_slice());
    }

    #[test]
    fn encode_esc_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, ESC, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_ESC, 0x03, 0xc0];
        let mut output = Vec::<u8>::new();

        let mut slip = Encoder::new();
        let len = slip.encode(&INPUT, &mut output).unwrap();
        assert_eq!(EXPECTED.len(), len);
        assert_eq!(&EXPECTED, output.as_slice());
    }

    #[test]
    fn encode_end_esc_sequence() {
        const INPUT: [u8; 3] = [0x01, END, 0x03];
        const EXPECTED: [u8; 6] = [0xc0, 0x01, ESC, ESC_END, 0x03, 0xc0];
        let mut output = Vec::<u8>::new();

        let mut slip = Encoder::new();
        let len = slip.encode(&INPUT, &mut output).unwrap();
        assert_eq!(EXPECTED.len(), len);
        assert_eq!(&EXPECTED, output.as_slice());
    }
}
