use std::io::Write as _;
use std::process::{Command, Stdio};
use std::{env, fs, io};

use anyhow::{bail, Context as _};
use colored::Colorize as _;

use crate::commands::Run;
use crate::context::Context;

#[derive(Debug, Clone, clap::Args)]
pub struct Init;

impl Run for Init {
    fn run(&self, _ctx: &mut Context) -> anyhow::Result<()> {
        let cwd = env::current_dir().context("failed to get current directory")?;
        let install_inf = cwd.join("Install.inf");
        let cursor_toml = cwd.join("Cursor.toml");

        let child = Command::new("python3")
            .args([
                "-c",
                include_str!("./init.py"),
                "--input",
                install_inf
                    .to_str()
                    .context("expected path to be valid unicode")?,
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to execute python3")?;

        let output = child
            .wait_with_output()
            .context("failed to get output from child process")?;

        if output.stdout.is_empty() {
            bail!("failed to get output from child process");
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        fs::write(&cursor_toml, &text).context("failed to print Cursor.toml contents")?;

        let mut stderr = io::stderr();
        writeln!(stderr, "{}", "Ready!".bold().green())?;

        Ok(())
    }
}
