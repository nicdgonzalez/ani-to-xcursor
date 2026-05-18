use std::io::{ErrorKind, Write as _};
use std::{fs, io};

use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::install::get_icons_dir;
use crate::commands::prelude::*;
use crate::config::Config;

#[derive(Debug, Default, clap::Args)]
pub struct Uninstall;

impl Run for Uninstall {
    fn run(self, ctx: &mut Context) -> anyhow::Result<()> {
        let manifest = ctx.package.manifest();

        // Don't touch anything if we are not in an `ani2xcur` package.
        if !manifest.exists() {
            anyhow::bail!("Cursor.toml file not found in {:#}", manifest.display());
        }

        // Delete all of the build artifacts.
        let build = &ctx.package.theme();

        match fs::remove_dir_all(build.as_path()) {
            Ok(()) => tracing::info!("directory deleted: {:#}", build.as_path().display()),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {}
                _ => Err(err).context("failed to remove build directory")?,
            },
        }

        // Get the name of the theme we created.
        let config = Config::from_path(&manifest).context("failed to read manifest file")?;
        let theme_name = config.theme();

        let mut icons = get_icons_dir().context("failed to get icons directory")?;
        icons.push(theme_name);

        // Delete the theme from the icons directory.
        match fs::remove_dir_all(&icons) {
            Ok(()) => tracing::info!("file deleted: {:#}", icons.display()),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {}
                _ => Err(err).context("failed to remove theme from icons directory")?,
            },
        }

        // Delete the package manifest.
        match fs::remove_file(&manifest) {
            Ok(()) => tracing::info!("file deleted: {:#}", manifest.display()),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => {}
                _ => Err(err).context("failed to remove package manifest")?,
            },
        }

        writeln!(
            io::stderr(),
            "{}",
            format!("Uninstalled theme {theme_name:?}").bold().green()
        )
        .ok();

        Ok(())
    }
}
