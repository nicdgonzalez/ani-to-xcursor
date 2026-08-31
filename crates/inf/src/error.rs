use std::{error, fmt, io};

#[derive(Debug)]
pub enum ParseError {
    ReadFailure { source: io::Error },
    SectionNameEmpty,
    SectionNameTooLong,
    UnexpectedCharacter { c: char },
    UnterminatedString,
}

impl error::Error for ParseError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::ReadFailure { ref source } => Some(source),
            Self::SectionNameEmpty
            | Self::SectionNameTooLong
            | Self::UnexpectedCharacter { .. }
            | Self::UnterminatedString => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::ReadFailure { source: _ } => "failed to read data".fmt(f),
            Self::SectionNameEmpty => "section name cannot be empty".fmt(f),
            Self::SectionNameTooLong => "section name cannot exceed 255 characters".fmt(f),
            Self::UnexpectedCharacter { c } => write!(f, "unexpected character: {c:?}"),
            Self::UnterminatedString => "unterminated string".fmt(f),
        }
    }
}
