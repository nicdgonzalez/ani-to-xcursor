use std::io::{self, Write as _};

use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::prelude::*;
use crate::ops::clean::{CleanRequest, clean_artifacts};

#[derive(Debug, clap::Args)]
pub struct Clean;

impl Run for Clean {
    fn run(self, ctx: Context) -> anyhow::Result<()> {
        let request = CleanRequest {
            path: ctx.current_dir,
        };

        clean_artifacts(request).context("failed to clean up artifacts")?;

        writeln!(io::stderr(), "{}", "Removed generated files".bold().green()).ok();

        Ok(())
    }
}
