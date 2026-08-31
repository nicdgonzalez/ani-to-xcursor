use std::collections::BTreeSet;
use std::time::Duration;

use ani::Ani;
use ani2xcur_core::size::Size;
use anyhow::Context as _;
use image::RgbaImage;
use image::imageops::{self, FilterType};
use tracing::{debug, debug_span, warn};
use xcur::Xcursor;

#[derive(Clone, Copy)]
pub struct ConvertCursorRequest<'a> {
    pub ani: &'a Ani,
    pub sizes: &'a [Size],
}

struct Image {
    rgba: RgbaImage,
    size: Size,
    hotspot_x: u16,
    hotspot_y: u16,
}

/// Converts a cursor from ANI to Xcursor format.
pub fn convert_cursor(request: ConvertCursorRequest) -> anyhow::Result<Xcursor> {
    let ani = request.ani;
    let rates = ani.rates_or_default();
    let sequence = ani.sequence_or_default();
    debug!("unique frames: {}", ani.frames().len());

    let mut frames = ani
        .frames()
        .iter()
        .enumerate()
        .map(|(frame_idx, frame)| {
            let span = debug_span!("frame", frame_idx);
            let sizes = request.sizes.iter().copied().collect::<BTreeSet<_>>();
            let delay = rates
                .get(frame_idx)
                .map(|&rate| Duration::from_millis(u64::from(rate) * 1000 / 60))
                .inspect(|delay| debug!("delay: {delay:?}"))
                .with_context(|| format!("rate not found at index {frame_idx}"))?;
            span.in_scope(move || extract_images(frame, sizes, &delay))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    frames.sort_by_key(|image| image.width().max(image.height()));
    assert!(frames.len().is_multiple_of(request.sizes.len()));
    debug!("frames length: {} (frames * sizes)", frames.len());

    let frames_per_size = frames.len() / request.sizes.len();
    let mut images = Vec::<xcur::Image>::with_capacity(sequence.len() * request.sizes.len());

    for size_index in 0..request.sizes.len() {
        let offset = size_index * frames_per_size;
        let batch = sequence
            .iter()
            .map(|&i| {
                let index = usize::try_from(i).expect("u32 overflowed usize");
                let index = index + offset;

                frames
                    .get(index)
                    .cloned()
                    .with_context(|| format!("frame not found at index {index}"))
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        images.extend(batch);
    }

    Ok(Xcursor::new(images, vec![]))
}

fn extract_images(
    frame: &[ani::Image],
    mut targets: BTreeSet<Size>,
    delay: &Duration,
) -> anyhow::Result<Vec<xcur::Image>> {
    let mut images = Vec::<xcur::Image>::new();
    let mut largest = None::<Image>;

    for image in frame {
        let width = image.width();
        let height = image.height();

        let nominal = width
            .max(height)
            .try_into()
            .context("cursor width/height too large")?;

        let Some(size) = Size::checked_new(nominal) else {
            warn!("skipping image with non-standard size: {nominal}");
            continue;
        };

        let (hotspot_x, hotspot_y) = image.cursor_hotspot().unwrap_or((0, 0));
        debug!("Hotspot: ({hotspot_x}, {hotspot_y})");

        let rgba = RgbaImage::from_raw(width, height, image.rgba_data().to_vec())
            .context("failed to load image from RGBA data")?;

        images.push(xcur::Image::new(
            width.try_into().unwrap(),
            height.try_into().unwrap(),
            hotspot_x,
            hotspot_y,
            *delay,
            rgba_to_argb(rgba.as_raw()).collect(),
        )?);

        targets.remove(&size);

        if largest.as_ref().is_none_or(|largest| size > largest.size) {
            largest = Some(Image {
                rgba,
                size,
                hotspot_x,
                hotspot_y,
            });
        }
    }

    // Iterate over the remaining targets using the largest valid image to scale up/down.
    if let Some(source) = largest {
        generate_resized_images(&source, &targets, delay).try_for_each(|image| {
            let image = image?;
            images.push(image);
            Ok::<_, anyhow::Error>(())
        })?;
    }

    Ok(images)
}

fn rgba_to_argb(bytes: &[u8]) -> impl Iterator<Item = u32> {
    let (chunks, remainder) = bytes.as_chunks::<4>();
    debug_assert!(remainder.is_empty());

    chunks.iter().map(|&[r, g, b, a]| {
        (u32::from(a) << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
    })
}

fn generate_resized_images(
    source: &Image,
    targets: &BTreeSet<Size>,
    delay: &Duration,
) -> impl Iterator<Item = anyhow::Result<xcur::Image>> {
    targets.iter().map(|target| -> anyhow::Result<xcur::Image> {
        let source_size = u16::from(source.size);
        let target_size = u16::from(*target);

        let target_x = source.hotspot_x * target_size / source_size;
        let target_y = source.hotspot_y * target_size / source_size;
        debug!("Hotspot: ({target_x}, {target_y})");

        let target_image = imageops::resize(
            &source.rgba,
            u32::from(target_size),
            u32::from(target_size),
            FilterType::Lanczos3,
        );

        let argb = rgba_to_argb(target_image.as_raw()).collect::<Vec<_>>();

        Ok(xcur::Image::new(
            u16::try_from(target_image.width()).unwrap(),
            u16::try_from(target_image.height()).unwrap(),
            target_x,
            target_y,
            *delay,
            argb,
        )?)
    })
}
