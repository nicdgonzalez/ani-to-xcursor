use std::io;
use std::io::Write as _;

use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::prelude::*;
use crate::ops::install::{InstallPackageError, InstallPackageRequest, install_package};

#[derive(Debug, Default, clap::Args)]
pub struct Install {
    /// Run the `init` command with default arguments prior to installing.
    #[arg(long)]
    default_init: bool,
}

impl Run for Install {
    fn run(self, ctx: Context) -> anyhow::Result<()> {
        let request = InstallPackageRequest {
            input: ctx.current_dir,
            default_init: self.default_init,
        };

        match install_package(request) {
            Ok(theme) => {
                writeln!(
                    io::stderr(),
                    "{}",
                    format!("Installed theme {theme:?}").bold().green()
                )
                .ok();

                Ok(())
            }
            Err(InstallPackageError::AlreadyInstalled { theme }) => {
                writeln!(
                    io::stderr(),
                    "{}",
                    format!("Theme {theme:?} already exists").bold().yellow()
                )
                .ok();

                // Since the end goal is the same, we'll consider it completed successfully.
                //
                // TODO: Maybe in the future this condition can have it's own status code.
                Ok(())
            }
            Err(err) => Err(err).context("failed to install package"),
        }
    }
}
