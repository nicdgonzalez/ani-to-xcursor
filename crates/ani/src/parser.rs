use crate::error::ParseError;

/// Length of a chunk identifier.
const IDENTIFIER_LENGTH: usize = 4;

/// Represents an on-going parse over an ANI-formatted buffer.
///
/// This parser provides a zero-copy, forward-only reading over a byte slice. All reads advance
/// an internal cursor and return borrowed data tied to the original buffer's lifetime.
pub struct Parser<'a> {
    buffer: &'a [u8],
}

impl<'a> Parser<'a> {
    /// Construct a new parser over the provided byte buffer.
    #[must_use]
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    /// Returns `true` if the entire buffer has been consumed.
    #[must_use]
    pub fn finished(&self) -> bool {
        self.buffer.is_empty()
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

    /// Read the next chunk identifier (4 bytes) from the buffer.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Fewer than 4 bytes remain in the buffer
    pub fn read_identifier(&mut self) -> Result<Identifier<'a>, ParseError> {
        self.read_bytes(IDENTIFIER_LENGTH).map(Identifier::from)
    }

    /// Read the next chunk size (4 bytes) from the buffer.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Fewer than 4 bytes remain in the buffer
    pub fn read_size(&mut self) -> Result<u32, ParseError> {
        #[expect(clippy::missing_panics_doc, reason = "unreachable panic")]
        self.read_bytes(std::mem::size_of::<u32>())
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// Read `size` bytes from the buffer.
    ///
    /// This function is similar to [`Self::read_bytes`], but discards the extra pad byte when
    /// `size` is odd.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Fewer than `size` (or `size` + 1) bytes remain in the buffer
    ///
    /// # Panics
    ///
    /// This function panics on architectures where `usize` is smaller than a `u32`.
    pub fn read_data(&mut self, size: u32) -> Result<Data<'a>, ParseError> {
        let size = usize::try_from(size).expect("expected u32 to fit within a usize");

        // If the chunk size is odd, there is an additional pad byte at the end of the data.
        let read_size = size.next_multiple_of(2);
        assert!(size <= read_size, "read size overflowed");

        // Discard the pad byte, if it exists.
        //
        // SAFETY: `size` is always less than or equal to the number of bytes read.
        let (data, _) = unsafe { self.read_bytes(read_size)?.split_at_unchecked(size) };

        Ok(Data { inner: data })
    }

    /// Read a chunk's identifier, size, and data.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Not enough bytes to read the chunk header (identifier + chunk size)
    /// - Chunk header requests more bytes than are available
    pub fn read_chunk(&mut self) -> Result<Chunk<'a>, ParseError> {
        let identifier = self.read_identifier()?;
        let size = self.read_size()?;
        let data = self.read_data(size)?;
        Ok(Chunk {
            identifier,
            size,
            data,
        })
    }
}

impl<'a> IntoIterator for Parser<'a> {
    type Item = Result<Chunk<'a>, ParseError>;
    type IntoIter = ChunkIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ChunkIter { parser: self }
    }
}

pub struct ChunkIter<'a> {
    parser: Parser<'a>,
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = Result<Chunk<'a>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.parser.finished() {
            None
        } else {
            Some(self.parser.read_chunk())
        }
    }
}

#[derive(Debug)]
pub struct Chunk<'a> {
    pub identifier: Identifier<'a>,
    pub size: u32,
    pub data: Data<'a>,
}

#[derive(Debug)]
pub struct Identifier<'a> {
    inner: &'a [u8],
}

impl<'a> From<&'a [u8]> for Identifier<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self { inner: value }
    }
}

impl Identifier<'_> {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.inner
    }

    pub fn as_str(&self) -> Result<&str, ParseError> {
        str::from_utf8(self.inner).map_err(|_| ParseError::InvalidIdentifier {
            identifier: self.inner.to_vec(),
        })
    }
}

#[derive(Debug)]
pub struct Data<'a> {
    inner: &'a [u8],
}

impl Data<'_> {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.inner
    }
}

impl<'a> Data<'a> {
    pub fn as_subchunk(&self) -> Result<Chunk<'a>, ParseError> {
        self.inner
            .split_at_checked(IDENTIFIER_LENGTH)
            .map(|(identifier, data)| Chunk {
                identifier: Identifier { inner: identifier },
                size: data
                    .len()
                    .try_into()
                    .expect("length of data to fit within u32"),
                data: Data { inner: data },
            })
            .ok_or_else(|| ParseError::NotEnoughBytes {
                needed: IDENTIFIER_LENGTH - self.inner.len(),
            })
    }
}
