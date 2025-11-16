//! Decode Windows animated cursors.

#![allow(dead_code)]

mod error;
mod header;
mod metadata;
mod parser;

use std::path::Path;
use std::{fs, io, mem};

use error::DecodeError;
use header::Header;
use ico::IconImage;
use metadata::Metadata;
use parser::Parser;
use tracing::debug;

use crate::de::parser::Identifier;

/// The unit of measurement for a frame's display rate.
pub const JIFFY: f32 = 1000.0 / 60.0;

/// Represents the contents of an ANI file.
pub struct Ani {
    metadata: Option<Metadata>,
    header: Header,
    rates: Option<Vec<u32>>,
    sequence: Option<Vec<u32>>,
    frames: Vec<Vec<IconImage>>,
}

impl Ani {
    /// Read and decode an ANI file.
    ///
    /// # Panics
    ///
    /// This function panics on architectures where `usize` is smaller than a `u32`.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Cannot read the file at path.
    /// - Data does not follow the ANI file format specification.
    pub fn open(path: &Path, strict: bool) -> Result<Self, DecodeError> {
        let data = fs::read(path).map_err(|err| DecodeError::ReadFailure { source: err })?;

        if strict {
            Self::from_bytes_strict(&data)
        } else {
            Self::from_bytes(&data)
        }
    }

    /// Decode ANI data.
    ///
    /// This function expects the data to be structured according to the ANI file format
    /// specification. If you are not sure whether the data is structured properly, use
    /// [`Self::from_bytes`] instead.
    ///
    /// # Panics
    ///
    /// This function panics on architectures where `usize` is smaller than a `u32`.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Data has an invalid file signature.
    /// - Data does not follow the ANI file format specification.
    pub fn from_bytes_strict(data: &[u8]) -> Result<Self, DecodeError> {
        let mut parser = Parser::new(data);
        validate_signature(&mut parser)?;

        let metadata = match parser.expect_identifier(*b"LIST") {
            Ok(()) => parser
                .read_size()
                .and_then(|_| parser.expect_identifier(*b"INFO"))
                .and_then(|()| parse_info_chunk(&mut parser))
                .map(Some)?,
            Err(DecodeError::UnexpectedIdentifier { .. }) => None,
            Err(err) => return Err(err),
        };

        let header = parser
            .expect_identifier(*b"anih")
            .and_then(|()| parse_anih_chunk(&mut parser))?;

        let rates = match parser.expect_identifier(*b"rate") {
            Ok(()) => parse_rate_chunk(&mut parser).map(Some)?,
            Err(DecodeError::UnexpectedIdentifier { .. }) => None,
            Err(err) => return Err(err),
        };

        let sequence = match parser.expect_identifier(*b"seq ") {
            Ok(()) => parse_seq_chunk(&mut parser).map(Some)?,
            Err(DecodeError::UnexpectedIdentifier { .. }) => None,
            Err(err) => return Err(err),
        };

        let frames = parser
            .expect_identifier(*b"LIST")
            .and_then(|()| parser.read_size())
            .and_then(|_| parser.expect_identifier(*b"fram"))
            .and_then(|()| parse_fram_chunk(&mut parser, header.frames()))?;

        Ok(Self {
            metadata,
            header,
            rates,
            sequence,
            frames,
        })
    }

