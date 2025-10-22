use crate::commands::Run;
use crate::context::Context;

#[derive(Debug, Clone, clap::Args)]
pub struct Init;

impl Run for Init {
    fn run(&self, _ctx: &mut Context) -> anyhow::Result<()> {
        todo!()
    }
}
