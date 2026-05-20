use std::collections::BTreeSet;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};

use ani::Ani;
use anyhow::Context as _;
use colored::Colorize as _;
use image::imageops::FilterType;
use image::{RgbaImage, imageops};
use tracing::{debug, warn};
use xcur::Xcursor;

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
            debug!("Frame #{frame_idx}");

            let mut sizes: BTreeSet<_> = sizes.iter().copied().collect();
            let mut images = Vec::<xcur::Image>::new();
            let mut largest = None::<Image>;

            let delay = rates
                .get(frame_idx)
                .map(|&rate| Duration::from_millis(u64::from(rate) * 1000 / 60))
                .inspect(|delay| debug!("Delay: {delay:?}"))
                .with_context(|| format!("rate not found at index {frame_idx}"))?;

            for (image_idx, image) in frame.iter().enumerate() {
                let width = image.width();
                let height = image.height();

                let Ok(size) = Size::new(width.max(height)) else {
                    warn!("skipping non-standard cursor size: {width}");
                    continue;
                };

                let (hotspot_x, hotspot_y) = image.cursor_hotspot().unwrap_or((0, 0));
                debug!("Hotspot: ({hotspot_x}, {hotspot_y})");

                let rgba = RgbaImage::from_raw(width, height, image.rgba_data().to_vec())
                    .with_context(|| {
                        format!("failed to load image from RGBA data ({frame_idx}, {image_idx})")
                    })?;

                let argb = rgba_to_argb(rgba.as_raw()).collect::<Vec<_>>();

                images.push(xcur::Image::new(
                    u16::try_from(width).unwrap(),
                    u16::try_from(height).unwrap(),
                    hotspot_x,
                    hotspot_y,
                    delay,
                    argb,
                )?);

                sizes.remove(&size);

                if largest.as_ref().is_none_or(|largest| size > largest.size) {
                    largest = Some(Image {
                        rgba,
                        size,
                        hotspot_x,
                        hotspot_y,
                    });
                }
            }

            // Iterate over the remaining targets using the largest image to upscale/downscale.
            if let Some(source) = &largest {
                for target in sizes {
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

                    images.push(xcur::Image::new(
                        u16::try_from(target_image.width()).unwrap(),
                        u16::try_from(target_image.height()).unwrap(),
                        target_x,
                        target_y,
                        delay,
                        argb,
                    )?);
                }
            }

            Ok(images)
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    images.sort_by_key(|image| image.width().max(image.height()));

    let xcursor = Xcursor::new(images, vec![]);

    let mut buffer = Vec::new();
    xcursor
        .write(&mut buffer)
        .context("failed to create xcursor")?;

    fs::write(output, &buffer).context("failed to save Xcursor")?;

    Ok(())
}

// fn convert_frame(
//     frame: &ani::Image,
//     mut target_sizes: Vec<Size>,
//     delay: Duration,
// ) -> anyhow::Result<(Size, xcur::Image)> {
//     todo!()
// }

fn rgba_to_argb(bytes: &[u8]) -> impl Iterator<Item = u32> {
    let (chunks, remainder) = bytes.as_chunks::<4>();
    debug_assert!(remainder.is_empty());

    chunks.iter().map(|&[r, g, b, a]| {
        (u32::from(a) << 24) | (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
    })
}
