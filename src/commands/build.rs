use std::io::{self, Write as _};

use ani2xcur_core::Size;
use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::{Context, Run};
use crate::ops::build::{BuildPackageRequest, build_package};

#[derive(Debug, Default, clap::Args)]
pub struct Build {
    #[arg(long, value_delimiter = ',', default_value = "24,32,48,64,96,128")]
    pub sizes: Vec<Size>,
}

impl Run for Build {
    fn run(self, ctx: Context) -> anyhow::Result<()> {
        let request = BuildPackageRequest {
            path: ctx.current_dir,
            sizes: self.sizes,
        };

        build_package(request).context("failed to build package")?;
        writeln!(io::stderr(), "{}", "Theme built".bold().green()).ok();

        Ok(())
    }
}
