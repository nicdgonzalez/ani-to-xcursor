mod build;
mod init;
mod install;

use crate::context::Context;

pub trait Run {
    fn run(&self, ctx: &mut Context) -> anyhow::Result<()>;
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    Init(init::Init),
    Build(build::Build),
    Install(install::Install),
}

impl Subcommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let handler: &dyn Run = match *self {
            Self::Init(ref inner) => inner,
            Self::Build(ref inner) => inner,
            Self::Install(ref inner) => inner,
        };

        let mut ctx = Context::default();
        handler.run(&mut ctx)
    }
}
