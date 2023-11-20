#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#![cfg_attr(not(feature = "use-std"), no_std)]

// The code in this module is based on
// [postcard](https://crates.io/crates/postcard).

use serde::{Deserialize, Serialize};

pub mod accumulator;

#[cfg(feature = "codec")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "codec")))]
pub mod codec;

#[cfg(not(feature = "use-std"))]
extern crate core as std;

/// This is the result type used by json-lines
pub type Result<T> = std::result::Result<T, Error>;

/// This is the error type used by json-lines
#[derive(Debug)]
#[cfg_attr(feature = "use-std", derive(thiserror::Error))]
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
pub enum Error {
    /// An input-output error
    #[cfg(feature = "codec")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "codec")))]
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// A deserialization error
    #[cfg_attr(feature = "use-std", error("JSON deserialization error"))]
    DeserializeJson,
    /// A serialization error
    #[cfg_attr(feature = "use-std", error("JSON serialization error"))]
    SerializeJson,
    /// A newline character was present in the JSON data
    #[cfg_attr(feature = "use-std", error("newline in JSON data"))]
    NewlineInData,
}

/// Deserialize a message of type `T` from a byte slice.
pub fn from_bytes<'a, T>(s: &'a mut [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let (t, used) = serde_json_core::from_slice(s).map_err(|_| Error::DeserializeJson)?;
    if used == s.len() {
        Ok(t)
    } else {
        Err(Error::DeserializeJson)
    }
}

/// Serialize a `T` to the given slice.
pub fn to_slice<'a, 'b, T>(value: &'b T, buf: &'a mut [u8]) -> Result<&'a mut [u8]>
where
    T: Serialize + ?Sized,
{
    let nbytes = serde_json_core::to_slice(&value, buf).map_err(|_| Error::SerializeJson)?;
    let encoded = &mut buf[..nbytes];

    if encoded.iter().position(|&i| i == b'\n').is_some() {
        return Err(Error::NewlineInData);
    }
    Ok(encoded)
}

/// Serialize a `T` to the given slice. The terminating newline is included in
/// the output buffer.
pub fn to_slice_newline<'a, 'b, T>(value: &'b T, buf: &'a mut [u8]) -> Result<&'a mut [u8]>
where
    T: Serialize + ?Sized,
{
    let encoded = to_slice(value, buf)?;
    let nbytes = encoded.len();

    let (used, _drop) = buf.split_at_mut(nbytes + 1);
    used[nbytes] = b'\n';
    Ok(used)
}
