use std::fmt::Write as _;
use std::io::{self, ErrorKind, Write as _};
use std::path::Path;
use std::process::Command;
use std::{env, fs, iter, path, thread};

use ani::de::{Ani, JIFFY};
use anyhow::{anyhow, Context as _};
use colored::Colorize as _;
use image::ImageFormat;
use tracing::{error, error_span, info};

use crate::commands::Run;
use crate::config::{Config, Cursor};
use crate::context::Context;
use crate::package::{Build as BuildDir, Package};
use crate::verbosity::VerbosityLevel;

#[derive(Debug, Clone, Default, clap::Args)]
pub struct Build {
    #[clap(long)]
    strict: bool,
}

impl Build {
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }
}

impl Run for Build {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        let package = if let Some(ref package) = ctx.package {
            package
        } else {
            let current_dir = env::current_dir().context("failed to get current directory")?;
            ctx.package = Some(Package::new(current_dir));
            ctx.package.as_ref().unwrap()
        };

        let config = if let Some(ref config) = ctx.config {
            config
        } else {
            let path = package.config();
            ctx.config = Some(Config::from_file(&path)?);
            ctx.config.as_ref().unwrap()
        };

        setup_build_directory(package.build(), config.theme())?;

        let handles = config
            .cursors()
            .to_owned()
            .into_iter()
            .map(|cursor| {
                // Attach context so we know which thread is emitting the events.
                let span = error_span!("", cursor = ?cursor.name());

                let build = package.build().clone();
                let name = cursor.name().to_owned();
                let strict = self.strict;

                let handle = thread::spawn(move || {
                    span.in_scope(move || process_cursor(&cursor, &build, strict))
                });

                (name, handle)
            })
            .collect::<Vec<_>>();

        let mut error_count = 0;
        for (name, handle) in handles {
            match handle.join() {
                Ok(result) => {
                    if let Err(err) = result {
                        let mut error_message = err.to_string();

                        if ctx.level >= VerbosityLevel::Verbose {
                            error_message.push('\n');

                            for cause in err.chain() {
                                _ = writeln!(error_message, "  Cause: {cause}");
                            }
                        }

                        error!("failed to process cursor: {name}: {error_message}");
                        error_count += 1;
                    }
                }
                Err(err) => {
                    // The thread most likely panicked.
                    error!("failed to join on the associated thread: {err:#?}");
                    error_count += 1;
                }
            }
        }

        if error_count > 0 {
            Err(anyhow!("failed to create ({error_count}) cursors"))
        } else {
            let mut stderr = io::stderr();
            writeln!(stderr, "{}", "Successfully built theme!".bold().green())?;

            Ok(())
        }
    }
}

fn setup_build_directory(build: &BuildDir, theme_name: &str) -> anyhow::Result<()> {
    fs::create_dir_all(build.as_path()).context("failed to create build directory")?;
    info!("created directory: {:#}", build.as_path().display());

    let frames = build.frames();
    fs::create_dir_all(&frames).context("failed to create frames directory")?;
    info!("created directory: {:#}", frames.display());

    let theme = build.theme();
    fs::create_dir_all(theme.as_path()).context("failed to create theme directory")?;
    info!("created directory: {:#}", theme.as_path().display());

    let cursors = theme.cursors();
    fs::create_dir_all(&cursors).context("failed to create theme directory")?;
    info!("created directory: {:#}", cursors.display());

    let index_theme = theme.index_theme();
    let contents = format!(
        "[Icon Theme]\n\
        Name = {theme_name}\n\
        Inherits = Adwaita"
    );
    fs::write(&index_theme, &contents).context("failed to create index.theme file")?;
    info!("created file: {:#}", index_theme.display());

    Ok(())
}

