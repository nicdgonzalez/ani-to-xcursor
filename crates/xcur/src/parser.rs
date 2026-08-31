use crate::error::ParseError;

/// Represents an on-going parse over an Xcursor-formatted buffer.
///
/// This parser provides a zero-copy, forward-only reading over a byte slice. All reads advance
/// an internal cursor and return borrowed data tied to the original buffer's lifetime.
pub struct Parser<'a> {
    buffer: &'a [u8],
}

impl<'a> Parser<'a> {
    /// Construct a new parser over the provided buffer.
    #[must_use]
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    /// Read `size` bytes from the buffer.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Fewer than `size` bytes remain in the buffer
    pub fn read_bytes(&mut self, size: usize) -> Result<&'a [u8], ParseError> {
        let (bytes, remainder) =
            self.buffer
                .split_at_checked(size)
                .ok_or_else(|| ParseError::NotEnoughBytes {
                    needed: size - self.buffer.len(),
                })?;

        self.buffer = remainder;
        Ok(bytes)
    }

    pub fn read_card32(&mut self) -> Result<u32, ParseError> {
        let bytes = self.read_bytes(4)?;
        Ok(bytes.try_into().map(u32::from_le_bytes).unwrap())
    }
}
