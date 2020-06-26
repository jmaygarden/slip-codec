# slip-codec

SLIP encoder/decoder with Rust std::io::{Read, Write} interfaces.

Pure Rust implementation of [RFC 1055](https://tools.ietf.org/html/rfc1055) Serial Line Internet Protocol (SLIP). Test cases are lifted from the [serial_line_ip](https://crates.io/crates/serial-line-ip) crate that serves the same role, but uses slices for data handling.
