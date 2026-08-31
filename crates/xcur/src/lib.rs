//! # Xcursor
//!
//! Parser for the Xcursor file format.

#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

mod comment;
mod error;
mod image;
mod parser;

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::mem;
use std::path::Path;

use crate::comment::COMMENT_HEADER_SIZE;
pub use crate::comment::{Comment, CommentKind};
use crate::error::ParseError;
use crate::image::IMAGE_HEADER_SIZE;
pub use crate::image::Image;
use crate::parser::Parser;

pub(crate) const CARD32_SIZE: usize = mem::size_of::<u32>();

const FILE_SIGNATURE: &[u8; 4] = b"Xcur";
const FILE_HEADER_SIZE: u32 = 16;
const FILE_VERSION: u32 = 0x0001_0000;

const ENTRY_SIZE: usize = mem::size_of::<Entry>();

fn write_card32<W>(writer: &mut W, value: u32) -> io::Result<()>
where
    W: Write,
{
    writer.write_all(value.to_le_bytes().as_ref())
}

#[derive(Debug, Clone)]
pub struct Xcursor {
    images: Vec<Image>,
    comments: Vec<Comment>,
}

impl Xcursor {
    #[must_use]
    pub fn new(images: Vec<Image>, comments: Vec<Comment>) -> Self {
        Self { images, comments }
    }

    /// Read the file at path and decode it.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Failed to read file due to I/O errors (e.g., lacking permissions)
    /// - Data is malformed
    pub fn open<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        File::open(path)
            .map_err(|err| ParseError::Io { source: err })
            .and_then(Self::from_reader)
    }

    /// Read and decode the data provided by reader.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Failed to read data due to I/O errors
    /// - Data is malformed
    pub fn from_reader<R>(mut reader: R) -> Result<Self, ParseError>
    where
        R: Read,
    {
        let mut buffer = Vec::new();
        _ = reader
            .read_to_end(&mut buffer)
            .map_err(|err| ParseError::Io { source: err })?;

        Self::try_from(buffer.as_slice())
    }

    /// Decode the data provided.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Data is malformed
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, ParseError> {
        Self::try_from(buffer)
    }

    pub fn write<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: Write,
    {
        let entry_count = self.images.len() + self.comments.len();
        let entry_count = u32::try_from(entry_count).expect("usize overflowed u32");

        writer.write_all(FILE_SIGNATURE.as_slice())?;
        write_card32(&mut writer, FILE_HEADER_SIZE)?;
        write_card32(&mut writer, FILE_VERSION)?;
        write_card32(&mut writer, entry_count)?;

        let entry_size = u32::try_from(ENTRY_SIZE).expect("u32 overflowed usize");
        let mut offset = 16 + (entry_size * entry_count); // Start from the end of the Table of Contents.

        // Construct the Table of Contents.
        for entry in &self.images {
            write_card32(&mut writer, Type::Image as u32)?;
            write_card32(&mut writer, u32::from(entry.width()))?;
            write_card32(&mut writer, offset)?;

            let image_size = entry.argb().len() * CARD32_SIZE;
            let image_size = u32::try_from(image_size).expect("u32 overflowed usize");

            offset += IMAGE_HEADER_SIZE + image_size;
        }

        for entry in &self.comments {
            write_card32(&mut writer, Type::Comment as u32)?;
            write_card32(&mut writer, entry.kind() as u32)?;
            write_card32(&mut writer, offset)?;

            let comment_size = u32::try_from(entry.buffer().len()).expect("u32 overflowed usize");
            offset += COMMENT_HEADER_SIZE + comment_size;
        }

        // Write chunks in same order as Table of Contents (i.e., images first, then comments).
        for entry in &self.images {
            entry.write(&mut writer)?;
        }

        for entry in &self.comments {
            entry.write(&mut writer)?;
        }

        Ok(())
    }

    pub fn save<P>(&self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let mut buffer = Vec::new();
        self.write(&mut buffer)?;

        fs::write(path.as_ref(), &buffer)
    }

    #[must_use]
    pub fn images(&self) -> &[Image] {
        &self.images
    }

    #[must_use]
    pub fn comments(&self) -> &[Comment] {
        &self.comments
    }
}

// Represents an entry in the Table of Contents.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    kind: EntryKind,
    position: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum EntryKind {
    Image(u32),
    Comment(CommentKind),
}

/// Represents the available chunk types.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Image = 0xFFFD_0002,
    Comment = 0xFFFE_0001,
}

impl TryFrom<u32> for Type {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0xFFFD_0002 => Ok(Self::Image),
            0xFFFE_0001 => Ok(Self::Comment),
            n => Err(n),
        }
    }
}

impl TryFrom<&[u8]> for Xcursor {
    type Error = ParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut parser = Parser::new(value);

        let signature = parser.read_bytes(4)?;

        if signature != b"Xcur" {
            return Err(ParseError::InvalidSignature);
        }

        let _header_size = parser.read_card32()?;
        let _version = parser.read_card32()?;

        let entry_count = parser.read_card32()?;
        let entry_count_usize = usize::try_from(entry_count).expect("u32 overflowed usize");
        let table_of_contents_size = ENTRY_SIZE * entry_count_usize;

        let table_of_contents = parser
            .read_bytes(table_of_contents_size)?
            .as_chunks::<ENTRY_SIZE>()
            .0 // Skip remainder because we know there is none.
            .iter()
            .filter_map(parse_entry)
            .collect::<Vec<_>>();

        let images = table_of_contents
            .iter()
            .filter(|entry| matches!(entry.kind, EntryKind::Image(_)))
            .map(|entry| Image::from_chunk(value, entry))
            .collect::<Result<Vec<_>, ParseError>>()?;

        let comments = table_of_contents
            .iter()
            .filter_map(|entry| match entry.kind {
                EntryKind::Image(_) => None,
                EntryKind::Comment(kind) => Some((entry.position, kind)),
            })
            .map(|(position, kind)| Comment::from_chunk(value, position, kind))
            .collect::<Vec<_>>();

        Ok(Xcursor { images, comments })
    }
}

fn parse_entry(entry: &[u8; ENTRY_SIZE]) -> Option<Entry> {
    let mut fields = entry
        .as_chunks::<CARD32_SIZE>()
        .0 // Skip remainder because we know there is none.
        .iter()
        .copied()
        .map(u32::from_le_bytes);

    let raw_type = fields.next().unwrap();
    let raw_subtype = fields.next().unwrap();
    let position = fields.next().unwrap();

    let kind = match Type::try_from(raw_type)
        .inspect_err(|n| tracing::warn!("unknown entry type: {n}"))
        .ok()?
    {
        Type::Image => EntryKind::Image(raw_subtype),
        Type::Comment => CommentKind::try_from(raw_subtype)
            .inspect_err(|n| tracing::warn!("unknown comment entry subtype: {n}"))
            .map(EntryKind::Comment)
            .ok()?,
    };

    Some(Entry { kind, position })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_images() {
        let buffer = &[
            b'X', b'c', b'u', b'r', // File signature
            16, 0, 0, 0, // Header size
            0, 0, 1, 0, // Version
            0, 0, 0, 0, // Entry count for the Table of Contents
        ];

        let cursor = Xcursor::from_bytes(buffer).expect("failed to construct cursor");
        assert_eq!(cursor.images.len(), 0);
        assert_eq!(cursor.comments.len(), 0);
    }
}
