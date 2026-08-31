use std::io::{self, Write as _};
use std::path::PathBuf;

use ani::Ani;
use ani2xcur_core::size::Size;
use anyhow::Context as _;
use colored::Colorize as _;

use crate::commands::{Context, Run};
use crate::ops::convert::{ConvertCursorRequest, convert_cursor};

#[derive(Debug, Default, clap::Args)]
pub struct Convert {
    pub input: PathBuf,
    pub output: Option<PathBuf>,

    #[arg(long, value_delimiter = ',', default_value = "32,48,64,96")]
    pub sizes: Vec<Size>,
}

impl Run for Convert {
    fn run(self, ctx: Context) -> anyhow::Result<()> {
        let input = ctx.current_dir.join(&self.input);
        let file_stem = input
            .file_stem()
            .and_then(|f| f.to_str())
            .context("file name is empty or contains invalid Unicode")?;
        let output = ctx
            .current_dir
            .join(self.output.unwrap_or_else(|| PathBuf::from(file_stem)));

        let ani = Ani::open(input).context("failed to decode ANI file")?;

        let request = ConvertCursorRequest {
            ani: &ani,
            sizes: &self.sizes,
        };
        let xcursor = convert_cursor(request).context("failed to convert cursor")?;

        xcursor.save(&output).context("failed to save Xcursor")?;

        writeln!(
            io::stderr(),
            "{}: {:#}",
            "Created Xcursor".bold().green(),
            output.display()
        )
        .ok();

        Ok(())
    }
}
