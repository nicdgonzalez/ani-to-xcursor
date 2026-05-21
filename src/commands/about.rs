use std::io::{self, Write as _};

use colored::Colorize;

use crate::commands::prelude::*;

#[derive(Debug, Default, clap::Args)]
pub struct About;

impl Run for About {
    fn run(self, _ctx: &mut Context) -> anyhow::Result<()> {
        writeln!(
            io::stdout(),
            "\
                {package_name} - Convert Windows animated cursors (.ani) to Linux (.xcur)\n\
                \n\
                Independently developed and maintained by {maintainer}.\n\
                \n\
                {package_name} provides reliable ANI-to-Xcursor conversions with\n\
                configurable build steps and an easy-to-use command-line interface.
                \n\
                If this project was useful to you, consider supporting development\n\
                or starring the repository on GitHub!\n\
                \n\
                {github}: {github_link}\n\
                {support}: {support_link}\n\
                {discord}: {discord_link}\
            ",
            package_name = env!("CARGO_PKG_NAME").bold(),
            maintainer = "@nicdgonzalez".bold(),
            github = "GitHub".yellow(),
            github_link = "https://github.com/nicdgonzalez/ani2xcur".cyan(),
            support = "Support".yellow(),
            support_link = "https://www.buymeacoffee.com/nicdgonzalez".cyan(),
            discord = "Discord".yellow(),
            discord_link = "https://discord.gg/j8aUuMUN39".cyan(),
        )
        .ok();

        Ok(())
    }
}
