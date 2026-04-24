use std::io::{self, Write as _};
use std::path::PathBuf;

use anyhow::Context as _;
use colored::Colorize;

use crate::commands::build::{Build, symlink};
use crate::commands::init::Init;
use crate::commands::prelude::*;
use crate::config::{Config, default_sizes};

#[derive(Debug, Default, clap::Args)]
pub struct Install {
    /// Run the `init` command with default arguments prior to installing.
    #[arg(long)]
    default_init: bool,
}

impl Run for Install {
    fn run(self, ctx: &mut Context) -> anyhow::Result<()> {
        let manifest_path = ctx.package.manifest();

        if !manifest_path.exists() {
            if self.default_init {
                Init {
                    sizes: default_sizes(),
                    ..Default::default()
                }
                .run(ctx)?;
                assert!(manifest_path.exists());
            } else {
                anyhow::bail!("Cursor.toml not found; try running the `init` command first");
            }
        }

        if !ctx.package.build().as_path().exists() {
            Build {}.run(ctx)?;
        }

        let config =
            Config::from_path(&ctx.package.manifest()).context("failed to read manifest file")?;

        let theme = ctx.package.build().theme();
        let theme_name = config.theme();

        let mut target = get_icons_dir().context("failed to get icons directory")?;
        target.push(theme_name);

        if target.try_exists().is_ok_and(|exists| exists) {
            writeln!(
                io::stderr(),
                "{}",
                format!("Theme named {theme_name:?} already exists")
                    .bold()
                    .yellow()
            )
            .ok();
        } else {
            symlink(theme.as_path(), &target)?;
            writeln!(
                io::stderr(),
                "{}",
                format!("Installed theme {theme_name:?}").bold().green()
            )
            .ok();
        }

        Ok(())
    }
}

pub(crate) fn get_icons_dir() -> anyhow::Result<PathBuf> {
    let mut legacy_path = dirs::home_dir().context("failed to get home directory")?;
    legacy_path.push(".icons");

    if legacy_path.exists() {
        return Ok(legacy_path);
    }

    let mut modern = dirs::data_local_dir().context("failed to get data directory")?;
    modern.push("icons");

    Ok(modern)
}
