use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, io};

use anyhow::Context as _;
use colored::Colorize;

use crate::commands::Run;
use crate::commands::build::{Build, symlink};
use crate::config::Config;
use crate::context::Context;
use crate::package::Package;

#[derive(Debug, Clone, Default, clap::Args)]
pub struct Install {
    #[clap(long)]
    strict: bool,
}

impl Run for Install {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()> {
        if ctx.package.is_none() {
            let current_dir = env::current_dir().context("failed to get current directory")?;
            ctx.package = Some(Package::new(current_dir));
        }
        let package = ctx.package.as_ref().unwrap();

        if ctx.config.is_none() {
            let path = package.config();
            ctx.config = Some(Config::from_file(&path)?);
        }
        let config = ctx.config.as_ref().unwrap();

        let theme_input = package.build().theme().as_path().to_owned();
        let theme_name = config.theme().to_owned();

        Build::new(self.strict).run(ctx)?;

        install_theme(&theme_input, &theme_name)?;
        print_install_instructions(&theme_name)?;

        Ok(())
    }
}

fn install_theme(theme_input: &Path, theme_name: &str) -> anyhow::Result<()> {
    let mut theme_output = dirs::data_dir().context("failed to get data directory")?;
    theme_output.extend(["icons", theme_name]);

    symlink(theme_input, &theme_output)
        .with_context(|| format!("failed to create symlink to {}", theme_output.display()))?;

    Ok(())
}

fn print_install_instructions(theme_name: &str) -> anyhow::Result<()> {
    let mut stderr = io::stderr();
    let mut stdout = io::stdout();

    writeln!(stderr, "{}", "Successfully installed theme!".bold().green())?;
    writeln!(
        stderr,
        "{}",
        "Use the following command to set the cursor theme:".cyan()
    )?;

    let command = if has_command("gsettings") {
        format!("gsettings set org.gnome.desktop.interface cursor-theme {theme_name:?}")
    } else if has_command("xfconf-query") {
        format!("xfconf-query -c xsettings -p /Gtk/CursorThemeName -s {theme_name:?}")
    } else if has_command("kwriteconfig5") {
        format!("kwriteconfig5 --file kcminputrc --group Mouse --key cursorTheme {theme_name:?}")
    } else {
        "No known theme-setting command detected.".to_owned()
    };

    writeln!(stdout, "  {}", command.bold())?;
    Ok(())
}

fn has_command(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd}"))
        .stdout(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
