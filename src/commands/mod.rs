mod about;
mod build;
mod clean;
mod completions;
mod convert;
mod init;
mod inspect;
mod install;
mod uninstall;

use std::env;
use std::path::PathBuf;

pub trait Run {
    fn run(self, ctx: Context) -> anyhow::Result<()>;
}

/// Shared state for command execution.
#[derive(Debug, Clone)]
pub struct Context {
    pub current_dir: PathBuf,
}

/// Executes the subcommand.
pub fn run(args: Parser) -> anyhow::Result<()> {
    let current_dir = args
        .directory
        .unwrap_or_else(|| env::current_dir().unwrap_or(PathBuf::from("/")));

    let ctx = Context { current_dir };

    match args.subcommand {
        Subcommand::Completions(cmd) => cmd.run(ctx),
        Subcommand::About(cmd) => cmd.run(ctx),
        Subcommand::Inspect(cmd) => cmd.run(ctx),
        Subcommand::Init(cmd) => cmd.run(ctx),
        Subcommand::Convert(cmd) => cmd.run(ctx),
        Subcommand::Build(cmd) => cmd.run(ctx),
        Subcommand::Install(cmd) => cmd.run(ctx),
        Subcommand::Uninstall(cmd) => cmd.run(ctx),
        Subcommand::Clean(cmd) => cmd.run(ctx),
    }
}

/// Convert Windows animated cursors to Unix-like systems running the X Window System.
#[derive(clap::Parser)]
#[clap(version)]
pub struct Parser {
    #[command(subcommand)]
    pub subcommand: Subcommand,

    /// Change to the specified directory prior to running the command.
    #[clap(long, short = 'C', global = true)]
    pub directory: Option<PathBuf>,

    #[clap(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Generate auto-complete options for your preferred shell.
    #[clap(hide = true)]
    Completions(completions::Completions),

    /// Reads the contents of an ANI file.
    Inspect(inspect::Inspect),

    /// Displays project and author information then exits.
    About(about::About),

    /// Creates the manifest (Cursor.toml) file.
    Init(init::Init),

    /// Creates a single Xcursor file from an ANI file.
    Convert(convert::Convert),

    /// Converts multiple cursors from a setup information file (INF).
    Build(build::Build),

    /// Makes the built cursor theme findable by X.
    Install(install::Install),

    /// Deletes the theme and all of its build artifacts.
    Uninstall(uninstall::Uninstall),

    /// Deletes package artifacts.
    Clean(clean::Clean),
}

pub mod prelude {
    pub use super::{Context, Run};
}
