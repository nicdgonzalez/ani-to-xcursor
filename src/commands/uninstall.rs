use std::io::{self, Write as _};

use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::prelude::*;
use crate::ops::uninstall::{UninstallRequest, uninstall_package};

#[derive(Debug, Default, clap::Args)]
pub struct Uninstall;

impl Run for Uninstall {
    fn run(self, ctx: Context) -> anyhow::Result<()> {
        let request = UninstallRequest {
            path: ctx.current_dir,
        };
        let theme = uninstall_package(request).context("failed to uninstall package")?;

        writeln!(
            io::stderr(),
            "{}",
            format!("Uninstalled theme: {theme}").bold().green()
        )
        .ok();

        Ok(())
    }
}
