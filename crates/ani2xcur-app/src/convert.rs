use std::num::TryFromIntError;
use std::time::Duration;

use ani::Ani;
use ani2xcur_core::Size;
use image::RgbaImage;
use image::imageops::{self, FilterType};
use tracing::warn;
use xcur::Xcursor;

/// Request to convert a cursor from ANI to Xcursor.
pub struct ConvertCursorRequest {
    pub ani: Ani,
    pub sizes: Vec<Size>,
}

/// Errors that can occur while converting a cursor from ANI to Xcursor.
#[derive(Debug, thiserror::Error)]
pub enum ConvertCursorError {
    #[error("cursor width/height too large")]
    SizeTooLarge(#[source] TryFromIntError),

    #[error("invalid Xcursor image")]
    InvalidXcursorImage(#[source] xcur::ParseError),

    // TODO: This is an implementation detail; error needs to be caught earlier.
    #[error("frame not found at index {index}")]
    InvalidFrameIndex { index: usize },
}

struct ImageMetadata {
    rgba: RgbaImage,
    size: Size,
    hotspot_x: u16,
    hotspot_y: u16,
}

/// Converts a cursor from ANI to Xcursor format.
pub fn xcursor_from_ani(request: ConvertCursorRequest) -> Result<Xcursor, ConvertCursorError> {
    let ani = request.ani;

    let rates = ani.rates_or_default();
    let sequence = ani.sequence_or_default();

    let delays = rates
        .into_iter()
        .map(|r| Duration::from_millis(u64::from(r) * 1000 / 60));

    let mut frames = Vec::<xcur::Image>::new();

    for (frame, delay) in ani.frames().iter().zip(delays) {
        let mut sizes = request.sizes.clone();

        // Track the largest source image for use when rescaling the remaining target `sizes`.
        let mut largest = None::<ImageMetadata>;

        for image in frame {
            let width = image
                .width()
                .try_into()
                .map_err(ConvertCursorError::SizeTooLarge)?;
            let height = image
                .height()
                .try_into()
                .map_err(ConvertCursorError::SizeTooLarge)?;
            let nominal = u8::max(width, height);

            let Some(size) = Size::checked_new(nominal) else {
                warn!("skipping image with non-standard size: {nominal}");
                continue;
            };

            let (hotspot_x, hotspot_y) = image.cursor_hotspot().unwrap_or((0, 0));
            let rgba = RgbaImage::from_raw(
                u32::from(width),
                u32::from(height),
                image.rgba_data().to_vec(),
            )
            // This would only panic if `ico::IconDirEntry::decode` (from our `ani` dependency)
            // gives us the wrong height/width for our image data, causing our width/height
            // to claim more data than it can actually holds.
            .expect("width/height derived from image buffer so container should always fit");

            let xcursor_image = xcur::Image::new(
                u16::from(width),
                u16::from(height),
                hotspot_x,
                hotspot_y,
                delay,
                rgba_to_argb(rgba.as_raw()).collect(),
            )
            .map_err(ConvertCursorError::InvalidXcursorImage)?;

            if let Some(index) = sizes.iter().position(|s| size == *s) {
                // Only push to images if the user wants this cursor size.
                frames.push(xcursor_image);
                _ = sizes.remove(index);
            }

            if largest.as_ref().is_some_and(|largest| size > largest.size) {
                // Track the largest source image for resizing to requested sizes.
                largest = Some(ImageMetadata {
                    rgba,
                    size,
                    hotspot_x,
                    hotspot_y,
                });
            }
        }

        // Generate the remaining requested sizes using the largest source image to scale up/down.
        if let Some(source) = largest {
            for target in sizes {
                let source_size = u16::from(source.size);
                let target_size = u16::from(target);

                let target_x = source.hotspot_x * target_size / source_size;
                let target_y = source.hotspot_y * target_size / source_size;

                let target_image = imageops::resize(
                    &source.rgba,
                    u32::from(target_size),
                    u32::from(target_size),
                    FilterType::Lanczos3,
                );

                let argb = rgba_to_argb(target_image.as_raw()).collect();
                let xcursor_image =
                    xcur::Image::new(target_size, target_size, target_x, target_y, delay, argb)
                        .map_err(ConvertCursorError::InvalidXcursorImage)?;

                frames.push(xcursor_image);
            }
        }
    }

    frames.sort_by_key(|image| u16::max(image.width(), image.height()));
    assert!(frames.len().is_multiple_of(request.sizes.len()));

    let frames_per_size = frames.len() / request.sizes.len();
    // `frames` contains one of each of the available animation frames.
    // `images` contains the actual frames needed for the animation (e.g., if the animation asks
    // for (6) Frame 1's, `images` should contain (6) copies of Frame 1).
    let mut images = Vec::<xcur::Image>::with_capacity(sequence.len() * request.sizes.len());

    for size_idx in 0..request.sizes.len() {
        let offset = size_idx * frames_per_size;

        for sequence_idx in &sequence {
            let sequence_idx = usize::try_from(*sequence_idx).expect("u32 overflowed usize");
            let frame_idx = sequence_idx + offset;

            let frame = frames
                .get(frame_idx)
                .cloned()
                .ok_or(ConvertCursorError::InvalidFrameIndex { index: frame_idx })?;

            images.push(frame);
        }
    }

    let comments = vec![]; // Comments are ignored.

    Ok(Xcursor::new(images, comments))
}

/// Shifts the bytes from RGBA to ARGB format.
fn rgba_to_argb(bytes: &[u8]) -> impl Iterator<Item = u32> {
    let (chunks, remainder) = bytes.as_chunks::<4>();
    debug_assert!(remainder.is_empty());

    chunks.iter().map(|&[r, g, b, a]| {
        (u32::from(a) << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
}
