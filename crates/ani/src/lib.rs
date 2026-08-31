//! # ANI
//!
//! Decoder for the ANI file format.
//!
//! <https://en.wikipedia.org/wiki/ANI_(file_format)>

#![warn(
    missing_docs,
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

mod builder;
mod error;
mod parser;

use std::fs::File;
use std::io::{self, Read};
use std::mem;
use std::path::Path;

use bitflags::bitflags;
pub use ico::IconImage as Image;
use ico::{IconDir, IconDirEntry};

use crate::builder::AniBuilder;
use crate::error::ParseError;
pub use crate::parser::Parser;

bitflags! {
    /// Represents a bit flag used in the ANI header.
    #[derive(Debug, Clone, Copy)]
    pub struct Flag: u32 {
        /// Indicates the frames are in Windows ICO format.
        const ICON = 0x01;
        /// Indicates the animation has a custom sequence.
        ///
        /// Custom sequences are commonly used to save space and avoid repeating frames.
        const SEQUENCE = 0x02;
    }
}

/// Represents a decoded ANI file.
#[derive(Clone)]
pub struct Ani {
    metadata: Option<Metadata>,
    header: Header,
    rates: Option<Vec<u32>>,
    sequence: Option<Vec<u32>>,
    frames: Vec<Vec<Image>>,
}

impl Ani {
    /// Open the ANI file at path and decode it.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - File cannot be read
    /// - File is not a valid ANI file
    pub fn open<P>(path: P) -> Result<Self, ParseError>
    where
        P: AsRef<Path>,
    {
        File::open(path)
            .map_err(|err| ParseError::ReadFailed { source: err })
            .and_then(Self::from_reader)
    }

    /// Read and decode an ANI file.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - Cannot read from `reader`
    /// - File is not a valid ANI file
    pub fn from_reader<R>(mut reader: R) -> Result<Self, ParseError>
    where
        R: Read,
    {
        let mut buffer = Vec::new();
        reader
            .read_to_end(&mut buffer)
            .map_err(|err| ParseError::ReadFailed { source: err })?;
        Self::try_from(buffer.as_slice())
    }

    /// Decode an ANI-formatted buffer.
    ///
    /// # Errors
    ///
    /// This function returns an error if:
    ///
    /// - File is not a valid ANI file
    pub fn from_bytes(buffer: &[u8]) -> Result<Self, ParseError> {
        Self::try_from(buffer)
    }

    /// Additional information about the cursor (title, author).
    #[must_use]
    pub fn metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    /// Additional context for building the animated cursor.
    ///
    /// <https://en.wikipedia.org/wiki/ANI_(file_format)>
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Display rate for each of the frames, if available.
    #[must_use]
    pub fn rates(&self) -> Option<&[u32]> {
        self.rates.as_deref()
    }

    /// Display rate for each of the frames, if available. Otherwise, it is created using
    /// the information provided in the header.
    ///
    /// [Explanation for how default is constructed]
    ///
    /// # Panics
    ///
    /// This function panics on architectures where `usize` is smaller than `u32`.
    #[must_use]
    pub fn rates_or_default(&self) -> Vec<u32> {
        self.rates.as_ref().map_or_else(
            || {
                let count = usize::try_from(self.header.frames).expect("u32 overflowed usize");
                vec![self.header.jif_rate(); count]
            },
            ToOwned::to_owned,
        )
    }

    /// Ordering of the frames, if available.
    #[must_use]
    pub fn sequence(&self) -> Option<&[u32]> {
        self.sequence.as_deref()
    }

    /// Ordering of the frames, if available. Otherwise, it is created using the information
    /// provided in the header.
    ///
    /// [Explanation for how default is constructed]
    #[must_use]
    pub fn sequence_or_default(&self) -> Vec<u32> {
        self.sequence.as_ref().map_or_else(
            || {
                (0..self.header.steps)
                    .map(|i| i % self.header.frames)
                    .collect()
            },
            ToOwned::to_owned,
        )
    }

    /// Collection of images stored within the ANI file.
    #[must_use]
    pub fn frames(&self) -> &[Vec<Image>] {
        &self.frames
    }
}

impl TryFrom<&[u8]> for Ani {
    type Error = ParseError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let mut parser = Parser::new(value);
        let riff = parser.read_chunk()?;
        let acon = riff.data.as_subchunk()?;

        if riff.identifier.as_bytes() != b"RIFF" || acon.identifier.as_bytes() != b"ACON" {
            return Err(ParseError::InvalidSignature);
        }

        Parser::new(acon.data.as_bytes())
            .into_iter()
            .try_fold(AniBuilder::default(), |builder, chunk| {
                tracing::debug!("reading next chunk");
                let chunk = chunk?;
                tracing::debug!("chunk identifier: {:?}", chunk.identifier.as_str());

                match chunk.identifier.as_bytes() {
                    b"LIST" => {
                        let subchunk = chunk.data.as_subchunk()?;
                        tracing::debug!("Parsing subchunk: {:?}", subchunk.identifier.as_str());

                        match subchunk.identifier.as_bytes() {
                            b"INFO" => Metadata::from_data(subchunk.data.as_bytes())
                                .map(|v| builder.with_metadata(v)),
                            b"fram" => {
                                read_fram(subchunk.data.as_bytes()).map(|v| builder.with_frames(v))
                            }
                            bytes => Err(ParseError::InvalidIdentifier {
                                identifier: bytes.to_vec(),
                            }),
                        }
                    }
                    b"anih" => {
                        Header::from_data(chunk.data.as_bytes()).map(|v| builder.with_header(v))
                    }
                    b"rate" => read_rate(chunk.data.as_bytes()).map(|v| builder.with_rates(v)),
                    b"seq " => read_seq(chunk.data.as_bytes()).map(|v| builder.with_sequence(v)),
                    bytes => Err(ParseError::InvalidIdentifier {
                        identifier: bytes.to_vec(),
                    }),
                }
            })?
            .build()
    }
}

