mod about;
mod build;
mod completions;
mod convert;
mod init;
mod inspect;
mod install;
mod uninstall;

use crate::context::Context;

pub trait Run {
    fn run(self, ctx: &mut Context) -> anyhow::Result<()>;
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Show project and author information
    About(about::About),

    /// Generate the Manifest (Cursor.toml) file
    Init(init::Init),

    /// Convert multiple cursors from a setup information file (INF)
    Build(build::Build),

    /// Make the built cursor theme findable by the X Window System
    Install(install::Install),

    /// Delete the theme and all of its build artifacts
    Uninstall(uninstall::Uninstall),

    /// Create a single Xcursor file from an ANI file
    Convert(convert::Convert),

    /// Read contents of an ANI file
    Inspect(inspect::Inspect),

    /// Generate auto-complete options for your preferred shell
    Completions(completions::Completions),
}

impl Subcommand {
    pub fn run(self, ctx: &mut Context) -> anyhow::Result<()> {
        match self {
            Self::Init(inner) => inner.run(ctx),
            Self::Convert(inner) => inner.run(ctx),
            Self::Build(inner) => inner.run(ctx),
            Self::Install(inner) => inner.run(ctx),
            Self::Uninstall(inner) => inner.run(ctx),
            Self::Inspect(inner) => inner.run(ctx),
            Self::About(inner) => inner.run(ctx),
            Self::Completions(inner) => inner.run(ctx),
        }
    }
}

pub mod prelude {
    pub use crate::commands::Run;
    pub use crate::context::Context;
}
