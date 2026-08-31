use std::fmt::{Display, Write as _};
use std::fs::{self, File};
use std::io::{self, Read, Write as _};
use std::path::PathBuf;
use std::time::Duration;

use ani::{Ani, Flag};
use anyhow::Context as _;
use bytesize::ByteSize;
use colored::Colorize as _;

use crate::commands::prelude::*;

#[derive(Debug, Default, clap::Args)]
pub struct Inspect {
    /// Path to file to inspect
    input: PathBuf,
}

impl Run for Inspect {
    fn run(self, _: Context) -> anyhow::Result<()> {
        let mut file = File::open(&self.input).context("failed to reopen ANI file")?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .context("failed to read ANI data")?;

        let reader = io::Cursor::new(&mut buffer);
        let ani = Ani::from_reader(reader).context("failed to decode ANI file")?;

        let input = fs::canonicalize(&self.input).expect("expected input file to be valid");
        let name = input.file_name().expect("expected absolute path").display();

        let riff = ani::Parser::new(&buffer)
            .into_iter()
            .next()
            .expect("riff chunk to be present")
            .expect("riff chunk to be valid");
        let size = u64::from(riff.size) + 4; // Add +4 to account for the "RIFF" identifier.

        let mut buffer = String::new();

        buffer.push_str(&summary(&ani, name, size));
        buffer.push('\n');
        buffer.push_str(&diagnostics(&ani));
        buffer.push('\n');
        buffer.push_str(&header(&ani));
        buffer.push('\n');
        buffer.push_str(&frames(&ani));
        buffer.push('\n');
        buffer.push_str(&animation(&ani)?);

        io::stdout().write_all(buffer.as_bytes()).ok();

        Ok(())
    }
}

fn summary(ani: &Ani, name: impl Display, size: u64) -> String {
    let mut buffer = String::new();

    writeln!(buffer, "{}", "Summary".bold().underline()).ok();
    writeln!(buffer, "File: {name}").ok();
    writeln!(buffer, "Size: {}", ByteSize::b(size).display().si()).ok();

    if let Some(metadata) = ani.metadata() {
        if let Some(title) = metadata.title() {
            writeln!(buffer, "Title: {title}").ok();
        }

        if let Some(author) = metadata.author() {
            writeln!(buffer, "Author: {author}").ok();
        }
    }

    buffer
}

fn diagnostics(ani: &Ani) -> String {
    let mut buffer = String::new();

    writeln!(buffer, "{}", "Diagnostics".bold().underline()).ok();

    if ani.rates().is_some() {
        writeln!(buffer, "{}", "✔ Found 'rate' chunk".green()).ok();
    } else {
        writeln!(buffer, "{}", "⚠ Missing 'rate' chunk (optional)".yellow()).ok();
    }

    if ani.sequence().is_some() {
        writeln!(buffer, "{}", "✔ Found 'seq ' chunk".green()).ok();
    } else {
        writeln!(buffer, "{}", "⚠ Missing 'seq ' chunk (optional)".yellow()).ok();
    }

    if ani.header().frames() == u32::try_from(ani.frames().len()).unwrap() {
        writeln!(
            buffer,
            "{}",
            format!(
                "✔ Frame counts match (anih={}, actual={})",
                ani.header().frames(),
                ani.frames().len()
            )
            .green()
        )
        .ok();
    } else {
        writeln!(
            buffer,
            "{}",
            format!(
                "✘ Frame count mismatch (anih={}, actual={})",
                ani.header().frames(),
                ani.frames().len()
            )
            .red()
        )
        .ok();
    }

    buffer
}

fn header(ani: &Ani) -> String {
    let mut buffer = String::new();

    let header = ani.header();
    let frames = header.frames();
    let steps = header.steps();
    let rate = Duration::from_millis(u64::from(header.jif_rate() * 1000 / 60));
    let flags = header.flags();

    let has_flag = |other| {
        if flags.contains(other) {
            "true".green()
        } else {
            "false".red()
        }
    };

    writeln!(buffer, "{}", "Header information".bold().underline()).ok();
    writeln!(buffer, "Frames: {frames}").ok();
    writeln!(buffer, "Steps: {steps}").ok();
    writeln!(buffer, "Default rate: {rate:?}").ok();
    writeln!(buffer, "Flags:").ok();
    writeln!(buffer, "- AF_ICON: {}", has_flag(Flag::ICON)).ok();
    writeln!(buffer, "- AF_SEQUENCE: {}", has_flag(Flag::SEQUENCE)).ok();

    buffer
}

fn frames(ani: &Ani) -> String {
    let mut buffer = String::new();

    writeln!(buffer, "{}", "Frames".bold().underline()).ok();

    for (i, frame) in ani.frames().iter().enumerate() {
        writeln!(buffer, "Frame {i}").ok();

        for (j, image) in frame.iter().enumerate() {
            let j = j + 1; // Start counting from 1.
            let width = image.width();
            let height = image.height();
            let (x, y) = image.cursor_hotspot().expect("expected cursor image");

            writeln!(buffer, "  {j}: {width}x{height}, Hotspot: ({x},{y})").ok();
        }
    }

    buffer
}

fn animation(ani: &Ani) -> anyhow::Result<String> {
    let mut buffer = String::new();

    writeln!(buffer, "{}", "Animation".bold().underline()).ok();

    let rates = ani.rates_or_default();
    let sequence = ani.sequence_or_default();

    for (step, &frame) in sequence.iter().enumerate() {
        let index = usize::try_from(frame).expect("u32 overflowed usize");

        let rate = rates
            .get(index)
            .with_context(|| format!("rate not found at index {index}"))?;
        let duration = Duration::from_millis(u64::from(*rate) * 1000 / 60);

        writeln!(buffer, "{step:>2}: Frame {frame} ({duration:?})").ok();
    }

    writeln!(buffer).ok();

    let total_duration = sequence
        .iter()
        .map(|&i| {
            let index = usize::try_from(i).expect("u32 overflowed usize");
            let rate = rates
                .get(index)
                .with_context(|| format!("rate not found at index {index}"))?;

            Ok(*rate)
        })
        .collect::<anyhow::Result<Vec<u32>>>()?
        .iter()
        .sum::<u32>();
    let duration = Duration::from_millis(u64::from(total_duration) * 1000 / 60);
    writeln!(buffer, "Total Duration: {duration:?}").ok();

    Ok(buffer)
}
