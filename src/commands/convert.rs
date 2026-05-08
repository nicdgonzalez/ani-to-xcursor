use std::fs::{self, File};
use std::io;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use ani::{Ani, Image};
use anyhow::Context as _;
use colored::Colorize as _;
use image::RgbaImage;
use image::imageops::FilterType;

use crate::commands::prelude::*;
use crate::config::Size;
use crate::package::Package;

#[derive(Debug, Default, clap::Args)]
pub struct Convert {
    pub input: PathBuf,

    #[arg(long, value_delimiter = ',', default_value = "32,48,64,96")]
    pub sizes: Vec<Size>,
}

struct Extracted<'a> {
    image: &'a Image,
    path: PathBuf,
}

struct XcursorEntry {
    size: u32,
    x: u16,
    y: u16,
    path: PathBuf,
    duration: u64,
}

impl Run for Convert {
    fn run(self, ctx: &mut Context) -> anyhow::Result<()> {
        let path = ctx.package.as_path().join(&self.input);
        let xcursor =
            convert_cursor(&path, &ctx.package, &self.sizes).context("failed to create Xcursor")?;

        writeln!(
            io::stderr(),
            "{}: {:#}",
            "Created Xcursor".bold().green(),
            xcursor.display()
        )
        .ok();

        Ok(())
    }
}

/// Convert from ANI to Xcursor.
pub(crate) fn convert_cursor(
    input: &Path,
    package: &Package,
    sizes: &[Size],
) -> anyhow::Result<PathBuf> {
    let ani = Ani::open(input).context("failed to decode ANI file")?;

    // To keep everything organized, we'll reuse the original input file name (minus the extension)
    // for our generated files and directories.
    let file_stem = input
        .file_stem()
        .expect("expected file name to exist")
        .to_str()
        .context("expected file name to be valid unicode")?;

    // Extract animation frames
    let build = package.build();
    debug_assert!(build.as_path().try_exists().is_ok_and(|exists| exists));

    let mut output_dir = build.frames();
    debug_assert!(output_dir.try_exists().is_ok_and(|exists| exists));

    output_dir.push(file_stem);
    fs::create_dir_all(&output_dir).context("failed to create frame output directory")?;

    debug_assert!(output_dir.try_exists().is_ok_and(|exists| exists));

    let mut entries = get_entries(&ani, &output_dir, sizes)?;
    entries.sort_by_key(|entry| entry.size);

    // Generate the xcursorgen configuration file
    let config = output_dir.join(format!("{file_stem}.cursor"));
    create_xcursorgen_configuration(&config, &entries)
        .context("failed to create xcursorgen configuration file")?;

    // Generate Xcursor
    let xcursor = output_dir.join(file_stem);
    call_xcursorgen(&output_dir, &config, &xcursor).context("failed to create xcursor")?;

    Ok(xcursor)
}

