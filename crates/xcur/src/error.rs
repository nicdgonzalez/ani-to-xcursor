use std::{error, fmt, io};

/// An error occurred while parsing the data.
#[derive(Debug)]
pub enum ParseError {
    /// An I/O-related error, such as failure to read bytes from a file
    Io {
        /// The underlying failure that caused the error
        source: io::Error,
    },
    /// Data is not in the expected format
    InvalidSignature,
    /// Expected to read more bytes than were available
    NotEnoughBytes {
        /// Number of additional bytes expected
        needed: usize,
    },
    ImageSize,
    InvalidHotspot {
        image_size: u16,
        hotspot: u16,
    },
    CommentLength,
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::Io { ref source } => Some(source),
            Self::InvalidSignature
            | Self::NotEnoughBytes { needed: _ }
            | Self::ImageSize
            | Self::InvalidHotspot {
                image_size: _,
                hotspot: _,
            }
            | Self::CommentLength => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Io { source: _ } => "I/O error".fmt(f),
            Self::InvalidSignature => "invalid file signature (A.K.A. magic number)".fmt(f),
            Self::NotEnoughBytes { needed } => {
                write!(f, "not enough bytes (needed {needed} more bytes)")
            }
            Self::ImageSize => "image larger than 32_767x32_767".fmt(f),
            Self::InvalidHotspot {
                image_size: source,
                hotspot: target,
            } => write!(
                f,
                "hotspot must be less than or equal to image width ({target} > {source})"
            ),
            Self::CommentLength => "comment exceeds 4_294_967_295 characters".fmt(f),
        }
    }
}
