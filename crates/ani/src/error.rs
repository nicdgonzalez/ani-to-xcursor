use std::{error, fmt, io};

#[derive(Debug)]
pub enum ParseError {
    ReadFailed { source: io::Error },
    InvalidSignature,
    NotEnoughBytes { needed: usize },
    InvalidIdentifier { identifier: Vec<u8> },
    MissingRequiredChunk { identifier: &'static str },
    InvalidHeader,
    InvalidIconDir,
    InvalidIconDirEntry { source: io::Error },
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::ReadFailed { ref source } | Self::InvalidIconDirEntry { ref source } => {
                Some(source)
            }

            Self::InvalidSignature
            | Self::NotEnoughBytes { .. }
            | Self::InvalidIdentifier { .. }
            | Self::MissingRequiredChunk { .. }
            | Self::InvalidHeader
            | Self::InvalidIconDir => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ReadFailed { .. } => "failed to read file".fmt(f),
            Self::InvalidSignature => "invalid file signature (A.K.A. magic number)".fmt(f),
            Self::NotEnoughBytes { needed } => {
                write!(f, "not enough bytes (expected {needed} more bytes)")
            }
            Self::InvalidIdentifier { ref identifier } => {
                write!(
                    f,
                    "invalid chunk identifier: {}",
                    String::from_utf8_lossy(identifier)
                )
            }
            Self::MissingRequiredChunk { identifier } => {
                write!(f, "missing required chunk: {identifier}")
            }
            Self::InvalidHeader => "chunk 'anih' must be 36 bytes".fmt(f),
            Self::InvalidIconDir => "invalid frame header".fmt(f),
            Self::InvalidIconDirEntry { .. } => "invalid frame entry".fmt(f),
        }
    }
}
