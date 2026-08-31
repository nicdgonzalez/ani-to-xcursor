use ico::IconImage;

use crate::error::ParseError;
use crate::{Ani, Header, Metadata};

#[derive(Default)]
pub struct AniBuilder {
    metadata: Option<Metadata>,
    header: Option<Header>,
    rates: Option<Vec<u32>>,
    sequence: Option<Vec<u32>>,
    frames: Option<Vec<Vec<IconImage>>>,
}

impl AniBuilder {
    pub(crate) fn build(self) -> Result<Ani, ParseError> {
        let header = self
            .header
            .ok_or(ParseError::MissingRequiredChunk { identifier: "anih" })?;
        let frames = self
            .frames
            .ok_or(ParseError::MissingRequiredChunk { identifier: "fram" })?;

        Ok(Ani {
            metadata: self.metadata,
            header,
            rates: self.rates,
            sequence: self.sequence,
            frames,
        })
    }

    pub fn with_metadata(self, value: Metadata) -> Self {
        Self {
            metadata: Some(value),
            ..self
        }
    }

    pub fn with_header(self, value: Header) -> Self {
        Self {
            header: Some(value),
            ..self
        }
    }

    pub fn with_rates(self, value: Vec<u32>) -> Self {
        Self {
            rates: Some(value),
            ..self
        }
    }

    pub fn with_sequence(self, value: Vec<u32>) -> Self {
        Self {
            sequence: Some(value),
            ..self
        }
    }

    pub fn with_frames(self, value: Vec<Vec<IconImage>>) -> Self {
        Self {
            frames: Some(value),
            ..self
        }
    }
}
