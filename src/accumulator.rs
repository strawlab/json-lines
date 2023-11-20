//! An accumulator used to collect chunked newline-separated data and deserialize it.

// The code in this module is modified from
// [postcard](https://crates.io/crates/postcard).

use serde::Deserialize;

/// An accumulator used to collect chunked newline data and deserialize it.
///
/// This is often useful when you receive "parts" of the message at a time, for example when draining
/// a serial port buffer that may not contain an entire uninterrupted message.
///
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
pub struct NewlinesAccumulator<const N: usize> {
    buf: [u8; N],
    idx: usize,
}

/// The result of feeding the accumulator.
#[cfg_attr(feature = "use-defmt", derive(defmt::Format))]
pub enum FeedResult<'a, T> {
    /// Consumed all data, still pending.
    Consumed,

    /// Buffer was filled. Contains remaining section of input, if any.
    OverFull(&'a [u8]),

    /// Reached end of chunk, but deserialization failed. Contains remaining section of input, if.
    /// any
    DeserError(&'a [u8]),

    /// Deserialization complete. Contains deserialized data and remaining section of input, if any.
    Success {
        /// Deserialize data.
        data: T,

        /// Remaining data left in the buffer after deserializing.
        remaining: &'a [u8],
    },
}

impl<const N: usize> NewlinesAccumulator<N> {
    /// Create a new accumulator.
    pub const fn new() -> Self {
        Self {
            buf: [0; N],
            idx: 0,
        }
    }

    /// Appends data to the internal buffer and attempts to deserialize the accumulated data into
    /// `T`.
    #[inline]
    pub fn feed<'a, T>(&mut self, input: &'a [u8]) -> FeedResult<'a, T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.feed_ref(input)
    }

    /// Appends data to the internal buffer and attempts to deserialize the accumulated data into
    /// `T`.
    ///
    /// This differs from feed, as it allows the `T` to reference data within the internal buffer, but
    /// mutably borrows the accumulator for the lifetime of the deserialization.
    /// If `T` does not require the reference, the borrow of `self` ends at the end of the function.
    pub fn feed_ref<'de, 'a, T>(&'de mut self, input: &'a [u8]) -> FeedResult<'a, T>
    where
        T: Deserialize<'de>,
    {
        if input.is_empty() {
            return FeedResult::Consumed;
        }

        let newline_pos = input.iter().position(|&i| i == b'\n');

        if let Some(n) = newline_pos {
            // Yes! We have an end of message here.
            // Add one to include the newline in the "take" portion
            // of the buffer, rather than in "release".
            let (take, release) = input.split_at(n + 1);

            // Does it fit?
            if (self.idx + take.len()) <= N {
                // Aw yiss - add to array
                self.extend_unchecked(take);

                let json_buf_len = self.idx - 1; // newline is not JSON-encoded
                let retval = match crate::from_bytes::<T>(&mut self.buf[..json_buf_len]) {
                    Ok(t) => FeedResult::Success {
                        data: t,
                        remaining: release,
                    },
                    Err(_) => FeedResult::DeserError(release),
                };
                self.idx = 0;
                retval
            } else {
                self.idx = 0;
                FeedResult::OverFull(release)
            }
        } else {
            // Does it fit?
            if (self.idx + input.len()) > N {
                // nope
                let new_start = N - self.idx;
                self.idx = 0;
                FeedResult::OverFull(&input[new_start..])
            } else {
                // yup!
                self.extend_unchecked(input);
                FeedResult::Consumed
            }
        }
    }

    /// Extend the internal buffer with the given input.
    ///
    /// # Panics
    ///
    /// Will panic if the input does not fit in the internal buffer.
    fn extend_unchecked(&mut self, input: &[u8]) {
        let new_end = self.idx + input.len();
        self.buf[self.idx..new_end].copy_from_slice(input);
        self.idx = new_end;
    }
}
