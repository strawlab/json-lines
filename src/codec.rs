//! Provides implementations to use [tokio_util::codec]

// The implementations here do not use the no-std capable variants in the other
// modules in this crate. This should probably be fixed.

use bytes::{buf::Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{Error, Result};

/// JSON Lines text format, also called newline-delimited JSON.
pub struct JsonLinesCodec<D, S> {
    deser: std::marker::PhantomData<D>,
    ser: std::marker::PhantomData<S>,
}

impl<D, S> Default for JsonLinesCodec<D, S> {
    fn default() -> Self {
        Self {
            deser: std::marker::PhantomData,
            ser: std::marker::PhantomData,
        }
    }
}

impl<D, S> Decoder for JsonLinesCodec<D, S>
where
    D: serde::de::DeserializeOwned,
{
    type Item = D;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>> {
        if buf.is_empty() {
            Ok(None)
        } else {
            // Could investigate potential optimization in which size of
            // already-searched buffer is maintained.
            loop {
                // we loop until out of bytes or found actual JSON inside newline
                if let Some(newline_offset) = memchr::memchr(b'\n', &buf[..]) {
                    // Found a line!
                    let to_parse = &buf[..newline_offset];
                    if newline_offset == 0 {
                        buf.advance(newline_offset + 1);
                        // try again
                        continue;
                    }
                    debug_assert!(buf[newline_offset] == b'\n');
                    match serde_json::from_slice(&to_parse[..newline_offset]) {
                        Ok(msg) => {
                            buf.advance(newline_offset + 1);
                            return Ok(Some(msg));
                        }
                        Err(_e) => {
                            buf.advance(newline_offset + 1);
                            // If decode fails, we should still advance the buffer.
                            //
                            // In case of error, we want to advance our place in the buffer so that
                            // we don't attempt to re-parse this bad data again.
                            return Err(Error::DeserializeJson);
                        }
                    }
                } else {
                    // No newline, so stop trying and wait until we have more bytes.
                    return Ok(None);
                }
            }
        }
    }
}

// We encode `T` and not `&T` because we do not want to deal with
// the lifetime issues (this is used in async contexts.)
impl<D, S> Encoder<S> for JsonLinesCodec<D, S>
where
    S: serde::Serialize,
{
    type Error = Error;
    fn encode(&mut self, msg: S, final_buf: &mut BytesMut) -> Result<()> {
        let mut v = serde_json::to_vec(&msg).map_err(|_| Error::SerializeJson)?;
        if memchr::memchr(b'\n', &v).is_some() {
            return Err(Error::NewlineInData);
        }
        v.push(b'\n');
        final_buf.extend_from_slice(&v);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct MyStruct {
        val1: u8,
        val2: u8,
    }

    #[test]
    fn roundtrip() -> Result<()> {
        let msg1 = MyStruct { val1: 12, val2: 34 };
        let msg2 = MyStruct { val1: 56, val2: 78 };
        let mut bytes = BytesMut::new();
        let mut codec = JsonLinesCodec::default();
        codec.encode(msg1.clone(), &mut bytes)?;
        codec.encode(msg2.clone(), &mut bytes)?;
        let found1: Option<MyStruct> = codec.decode(&mut bytes)?;
        let found2: Option<MyStruct> = codec.decode(&mut bytes)?;
        assert_eq!(found1, Some(msg1));
        assert_eq!(found2, Some(msg2));
        Ok(())
    }
}
