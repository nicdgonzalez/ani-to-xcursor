use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};

use ani::{Ani, Image as Frame};
use anyhow::Context as _;
use colored::Colorize as _;
use image::imageops::FilterType;
use image::{RgbaImage, imageops};
use tracing::{debug, debug_span, warn};
use xcur::{Image as XcursorImage, Xcursor};

use crate::commands::prelude::*;
use crate::config::Size;

#[derive(Debug, Default, clap::Args)]
pub struct Convert {
    pub input: PathBuf,
    pub output: Option<PathBuf>,

    #[arg(long, value_delimiter = ',', default_value = "32,48,64,96")]
    pub sizes: Vec<Size>,
}

impl Run for Convert {
    fn run(self, ctx: &mut Context) -> anyhow::Result<()> {
        let input = ctx.package.as_path().join(&self.input);
        let file_stem = input
            .file_stem()
            .expect("expected file name to exist")
            .to_str()
            .context("expected file name to be valid unicode")?;
        let output = ctx
            .package
            .as_path()
            .join(self.output.unwrap_or_else(|| PathBuf::from(file_stem)));

        convert_cursor(&input, &self.sizes, &output).context("failed to create Xcursor")?;

        writeln!(
            io::stderr(),
            "{}: {:#}",
            "Created Xcursor".bold().green(),
            output.display()
        )
        .ok();

        Ok(())
    }
}

struct Image {
    rgba: RgbaImage,
    size: Size,
    hotspot_x: u16,
    hotspot_y: u16,
}

/// Convert from ANI to Xcursor.
pub(crate) fn convert_cursor(input: &Path, sizes: &[Size], output: &Path) -> anyhow::Result<()> {
    let ani = Ani::open(input).context("failed to decode ANI file")?;
    let rates = ani.rates_or_default();
    let mut images = ani
        .frames()
        .iter()
        .enumerate()
        .map(|(frame_idx, frame)| {
            let span = debug_span!("frame", frame_idx);
            let sizes = sizes.iter().copied().collect::<BTreeSet<_>>();
            let delay = rates
                .get(frame_idx)
                .map(|&rate| Duration::from_millis(u64::from(rate) * 1000 / 60))
                .inspect(|delay| debug!("Delay: {delay:?}"))
                .with_context(|| format!("rate not found at index {frame_idx}"))?;

            span.in_scope(move || extract_images(frame, sizes, &delay))
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    images.sort_by_key(|image| image.width().max(image.height()));

    let xcursor = Xcursor::new(images, vec![]);
    save_xcursor(output, &xcursor)?;

    Ok(())
}

fn extract_images(
    frame: &[Frame],
    mut targets: BTreeSet<Size>,
    delay: &Duration,
) -> anyhow::Result<Vec<XcursorImage>> {
    let mut images = Vec::<XcursorImage>::new();
    let mut largest = None::<Image>;

    for image in frame {
        let width = image.width();
        let height = image.height();
        let nominal = width.max(height);

        let Ok(size) = Size::new(nominal) else {
            warn!("skipping image with non-standard size: {nominal}");
            continue;
        };

        let (hotspot_x, hotspot_y) = image.cursor_hotspot().unwrap_or((0, 0));
        debug!("Hotspot: ({hotspot_x}, {hotspot_y})");

        let rgba = RgbaImage::from_raw(width, height, image.rgba_data().to_vec())
            .context("failed to load image from RGBA data")?;

        images.push(XcursorImage::new(
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
) -> impl Iterator<Item = anyhow::Result<XcursorImage>> {
    targets
        .iter()
        .map(|target| -> anyhow::Result<XcursorImage> {
            let source_size = u16::from(source.size.0);
            let target_size = u16::from(target.0);

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

            Ok(XcursorImage::new(
                u16::try_from(target_image.width()).unwrap(),
                u16::try_from(target_image.height()).unwrap(),
                target_x,
                target_y,
                *delay,
                argb,
            )?)
        })
}

fn save_xcursor(path: &Path, xcursor: &Xcursor) -> anyhow::Result<()> {
    let mut buffer = Vec::new();
    xcursor
        .write(&mut buffer)
        .context("failed to create xcursor")?;

    fs::write(path, &buffer).context("failed to save Xcursor")?;
    Ok(())
}
