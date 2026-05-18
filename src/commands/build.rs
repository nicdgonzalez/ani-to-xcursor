use std::io::Write as _;
use std::path::Path;
use std::process::Command;
use std::{io, thread};

use anyhow::Context as _;
use colored::Colorize as _;
use tracing::error_span;

use crate::commands::convert::convert_cursor;
use crate::commands::prelude::*;
use crate::config::{Config, Cursor, Size};
use crate::package::Package;

#[derive(Debug, Default, clap::Args)]
pub struct Build;

impl Run for Build {
    fn run(self, ctx: &mut Context) -> anyhow::Result<()> {
        let manifest_path = ctx.package.manifest();

        if !manifest_path.exists() {
            anyhow::bail!("Cursor.toml not found; try running the `init` command first");
        }

        assert!(manifest_path.exists());
        let config = Config::from_path(&manifest_path).context("failed to read manifest file")?;

        ctx.package
            .theme()
            .create_all(config.theme())
            .context("failed to create theme output directory")?;

        let sizes = config.sizes().to_owned();

        let handles = config
            .cursors()
            .to_owned()
            .into_iter()
            .map(|cursor| {
                // Attach context so we know which thread is emitting the events.
                let span = error_span!("", cursor = ?cursor.name());

                let name = cursor.name().to_owned(); // For better error reporting
                let package = ctx.package.clone();
                let sizes = sizes.clone();

                let handle = thread::spawn(move || {
                    span.in_scope(move || handler(&cursor, &package, &sizes))
                });

                (name, handle)
            })
            .collect::<Vec<_>>();

        let mut error_count = 0;
        for (name, handle) in handles {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    let error = err
                        .chain()
                        .map(|cause| format!("  Cause: {cause}"))
                        .collect::<Vec<_>>()
                        .join("\n");

                    tracing::error!("failed to convert cursor: {name}\n{error}");
                    error_count += 1;
                }
                Err(err) => {
                    // The thread most likely panicked.
                    tracing::error!("failed to join on the associated thread: {err:#?}");
                    error_count += 1;
                }
            }
        }

        if error_count > 0 {
            anyhow::bail!("Failed to create ({error_count}) cursors");
        }

        writeln!(io::stderr(), "{}", "Built theme".bold().green()).ok();
        Ok(())
    }
}

fn handler(cursor: &Cursor, package: &Package, sizes: &[Size]) -> anyhow::Result<()> {
    // Convert from ANI to Xcursor
    let input = package.as_path().join(cursor.input());
    let output = package.theme().cursors().join(cursor.name());
    convert_cursor(&input, sizes, &output).context("failed to create Xcursor")?;

    // Link Xcursor to the theme
    let cursors_dir = package.theme().cursors();
    debug_assert!(cursors_dir.try_exists().is_ok_and(|exists| exists));

    // Add aliases to the theme
    for alias in cursor.aliases() {
        let target = cursors_dir.join(alias);
        symlink(&output, &target).with_context(|| format!("failed to add alias: {alias}"))?;
    }

    Ok(())
}

pub(crate) fn symlink(source: &Path, target: &Path) -> anyhow::Result<()> {
    if target.try_exists().is_ok_and(|exists| exists) {
        return Ok(());
    }

    let status = Command::new("ln")
        .arg("--symbolic")
        .args([source, target])
        .status()
        .context("failed to execute ln")?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(anyhow::anyhow!("process failed with exit code: {code}")),
        None => Err(anyhow::anyhow!("process terminated due to signal")),
    }
}