fn get_entries(ani: &Ani, output_dir: &Path, sizes: &[Size]) -> anyhow::Result<Vec<XcursorEntry>> {
    let extracted = extract_frames(ani.frames(), output_dir).collect::<Vec<_>>();

    let rates = ani.rates_or_default();
    let sequence = ani.sequence_or_default();

    let mut entries = Vec::new();

    for (index, frame) in &extracted {
        process_frame(
            frame,
            *index,
            output_dir,
            &rates,
            &mut entries,
            sizes.to_owned(),
        )?;
    }

    let frames = extracted
        .into_iter()
        .map(|(_, frame)| {
            frame
                .into_iter()
                .map(|(_, extracted)| extracted)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    entries.extend(get_xcursorgen_entries(&sequence, &rates, &frames)?);

    Ok(entries)
}

fn process_frame(
    frame: &[(u32, Extracted<'_>)],
    index: usize,
    output_dir: &Path,
    rates: &[u32],
    entries: &mut Vec<XcursorEntry>,
    mut target_sizes: Vec<Size>,
) -> anyhow::Result<()> {
    let mut largest = None::<(Size, &Extracted)>;

    for (size, variant) in frame {
        let Ok(size) = Size::new(*size) else {
            tracing::warn!("skipping non-standard cursor size {size}");
            continue;
        };

        // Save frames to the file system for xcursorgen to reference.
        let writer = File::create(&variant.path)?;
        tracing::info!("created file: {:#}", variant.path.display());
        variant.image.write_png(writer)?;

        target_sizes.retain(|&target| target != size);

        if largest.is_none() || largest.is_some_and(|(largest_size, _)| size > largest_size) {
            largest = Some((size, variant));
        }
    }

    // loop through remaining targets, using largest image to upscale or downscale.
    if let Some((original_size, original_extracted)) = largest {
        for target in target_sizes {
            let original_size = u16::from(original_size.0);
            let target_size = u16::from(target.0);

            let (original_x, original_y) =
                original_extracted.image.cursor_hotspot().unwrap_or((0, 0));

            let scaled_x = original_x / original_size * target_size;
            let scaled_y = original_y / original_size * target_size;

            let file_name = format!("{index:02}-{target_size}.png");
            let path = output_dir.join(&file_name);

            let original_image = RgbaImage::from_raw(
                original_extracted.image.width(),
                original_extracted.image.height(),
                original_extracted.image.rgba_data().to_vec(),
            )
            .context("failed to load source image")?;

            let scaled_image = image::imageops::resize(
                &original_image,
                u32::from(target_size),
                u32::from(target_size),
                FilterType::Lanczos3,
            );

            scaled_image.save(&path).context("failed to resize image")?;

            let rate = rates
                .get(index)
                .with_context(|| format!("rate not found at index {index}"))?;
            let duration = u64::from(*rate) * 1000 / 60;

            entries.push(XcursorEntry {
                size: u32::from(target_size),
                x: scaled_x,
                y: scaled_y,
                path,
                duration,
            });
        }
    }

    Ok(())
}

fn extract_frames<'a>(
    frames: &'a [Vec<Image>],
    output_dir: &Path,
) -> impl Iterator<Item = (usize, Vec<(u32, Extracted<'a>)>)> {
    frames.iter().enumerate().map(move |(i, frame)| {
        let images = frame
            .iter()
            .map(move |image| {
                let width = image.width();
                let file_name = format!("{i:02}-{width}.png");
                let path = output_dir.join(&file_name);
                (width, Extracted { image, path })
            })
            .collect::<Vec<_>>();

        (i, images)
    })
}

fn create_xcursorgen_configuration(output: &Path, entries: &[XcursorEntry]) -> anyhow::Result<()> {
    let contents = entries
        .iter()
        .map(|entry| {
            format!(
                "{size} {x} {y} {file_name} {duration}",
                size = entry.size,
                x = entry.x,
                y = entry.y,
                file_name = entry
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .expect("code-generated file name is not valid unicode"),
                duration = entry.duration
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(output, contents).context("failed to write xcursorgen configuration file")?;

    Ok(())
}

fn get_xcursorgen_entries(
    sequence: &[u32],
    rates: &[u32],
    frames: &[Vec<Extracted<'_>>],
) -> anyhow::Result<Vec<XcursorEntry>> {
    let mut entries = Vec::new();

    for &i in sequence {
        let index = usize::try_from(i).expect("u32 overflowed usize");
        let frame = frames
            .get(index)
            .with_context(|| format!("frame not found at index {index}"))?;

        for variant in frame {
            // TODO: There is no guarantee that all frames contain the same number of variants,
            // which should result in an error since it affects the animation in an unexpected way.
            //
            // ...I think it's safe to remove this todo now... but not sure.
            let size = variant.image.width();
            let (x, y) = variant.image.cursor_hotspot().unwrap_or((0, 0));
            let path = variant.path.clone();

            let rate = *rates
                .get(index)
                .with_context(|| format!("rate not found at index {index}"))?;
            let duration_ms = u64::from(rate) * 1000 / 60;

            entries.push(XcursorEntry {
                size,
                x,
                y,
                path,
                duration: duration_ms,
            });
        }
    }

    Ok(entries)
}

fn call_xcursorgen(current_dir: &Path, config_path: &Path, target: &Path) -> anyhow::Result<()> {
    let status = Command::new("xcursorgen")
        .args([config_path, target])
        .current_dir(current_dir)
        .status()
        .context("failed to execute xcursorgen")?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow::anyhow!("process failed with exit code: {code}")),
        None => Err(anyhow::anyhow!("process terminated due to signal")),
    }
}