    /// Decode ANI data.
    ///
    /// This function does its best to parse the data, whether the chunks are the proper order
    /// or not. If you know that the data is structured correctly, you can use
    /// [`Self::from_bytes_strict`] instead.
    ///
    /// # Panics
    ///
    /// This function panics on architectures where `usize` is smaller than a `u32`.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Data has an invalid file signature.
    /// - Data does not follow the ANI file format specification.
    pub fn from_bytes(data: &[u8]) -> Result<Self, DecodeError> {
        #[derive(PartialEq, Eq)]
        enum Kind {
            Metadata,
            Header,
            Rate,
            Sequence,
            Frames,
        }

        struct Chunk {
            kind: Kind,
            data: Vec<u8>,
        }

        let mut parser = Parser::new(data);
        validate_signature(&mut parser)?;
        let mut chunks = Vec::<Chunk>::new();

        while parser.bytes_remaining() > 0 {
            if parser.bytes_remaining() == 1 {
                // TODO: Padding byte maybe?
                // https://en.wikipedia.org/wiki/Resource_Interchange_File_Format#Explanation
                _ = parser.read_bytes(1);
                continue;
            }

            let identifier = parser.read::<Identifier>()?;
            debug!("identifier: {:?}", String::from_utf8_lossy(&identifier));
            debug!("bytes remaining: {}", parser.bytes_remaining());

            let (kind, size) = match &identifier {
                b"LIST" => {
                    let s = parser.read_size()?;
                    let next = parser.read::<Identifier>()?;

                    match &next {
                        b"info" => (Kind::Metadata, s - 4),
                        b"fram" => (Kind::Frames, s - 4),
                        _ => return Err(DecodeError::UnknownIdentifier { actual: next }),
                    }
                }
                b"anih" => {
                    let size = parser.peek_size()?;
                    (Kind::Header, 4 + size)
                }
                b"rate" => {
                    let size = parser.peek_size()?;
                    (Kind::Rate, 4 + size)
                }
                b"seq " => {
                    let size = parser.peek_size()?;
                    (Kind::Sequence, 4 + size)
                }
                _ => return Err(DecodeError::UnknownIdentifier { actual: identifier }),
            };

            chunks.push(Chunk {
                kind,
                data: parser.read_bytes(usize::try_from(size).expect("u32 overflowed usize"))?,
            });
        }

        let metadata = if let Some(chunk) = chunks.iter().find(|c| c.kind == Kind::Metadata) {
            let mut parser = Parser::new(&chunk.data);
            Some(parse_info_chunk(&mut parser)?)
        } else {
            None
        };

        let header = chunks
            .iter()
            .find(|chunk| chunk.kind == Kind::Header)
            .ok_or(DecodeError::MissingChunk { expected: *b"anih" })
            .and_then(|chunk| {
                let mut parser = Parser::new(&chunk.data);
                parse_anih_chunk(&mut parser)
            })?;

        let rates = if let Some(chunk) = chunks.iter().find(|c| c.kind == Kind::Rate) {
            let mut parser = Parser::new(&chunk.data);
            Some(parse_rate_chunk(&mut parser)?)
        } else {
            None
        };

        let sequence = if let Some(chunk) = chunks.iter().find(|c| c.kind == Kind::Sequence) {
            let mut parser = Parser::new(&chunk.data);
            Some(parse_seq_chunk(&mut parser)?)
        } else {
            None
        };

        let frames = chunks
            .iter()
            .find(|chunk| chunk.kind == Kind::Frames)
            .ok_or(DecodeError::MissingChunk { expected: *b"fram" })
            .and_then(|chunk| {
                let mut parser = Parser::new(&chunk.data);
                parse_fram_chunk(&mut parser, header.frames())
            })?;

        Ok(Self {
            metadata,
            header,
            rates,
            sequence,
            frames,
        })
    }

    /// Additional information about the cursor (title, author).
    #[must_use]
    pub const fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Additional context for building the animated cursor.
    ///
    /// <https://en.wikipedia.org/wiki/ANI_(file_format)>
    #[must_use]
    pub const fn header(&self) -> &Header {
        &self.header
    }

    /// Display rate for each of the frames, if available.
    #[must_use]
    pub fn rates(&self) -> Option<&[u32]> {
        self.rates.as_deref()
    }

    /// Ordering of the frames, if available.
    #[must_use]
    pub fn sequence(&self) -> Option<&[u32]> {
        self.sequence.as_deref()
    }

    /// Collection of images stored within the ANI file.
    #[must_use]
    pub fn frames(&self) -> &[Vec<IconImage>] {
        &self.frames
    }
}

/// Check if the file contains a valid signature (A.K.A. magic number).
///
/// The ANI file format is based on the Resource Interchange File Format (RIFF), which is used
/// as a container for the individual frames. The first 4 bytes of a valid RIFF file should contain
/// the first chunk's identifier (always `RIFF`), followed by the chunk size (size of the ANI data),
/// followed by the ANI chunk's identifier, `ACON`.
///
/// # Panics
///
/// This function panics on architectures where `usize` is smaller than `u32`.
///
/// # Errors
///
/// This function returns an error if:
///
/// - There is not enough data remaining.
/// - The file signature is invalid.
fn validate_signature(parser: &mut Parser) -> Result<(), DecodeError> {
    parser.expect_identifier(*b"RIFF")?;
    let s = parser.read_size()?;
    let size = usize::try_from(s).expect("u32 overflowed usize");

    if parser.bytes_remaining() < size {
        return Err(DecodeError::SizeMismatch {
            expected: size,
            actual: parser.bytes_remaining(),
        });
    }

    parser.expect_identifier(*b"ACON")?;
    Ok(())
}

/// Decode the chunk containing cursor metadata.
///
/// # Panics
///
/// This function panics on architectures where `usize` is smaller than `u32`.
fn parse_info_chunk(parser: &mut Parser) -> Result<Metadata, DecodeError> {
    let title = match parser.expect_identifier(*b"INAM") {
        Ok(()) => {
            let s = parser.read_size()?;
            let size = usize::try_from(s).expect("u32 overflowed usize");
            let bytes = parser.read_bytes(size)?;
            let title = String::from_utf8_lossy(&bytes).to_string();
            Some(title)
        }
        Err(DecodeError::UnexpectedIdentifier { .. }) => None,
        Err(err) => return Err(err),
    };

    let author = match parser.expect_identifier(*b"IART") {
        Ok(()) => {
            let s = parser.read_size()?;
            let size = usize::try_from(s).expect("u32 overflowed usize");
            let bytes = parser.read_bytes(size)?;
            let author = String::from_utf8_lossy(&bytes).to_string();
            Some(author)
        }
        Err(DecodeError::UnexpectedIdentifier { .. }) => None,
        Err(err) => return Err(err),
    };

    Ok(Metadata::new(title, author))
}

