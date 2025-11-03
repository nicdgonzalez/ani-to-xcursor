use std::{error, fmt, io};

use crate::de::parser::Identifier;

#[non_exhaustive]
#[derive(Debug)]
pub enum DecodeError {
    /// An error occurred while attempting to read from a file.
    ReadFailure {
        /// The underlying error that caused the failure.
        source: io::Error,
    },

    /// Attempted to read more bytes than were available.
    NotEnoughBytes {
        /// The number of bytes needed to complete the operation.
        needed: usize,
    },

    /// The next chunk had a different identifier than was expected.
    UnexpectedIdentifier {
        /// The chunk identifier that was expected.
        expected: Identifier,
        /// The chunk identifier that was received.
        actual: Identifier,
    },

    /// The size of the "ACON" chunk does not match the length of the data.
    SizeMismatch {
        /// The size received for the "ACON" chunk.
        expected: usize,
        /// The real size of the "ACON" chunk.
        actual: usize,
    },

    /// The ANI header had an invalid size according to the file format specification.
    InvalidHeaderSize {
        /// The size received for the "anih" chunk.
        actual: u32,
    },

    /// The chunk size indicates the value is not properly aligned for `u32`s.
    InvalidAlignmentU32,

    MissingChunk {
        expected: Identifier,
    },
}

impl error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::ReadFailure { ref source } => Some(source),
            Self::NotEnoughBytes { .. }
            | Self::UnexpectedIdentifier { .. }
            | Self::SizeMismatch { .. }
            | Self::InvalidHeaderSize { .. }
            | Self::InvalidAlignmentU32
            | Self::MissingChunk { .. } => None,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ReadFailure { .. } => "failed to read ANI file".fmt(f),
            Self::NotEnoughBytes { needed } => {
                write!(f, "not enough data (needed {needed} additional bytes)")
            }
            Self::UnexpectedIdentifier { expected, actual } => {
                let expected = String::from_utf8_lossy(&expected).to_string();
                let actual = String::from_utf8_lossy(&actual).to_string();
                write!(f, "expected chunk identifier {expected:?}, got {actual:?}")
            }
            Self::SizeMismatch { expected, actual } => {
                write!(f, "expected chunk to be {expected} bytes, got {actual}")
            }
            Self::InvalidHeaderSize { actual } => {
                write!(f, "expected the 'anih' chunk to be 36 bytes, got {actual}")
            }
            Self::InvalidAlignmentU32 => {
                "expected chunk size to be properly aligned for u32".fmt(f)
            }
            Self::MissingChunk { expected } => {
                write!(f, "chunk not found: {expected:?}")
            }
        }
    }
}
