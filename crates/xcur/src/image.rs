use std::io::{self, Write};
use std::time::Duration;

use crate::error::ParseError;
use crate::{CARD32_SIZE, Entry, Type, write_card32};

pub const IMAGE_HEADER_SIZE: u32 = 36;
const IMAGE_VERSION: u32 = 1;

#[derive(Debug, Clone)]
pub struct Image {
    width: u16,
    height: u16,
    hotspot_x: u16,
    hotspot_y: u16,
    delay: Duration,
    argb: Vec<u32>,
}

impl Image {
    pub fn new(
        width: u16,
        height: u16,
        hotspot_x: u16,
        hotspot_y: u16,
        delay: Duration,
        argb: Vec<u32>,
    ) -> Result<Self, ParseError> {
        if width > 0x7FFF || height > 0x7FFF {
            return Err(ParseError::ImageSize);
        }

        if hotspot_x > width {
            return Err(ParseError::InvalidHotspot {
                image_size: width,
                hotspot: hotspot_x,
            });
        }

        if hotspot_y > height {
            return Err(ParseError::InvalidHotspot {
                image_size: height,
                hotspot: hotspot_y,
            });
        }

        Ok(Self {
            width,
            height,
            hotspot_x,
            hotspot_y,
            delay,
            argb,
        })
    }

    pub(crate) fn from_chunk(buffer: &[u8], entry: &Entry) -> Result<Self, ParseError> {
        let position = usize::try_from(entry.position).expect("u32 overflowed usize");
        let header_size = usize::try_from(IMAGE_HEADER_SIZE).expect("u32 overflowed usize");

        let start = position;
        let end = position + header_size;
        let raw_header = &buffer[start..end];

        let mut fields = raw_header
            .as_chunks::<CARD32_SIZE>()
            .0
            .iter()
            .copied()
            .map(u32::from_le_bytes);

        let _header_size = fields.next().unwrap();
        let _raw_type = fields.next().unwrap();
        let _raw_subtype = fields.next().unwrap();
        let _version = fields.next().unwrap();
        let width = fields.next().unwrap();
        let height = fields.next().unwrap();
        let hotspot_x = fields.next().unwrap();
        let hotspot_y = fields.next().unwrap();
        let delay = fields.next().unwrap();

        if width > 0x7FFF || height > 0x7FFF {
            return Err(ParseError::ImageSize);
        }

        if hotspot_x > width {
            return Err(ParseError::InvalidHotspot {
                image_size: u16::try_from(width).unwrap(),
                hotspot: u16::try_from(hotspot_x).expect("u32 hotspot x overflowed u16"),
            });
        }

        if hotspot_y > height {
            return Err(ParseError::InvalidHotspot {
                image_size: u16::try_from(height).unwrap(),
                hotspot: u16::try_from(hotspot_y).expect("u32 hotspot y overflowed u16"),
            });
        }

        let image_size = {
            let value = width * height * 4; // 4 = 1 byte per value in ARGB
            usize::try_from(value).expect("u32 overflowed usize")
        };
        let start = end;
        let end = start + image_size;
        let argb = buffer[start..end]
            .as_chunks::<4>()
            .0
            .iter()
            .copied()
            .map(u32::from_le_bytes)
            .collect::<Vec<u32>>();

        Ok(Self {
            width: u16::try_from(width).unwrap(),
            height: u16::try_from(height).unwrap(),
            hotspot_x: u16::try_from(hotspot_x).unwrap(),
            hotspot_y: u16::try_from(hotspot_y).unwrap(),
            delay: Duration::from_millis(u64::from(delay)),
            argb,
        })
    }

    pub fn write<W>(&self, mut writer: W) -> io::Result<()>
    where
        W: Write,
    {
        assert_eq!(
            self.argb.len(),
            usize::from(self.width) * usize::from(self.height)
        );

        let width = u32::from(self.width);
        let height = u32::from(self.height);
        let hotspot_x = u32::from(self.hotspot_x);
        let hotspot_y = u32::from(self.hotspot_y);

        // Nominal size subtype -- usually width/height for square cursors.
        let subtype = width.max(height);

        let delay = self.delay.as_millis().try_into().unwrap_or(u32::MAX);

        write_card32(&mut writer, IMAGE_HEADER_SIZE)?;
        write_card32(&mut writer, Type::Image as u32)?;
        write_card32(&mut writer, subtype)?;
        write_card32(&mut writer, IMAGE_VERSION)?;
        write_card32(&mut writer, width)?;
        write_card32(&mut writer, height)?;
        write_card32(&mut writer, hotspot_x)?;
        write_card32(&mut writer, hotspot_y)?;
        write_card32(&mut writer, delay)?;

        for &pixel in &self.argb {
            write_card32(&mut writer, pixel)?;
        }

        Ok(())
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.height
    }

    #[must_use]
    pub fn hotspot_x(&self) -> u16 {
        self.hotspot_x
    }

    #[must_use]
    pub fn hotspot_y(&self) -> u16 {
        self.hotspot_y
    }

    #[must_use]
    pub fn delay(&self) -> &Duration {
        &self.delay
    }

    #[must_use]
    pub fn argb(&self) -> &[u32] {
        &self.argb
    }
}
