mod about;
mod build;
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
    /// Generate the Manifest (Cursor.toml) file
    Init(init::Init),

    /// Create a single Xcursor file from an ANI file
    Convert(convert::Convert),

    /// Convert multiple cursors from a setup information file (INF)
    Build(build::Build),

    /// Make the built cursor theme findable by the X Window System
    Install(install::Install),

    /// Delete the theme and all of its build artifacts
    Uninstall(uninstall::Uninstall),

    /// Read contents of an ANI file
    Inspect(inspect::Inspect),

    /// About the author
    About(about::About),
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
        }
    }
}

pub mod prelude {
    pub use crate::commands::Run;
    pub use crate::context::Context;
}
