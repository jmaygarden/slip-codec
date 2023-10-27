slip-codec — SLIP Encoder/Decoder
=================================

[![crates.io][crates-badge]][crates-url]
[![docs.rs][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

SLIP encoder/decoder with Rust [std::io](https://doc.rust-lang.org/std/io/index.html)::{[Read](https://doc.rust-lang.org/std/io/trait.Read.html), [Write](https://doc.rust-lang.org/std/io/trait.Write.html)} interfaces.

Pure Rust implementation of [RFC 1055](https://tools.ietf.org/html/rfc1055) Serial Line Internet Protocol (SLIP). Test cases are lifted from the [serial_line_ip](https://crates.io/crates/serial-line-ip) crate that serves the same role, but uses slices for data handling.

## Optional features

Asynchronous interfaces are optionally provided in addition to the default synchronous interface.

* **`async-codec`** — Implements runtime agnostic [asynchronous_codec](https://crates.io/crates/asynchronous-codec) traits
* **`tokio-codec`** — Implements [tokio](https://tokio.rs) runtime [tokio_util::codec](https://docs.rs/tokio-util/latest/tokio_util/codec/index.html) traits

[crates-badge]: https://img.shields.io/crates/v/slip-codec.svg
[crates-url]: https://crates.io/crates/slip-codec
[docs-badge]: https://docs.rs/slip-codec/badge.svg
[docs-url]: https://docs.rs/slip-codec
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