fn process_cursor(cursor: &Cursor, build: &BuildDir, strict: bool) -> anyhow::Result<()> {
    let path = path::absolute(cursor.input()).context("failed to resolve cursor input path")?;
    let ani = Ani::open(&path, strict).context("failed to decode ANI file")?;

    let file_stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .context("expected path to be valid unicode")?;

    let mut frames_dir = build.frames();
    frames_dir.push(file_stem);
    let frames_dir = frames_dir;
    fs::create_dir_all(&frames_dir).context("failed to create frame output directory")?;

    let frame_names = extract_frames(&ani, &frames_dir)?;

    let cursor_config_path = frames_dir.join(format!("{file_stem}.cursor"));
    build_xcursor_config(&ani, &frame_names, &cursor_config_path)?;

    let xcursor_output = frames_dir.join(file_stem);
    create_xcursor(&frames_dir, &cursor_config_path, &xcursor_output)
        .context("failed to create Xcursor")?;

    link_to_theme(
        &build.theme().cursors(),
        cursor.name(),
        cursor.aliases(),
        &xcursor_output,
    )?;

    Ok(())
}

fn extract_frames(ani: &Ani, output_dir: &Path) -> anyhow::Result<Vec<String>> {
    let names = (0..ani.frames().len())
        .map(|i| format!("{i:0>2}.png"))
        .collect::<Vec<_>>();

    for (i, frame) in ani.frames().iter().enumerate() {
        let path = output_dir.join(&names[i]);
        let reader = io::Cursor::new(frame);
        let image = image::load(reader, ImageFormat::Ico).context("failed to load frame image")?;
        image.save_with_format(&path, ImageFormat::Png)?;
    }

    Ok(names)
}

#[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn build_xcursor_config(ani: &Ani, frame_names: &[String], output: &Path) -> anyhow::Result<()> {
    let sequence = ani.sequence().map_or_else(
        || {
            info!("ANI sequence missing, using default");
            (0..ani.header().steps())
                .map(|i| i % ani.header().frames())
                .collect()
        },
        ToOwned::to_owned,
    );
    let rates = ani.rates().map_or_else(
        || {
            info!("ANI frame rates missing, using default");
            iter::repeat_n(ani.header().jif_rate(), ani.frames().len()).collect()
        },
        ToOwned::to_owned,
    );

    let mut contents = String::with_capacity(20 * sequence.len());

    for i in sequence {
        let i = usize::try_from(i).context("invalid sequence index")?;
        let frame = &ani.frames()[i];

        // First byte of the ICONDIRENTRY structure.
        // TODO: Move this data to the `ani` crate.
        let width = frame[6];

        let file_name = &frame_names[i];
        let duration = rates[i] * (JIFFY.round() as u32);

        writeln!(
            contents,
            "{size} {x} {y} {file_name} {duration}",
            size = width,
            x = u16::from_le_bytes(frame[10..=11].try_into().unwrap()),
            y = u16::from_le_bytes(frame[12..=13].try_into().unwrap()),
            file_name = file_name,
            duration = duration,
        )?;
    }

    fs::write(output, contents).context("failed to create Xcursor configuration file")?;
    Ok(())
}

fn create_xcursor(frames_dir: &Path, config: &Path, output: &Path) -> anyhow::Result<()> {
    let status = Command::new("xcursorgen")
        .args([config.display().to_string(), output.display().to_string()])
        .current_dir(frames_dir)
        .status()
        .context("failed to execute xcursorgen")?;

    match status.code() {
        Some(0) => {
            info!("created Xcursor: {:#}", output.display());
            Ok(())
        }
        Some(code) => Err(anyhow!("process failed with exit code: {code}")),
        None => Err(anyhow!("process terminated due to signal")),
    }
}

fn link_to_theme(
    theme_cursors_dir: &Path,
    cursor_name: &str,
    aliases: &[String],
    target: &Path,
) -> anyhow::Result<()> {
    let target_link = theme_cursors_dir.join(cursor_name);
    symlink(target, &target_link)?;

    for alias in aliases {
        let alias_link = theme_cursors_dir.join(alias);
        symlink(&target_link, &alias_link)?;
        info!("created alias: {alias}");
    }

    Ok(())
}

pub fn symlink(source: &Path, target: &Path) -> anyhow::Result<()> {
    match fs::remove_file(target) {
        Ok(()) => {}
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {}
            _ => return Err(err).context("failed to remove existing file")?,
        },
    }

    let status = Command::new("ln")
        .args([
            "--symbolic",
            &source.display().to_string(),
            &target.display().to_string(),
        ])
        .status()
        .context("failed to execute ln")?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow!("process failed with exit code: {code}")),
        None => Err(anyhow!("process terminated due to signal")),
    }
}
