use std::{mem, ptr};

use crate::de::error::DecodeError;

pub const IDENTIFIER_SIZE: usize = 4;

pub type Identifier = [u8; IDENTIFIER_SIZE];

/// Represents an ongoing parse.
pub struct Parser<'a> {
    data: &'a [u8],
}

impl<'a> Parser<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl Parser<'_> {
    pub const fn bytes_remaining(&self) -> usize {
        self.data.len()
    }

    /// Return the next `size` bytes.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - There are not enough bytes to fill a buffer of size `size`.
    pub fn read_bytes(&mut self, size: usize) -> Result<Vec<u8>, DecodeError> {
        let (result, data) =
            self.data
                .split_at_checked(size)
                .ok_or_else(|| DecodeError::NotEnoughBytes {
                    needed: size.saturating_sub(self.data.len()),
                })?;

        self.data = data;
        Ok(result.to_vec())
    }

    /// Return the next `size` bytes without advancing.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - There are not enough bytes to fill a buffer of size `size`.
    pub fn peek_bytes(&mut self, size: usize) -> Result<Vec<u8>, DecodeError> {
        let (result, _) =
            self.data
                .split_at_checked(size)
                .ok_or_else(|| DecodeError::NotEnoughBytes {
                    needed: size.saturating_sub(self.data.len()),
                })?;

        Ok(result.to_vec())
    }

    pub fn read<T>(&mut self) -> Result<T, DecodeError>
    where
        T: Copy,
    {
        let size = mem::size_of::<T>();
        let (result, data) =
            self.data
                .split_at_checked(size)
                .ok_or_else(|| DecodeError::NotEnoughBytes {
                    needed: size.saturating_sub(self.data.len()),
                })?;

        // SAFETY: This cast is safe under the following conditions:
        //
        // - Size of the buffer is equal to the size of type `T`.
        // - Pointer to the buffer is aligned for a value of size `T`.
        let value = unsafe { ptr::read_unaligned(result.as_ptr().cast()) };

        self.data = data;
        Ok(value)
    }

    pub fn expect_identifier(&mut self, expected: Identifier) -> Result<(), DecodeError> {
        let (result, data) = self.data.split_at_checked(IDENTIFIER_SIZE).ok_or_else(|| {
            DecodeError::NotEnoughBytes {
                needed: IDENTIFIER_SIZE.saturating_sub(self.data.len()),
            }
        })?;

        if result != expected {
            return Err(DecodeError::UnexpectedIdentifier {
                expected,
                actual: (*result).try_into().unwrap(),
            });
        }

        self.data = data;
        Ok(())
    }

    pub fn read_size(&mut self) -> Result<u32, DecodeError> {
        let size = mem::size_of::<u32>();
        let (result, data) =
            self.data
                .split_at_checked(size)
                .ok_or_else(|| DecodeError::NotEnoughBytes {
                    needed: size.saturating_sub(self.data.len()),
                })?;

        // The ANI file format is based on the RIFF file format, which utilizes little-endian
        // byte order for multi-byte integers.
        //
        // <https://en.wikipedia.org/wiki/Resource_Interchange_File_Format#History>
        let value = u32::from_le_bytes(result.try_into().unwrap());

        self.data = data;
        Ok(value)
    }

    pub fn peek_size(&mut self) -> Result<u32, DecodeError> {
        let size = mem::size_of::<u32>();
        let (result, _) =
            self.data
                .split_at_checked(size)
                .ok_or_else(|| DecodeError::NotEnoughBytes {
                    needed: size.saturating_sub(self.data.len()),
                })?;

        // The ANI file format is based on the RIFF file format, which utilizes little-endian
        // byte order for multi-byte integers.
        //
        // <https://en.wikipedia.org/wiki/Resource_Interchange_File_Format#History>
        let value = u32::from_le_bytes(result.try_into().unwrap());

        Ok(value)
    }
}