/// Additional information provided by the cursor author.
#[derive(Debug, Clone)]
pub struct Metadata {
    title: Option<String>,
    author: Option<String>,
}

impl Metadata {
    fn from_data(data: &[u8]) -> Result<Self, ParseError> {
        let parser = Parser::new(data);
        let mut title = None::<String>;
        let mut author = None::<String>;

        for chunk in parser {
            let chunk = chunk?;

            match chunk.identifier.as_bytes() {
                b"INAM" => {
                    title = Some(String::from_utf8_lossy(chunk.data.as_bytes()).to_string());
                }
                b"IART" => {
                    author = Some(String::from_utf8_lossy(chunk.data.as_bytes()).to_string());
                }
                _ => {} // Ignore additional unregistered identifiers
            }
        }

        Ok(Self { title, author })
    }

    /// Name of the cursor, if available.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Author of the cursor, if available.
    #[must_use]
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }
}

/// Represents the `anih` chunk of an ANI file.
#[derive(Debug, Clone, Copy)]
pub struct Header {
    size: u32,
    frames: u32,
    steps: u32,

    #[allow(dead_code)]
    x: u32,
    #[allow(dead_code)]
    y: u32,
    #[allow(dead_code)]
    bit_count: u32,
    #[allow(dead_code)]
    planes: u32,

    jif_rate: u32,
    flags: Flag,
}

impl Header {
    fn from_data(data: &[u8]) -> Result<Self, ParseError> {
        let [
            size,
            frames,
            steps,
            x,
            y,
            bit_count,
            planes,
            jif_rate,
            flags,
        ] = data
            .chunks_exact(mem::size_of::<u32>())
            .map(|chunk| chunk.try_into().map(u32::from_le_bytes).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| ParseError::InvalidHeader)?;

        Ok(Self {
            size,
            frames,
            steps,
            x,
            y,
            bit_count,
            planes,
            jif_rate,
            flags: Flag::from_bits_retain(flags),
        })
    }

    /// The length of the ANI header (should always be 36).
    #[must_use]
    pub const fn size(&self) -> u32 {
        self.size
    }

    /// The number of frames we can expect to find in the `fram` chunk.
    #[must_use]
    pub const fn frames(&self) -> u32 {
        self.frames
    }

    /// The number of steps in the animation loop.
    #[must_use]
    pub const fn steps(&self) -> u32 {
        self.steps
    }

    /// The default display rate in, jiffies (1/60 seconds).
    #[must_use]
    pub const fn jif_rate(&self) -> u32 {
        self.jif_rate
    }

    /// Bit flags.
    #[must_use]
    pub const fn flags(&self) -> &Flag {
        &self.flags
    }
}

/// Number of bytes a rate entry occupies.
const RATE_SIZE: usize = mem::size_of::<u32>();

/// Decode the `rate` chunk.
fn read_rate(data: &[u8]) -> Result<Vec<u32>, ParseError> {
    let mut chunks = data.chunks_exact(RATE_SIZE);

    let rates = chunks
        .by_ref()
        .map(|chunk| chunk.try_into().map(u32::from_le_bytes).unwrap())
        .collect::<Vec<_>>();

    if !chunks.remainder().is_empty() {
        return Err(ParseError::NotEnoughBytes {
            needed: RATE_SIZE - data.len().rem_euclid(RATE_SIZE),
        });
    }

    Ok(rates)
}

/// Number of bytes a sequence entry occupies.
const SEQUENCE_SIZE: usize = mem::size_of::<u32>();

/// Decode the `seq ` chunk.
fn read_seq(data: &[u8]) -> Result<Vec<u32>, ParseError> {
    tracing::debug!("Sequence data length: {}", data.len());
    let mut chunks = data.chunks_exact(SEQUENCE_SIZE);

    let sequence = chunks
        .by_ref()
        .map(|chunk| chunk.try_into().map(u32::from_le_bytes).unwrap())
        .collect::<Vec<_>>();

    if !chunks.remainder().is_empty() {
        tracing::debug!(
            "Remainder: {:?}",
            chunks.remainder().iter().collect::<Vec<_>>()
        );
        return Err(ParseError::NotEnoughBytes {
            needed: SEQUENCE_SIZE - data.len().rem_euclid(SEQUENCE_SIZE),
        });
    }

    Ok(sequence)
}

fn read_fram(data: &[u8]) -> Result<Vec<Vec<Image>>, ParseError> {
    Parser::new(data)
        .into_iter()
        .map(|chunk| {
            let chunk = chunk?;

            if chunk.identifier.as_bytes() != b"icon" {
                return Err(ParseError::InvalidIdentifier {
                    identifier: chunk.identifier.as_bytes().to_vec(),
                });
            }

            let reader = io::Cursor::new(chunk.data.as_bytes());
            let icon_dir = IconDir::read(reader).map_err(|_| ParseError::InvalidIconDir)?;

            icon_dir
                .entries()
                .iter()
                .map(IconDirEntry::decode)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| ParseError::InvalidIconDirEntry { source: err })
        })
        .collect::<Result<Vec<_>, _>>()
}