/// Decode the chunk containing the ANI header.
fn parse_anih_chunk(parser: &mut Parser) -> Result<Header, DecodeError> {
    let size = parser.read_size()?;

    if size != 36 {
        return Err(DecodeError::InvalidHeaderSize { actual: size });
    }

    assert_eq!(mem::size_of::<Header>(), 36);
    let header = parser.read::<Header>()?;
    Ok(header)
}

/// Decode the chunk containing the display rate for each frame.
fn parse_rate_chunk(parser: &mut Parser) -> Result<Vec<u32>, DecodeError> {
    let s = parser.read_size()?;
    let size = usize::try_from(s).expect("u32 overflowed usize");

    if !size.is_multiple_of(mem::size_of::<u32>()) {
        return Err(DecodeError::InvalidAlignmentU32);
    }

    let rates = parser
        .read_bytes(size)?
        .chunks(4)
        // The ANI file format uses little-endian byte order for multi-byte integers.
        // <https://en.wikipedia.org/wiki/Resource_Interchange_File_Format#History>
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    Ok(rates)
}

/// Decode the chunk containing the frame ordering.
fn parse_seq_chunk(parser: &mut Parser) -> Result<Vec<u32>, DecodeError> {
    let s = parser.read_size()?;
    let size = usize::try_from(s).expect("u32 overflowed usize");

    if !size.is_multiple_of(mem::size_of::<u32>()) {
        return Err(DecodeError::InvalidAlignmentU32);
    }

    let sequence = parser
        .read_bytes(size)?
        .chunks(4)
        // The ANI file format uses little-endian byte order for multi-byte integers.
        // <https://en.wikipedia.org/wiki/Resource_Interchange_File_Format#History>
        .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    Ok(sequence)
}

/// Decode the chunk containing the frames.
fn parse_fram_chunk(
    parser: &mut Parser,
    frames_count: u32,
) -> Result<Vec<Vec<IconImage>>, DecodeError> {
    let mut frames = Vec::with_capacity(frames_count as usize);

    for _ in 0..frames_count {
        parser.expect_identifier(*b"icon")?;
        let s = parser.read_size()?;
        let size = usize::try_from(s).expect("u32 overflowed usize");

        let buffer = parser.read_bytes(size)?;
        let reader = io::Cursor::new(&buffer);

        let icon_dir = ico::IconDir::read(reader).expect("todo");
        let mut images = Vec::with_capacity(icon_dir.entries().len());

        for entry in icon_dir.entries() {
            let image = entry.decode().expect("todo");
            images.push(image);
        }

        frames.push(images);
    }

    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use header::Flag;

    #[test]
    fn signature() {
        let data = b"RIFF\x04\0\0\0ACON";
        let mut parser = Parser::new(data);
        validate_signature(&mut parser).expect("expected hardcoded bytes to be valid");
    }

    #[test]
    fn metadata_chunk() {
        let data = b"INAM\x1E\0\0\0Default - Hoshimachi Suisei v1IART\x09\0\0\0Hoshiyomi";
        let mut parser = Parser::new(data);
        let metadata = parse_info_chunk(&mut parser).expect("expected hardcoded bytes to be valid");

        assert_eq!(metadata.title(), Some("Default - Hoshimachi Suisei v1"));
        assert_eq!(metadata.author(), Some("Hoshiyomi"));
    }

    #[test]
    fn header_chunk() {
        let data = [
            36, 0, 0, 0, // Chunk size
            36, 0, 0, 0, // Header size
            9, 0, 0, 0, // Frames
            21, 0, 0, 0, // Steps
            0, 0, 0, 0, // Reserved
            0, 0, 0, 0, // Reserved
            0, 0, 0, 0, // Reserved
            0, 0, 0, 0, // Reserved
            6, 0, 0, 0, // JIF rate
            3, 0, 0, 0, // Flags
        ];
        let mut parser = Parser::new(&data);
        let header = parse_anih_chunk(&mut parser).expect("expected hardcoded bytes to be valid");

        assert_eq!(header.size(), 36);
        assert_eq!(header.frames(), 9);
        assert_eq!(header.steps(), 21);
        assert_eq!(header.jif_rate(), 6);
        assert!(header.flags().contains(Flag::ICON));
        assert!(header.flags().contains(Flag::SEQUENCE));
    }
}
