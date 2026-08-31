#![doc = include_str!("../README.md")]
#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

use std::io;
use std::io::Write as _;
use std::process::ExitCode;

use clap::Parser as _;
use colored::Colorize as _;

use commands::Parser;

mod commands;
mod manifest_from_inf;
mod ops;

fn main() -> ExitCode {
    try_main().unwrap_or_else(|err| {
        let mut stderr = io::stderr().lock();
        writeln!(stderr, "{}", "ani2xcur failed".bold().red()).ok();

        for cause in err.chain() {
            writeln!(stderr, "  {}: {}", "Cause".bold(), cause).ok();
        }

        ExitCode::FAILURE
    })
}

fn try_main() -> anyhow::Result<ExitCode> {
    let args = Parser::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.verbosity)
        .with_writer(io::stderr)
        .init();

    commands::run(args).map(|()| ExitCode::SUCCESS)
}
