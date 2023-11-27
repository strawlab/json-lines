# json-lines

[![Crates.io](https://img.shields.io/crates/v/json-lines.svg)](https://crates.io/crates/json-lines)
[![Documentation](https://docs.rs/json-lines/badge.svg)](https://docs.rs/json-lines/)
[![Crate License](https://img.shields.io/crates/l/json-lines.svg)](https://crates.io/crates/json-lines)
[![Dependency status](https://deps.rs/repo/github/strawlab/json-lines/status.svg)](https://deps.rs/repo/github/strawlab/json-lines)
[![build](https://github.com/strawlab/json-lines/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/strawlab/json-lines/actions?query=branch%3Amain)

json-lines is a `#![no_std]` + serde compatible message library implementing the
JSON Lines format.

# High-level overview

The crate provides a Rust-language implementation of JSON Lines (JSONL),
also known as Newline-Delimited JSON (NDJSON).

The wikipedia page [JSON
Streaming](https://en.wikipedia.org/wiki/JSON_streaming) and
[jsonlines.org](https://jsonlines.org) are good resources describing the
format.

This crate endeavors to have a similar API to the
[postcard](https://crates.io/crates/postcard) crate. This way, Rust code can
easily switch between JSONL and postcard formats depending on requirements.
JSONL is "self-describing" but less efficient, whereas postcard is very compact
but requires an out-of-band knowledge of message structure.

The crate contains a `#![no_std]` implementation for use in, e.g. embedded
contexts. The `std` feature is enabled by default and provides things such as
the [Error] enum implements the [std::error::Error] Trait.

The `codec` feature enables compilation of [crate::codec::JsonLinesCodec], which
provides an implementation of [tokio_util::codec::Decoder] and
[tokio_util::codec::Encoder].

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
