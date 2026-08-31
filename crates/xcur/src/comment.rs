use std::io::Write;
use std::{fmt, io};

use crate::error::ParseError;
use crate::{Type, write_card32};

pub const COMMENT_HEADER_SIZE: u32 = 20;
const COMMENT_VERSION: u32 = 1;

/// Represents the different subtypes of a comment chunk.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentKind {
    Copyright = 1,
    License = 2,
    Other = 3,
}

impl TryFrom<u32> for CommentKind {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Copyright),
            2 => Ok(Self::License),
            3 => Ok(Self::Other),
            n => Err(n),
        }
    }
}

impl fmt::Display for CommentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Copyright => "Copyright".fmt(f),
            Self::License => "License".fmt(f),
            Self::Other => "Other".fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    kind: CommentKind,
    buffer: String,
}

impl Comment {
    pub fn new(kind: CommentKind, buffer: String) -> Result<Self, ParseError> {
        let max_buffer_length = usize::try_from(u32::MAX).expect("u32 overflowed usize");

        if buffer.len() > max_buffer_length {
            return Err(ParseError::CommentLength);
        }

        Ok(Self { kind, buffer })
    }

    pub(crate) fn from_chunk(buffer: &[u8], position: u32, kind: CommentKind) -> Self {
        let position = usize::try_from(position).expect("u32 overflowed usize");
        let header_size = usize::try_from(COMMENT_HEADER_SIZE).expect("u32 overflowed usize");

        let start = position;
        let end = position + header_size;
        let raw_header = &buffer[start..end];

        let mut fields = raw_header
            .as_chunks::<4>()
            .0
            .iter()
            .copied()
            .map(u32::from_le_bytes);

        let _header_size = fields.next().unwrap();
        let _raw_type = fields.next().unwrap();
        let _raw_subtype = fields.next().unwrap();
        let _version = fields.next().unwrap();

        let comment_length = fields.next().unwrap();
        let comment_length = usize::try_from(comment_length).expect("u32 overflowed usize");

        let start = end;
        let end = start + comment_length;
        let bytes = &buffer[start..end];
        let buffer = String::from_utf8_lossy(bytes).to_string();

        Comment { kind, buffer }
    }

    pub fn write<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: Write,
    {
        let buffer_length = u32::try_from(self.buffer.len()).unwrap();

        write_card32(&mut writer, COMMENT_HEADER_SIZE)?;
        write_card32(&mut writer, Type::Comment as u32)?;
        write_card32(&mut writer, self.kind as u32)?;
        write_card32(&mut writer, COMMENT_VERSION)?;
        write_card32(&mut writer, buffer_length)?;
        writer.write_all(self.buffer.as_bytes())?;

        Ok(())
    }

    #[must_use]
    pub fn kind(&self) -> CommentKind {
        self.kind
    }

    #[must_use]
    pub fn buffer(&self) -> &str {
        self.buffer.as_ref()
    }
}
