use std::io;
use std::io::Write as _;
use std::path::PathBuf;

use ani2xcur_core::size::Size;
use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::prelude::*;
use crate::ops::init::{InitializeRequest, initialize_package};

#[derive(Debug, Default, clap::Args)]
pub struct Init {
    /// Unique name for the Cursor Theme. Defaults to the name specified in the INF file.
    #[arg(long)]
    pub theme: Option<String>,

    /// Path to INF file. Defaults to `./Install.inf`.
    #[arg(long)]
    pub inf: Option<PathBuf>,

    #[arg(long, value_delimiter = ',', default_value = "32,48,64,96")]
    pub sizes: Vec<Size>,

    /// Overwrite existing Cursor.toml file if it already exists.
    #[arg(long)]
    pub overwrite: bool,

    /// Create a generic Manifest instead of parsing the INF file.
    #[arg(long, conflicts_with = "inf")]
    pub skip_inf: bool,
}

impl Run for Init {
    fn run(self, ctx: Context) -> anyhow::Result<()> {
        let request = InitializeRequest {
            path: ctx.current_dir,
            overwrite: self.overwrite,
            skip_inf: self.skip_inf,
            inf: self.inf,
            theme: self.theme,
            sizes: self.sizes,
        };

        initialize_package(request).context("failed to initialize package")?;
        writeln!(io::stderr(), "{}", "Created Cursor.toml".bold().green()).ok();

        Ok(())
    }
}
