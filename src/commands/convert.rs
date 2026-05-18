use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, io};

use ani::{Ani, Image};
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

/// Convert from ANI to Xcursor.
pub(crate) fn convert_cursor(input: &Path, sizes: &[Size], output: &Path) -> anyhow::Result<()> {
    let ani = Ani::open(input).context("failed to decode ANI file")?;
    let rates = ani.rates_or_default();
    let mut images = ani
        .frames()
        .iter()
        .enumerate()
        .map(|(i, frame)| {
            debug!("Frame #{i}");
            let mut sizes = sizes.to_owned();
            let mut images = Vec::<(Size, xcur::Image)>::new();
            let mut largest = None::<(&Image, RgbaImage)>;
            let rate = rates
                .get(i)
                .with_context(|| format!("rate not found at index {i}"))?;
            let delay = Duration::from_millis(u64::from(*rate) * 1000 / 60);
            debug!("Delay: {delay:?}");

            for (j, image) in frame.iter().enumerate() {
                let width = image.width();
                let height = image.height();

                let Ok(size) = Size::new(width.max(height)) else {
                    warn!("skipping non-standard cursor size: {width}");
                    continue;
                };
                debug!("Target size: {}", size.0);

                let (hotspot_x, hotspot_y) = image.cursor_hotspot().unwrap_or((0, 0));
                debug!("Hotspot: ({hotspot_x}, {hotspot_y})");
                let rgba = image.rgba_data();

                let rgba = RgbaImage::from_raw(width, height, rgba.to_vec())
                    .with_context(|| format!("failed to load image ({i}, {j})"))?;

                let (chunks, remainder) = image.rgba_data().as_chunks::<4>();
                debug_assert!(remainder.is_empty());
                let argb = chunks
                    .iter()
                    .map(|&[r, g, b, a]| {
                        (u32::from(a) << 24)
                            | (u32::from(r) << 16)
                            | (u32::from(g) << 8)
                            | u32::from(b)
                    })
                    .collect::<Vec<u32>>();

                let xcur_image = xcur::Image::new(
                    u16::try_from(width).unwrap(),
                    u16::try_from(height).unwrap(),
                    hotspot_x,
                    hotspot_y,
                    delay,
                    argb,
                )?;

                images.push((size, xcur_image));
                sizes.retain(|&target| target != size);

                if largest
                    .as_ref()
                    .is_none_or(|&(largest, _)| width > largest.width())
                {
                    largest = Some((image, rgba));
                }
            }

            // Iterate over the remaining targets using the largest image to upscale/downscale.
            if let Some((original, rgba)) = &largest {
                for target in sizes {
                    let source_width = original.width();
                    let source_height = original.height();
                    let source_size = source_width.max(source_height);
                    let source_size = u16::try_from(source_size).unwrap();

                    let target_size = u16::from(target.0);
                    debug!("Target size: {target_size}");

                    let (source_x, source_y) = original.cursor_hotspot().unwrap_or((0, 0));
                    let target_x = source_x * target_size / source_size;
                    let target_y = source_y * target_size / source_size;
                    debug!("Hotspot: ({target_x}, {target_y})");

                    let target_image = imageops::resize(
                        rgba,
                        u32::from(target_size),
                        u32::from(target_size),
                        FilterType::Lanczos3,
                    );

                    let argb = target_image
                        .as_raw()
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|&[r, g, b, a]| {
                            (u32::from(a) << 24)
                                | (u32::from(r) << 16)
                                | (u32::from(g) << 8)
                                | u32::from(b)
                        })
                        .collect::<Vec<u32>>();

                    let xcur_image = xcur::Image::new(
                        u16::try_from(target_image.width()).unwrap(),
                        u16::try_from(target_image.height()).unwrap(),
                        target_x,
                        target_y,
                        delay,
                        argb,
                    )?;

                    images.push((target, xcur_image));
                }
            }

            Ok(images)
        })
        .collect::<anyhow::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    images.sort_by_key(|&(size, _)| size);
    let images = images
        .into_iter()
        .map(|(_, image)| image)
        .collect::<Vec<_>>();

    let xcursor = Xcursor::new(images, vec![]);

    let mut buffer = Vec::new();
    xcursor
        .write(&mut buffer)
        .context("failed to create xcursor")?;

    fs::write(output, &buffer).context("failed to save Xcursor")?;

    Ok(())
}
