//! The main entry point to the application.

#![warn(
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic
)]

mod commands;
mod config;
mod context;
mod package;
mod verbosity;

use std::io::Write as _;
use std::process::ExitCode;
use std::{env, io, panic};

use clap::Parser as _;
use colored::Colorize as _;
use tracing_subscriber::EnvFilter;

use crate::context::Context;
use crate::verbosity::{Verbosity, VerbosityLevel};

#[derive(Debug, clap::Parser)]
#[clap(
    about = "Convert Windows animated cursors to Linux",
    after_help = format!("{}: {}", "Repository".bold(), env!("CARGO_PKG_REPOSITORY")),
    version,
)]
struct Parser {
    #[clap(subcommand)]
    subcommand: commands::Subcommand,

    #[clap(flatten)]
    verbosity: Verbosity,
}

fn main() -> ExitCode {
    try_main().unwrap_or_else(|err| {
        let mut stderr = io::stderr().lock();
        _ = writeln!(stderr, "{}", "ani-to-xcursor failed".bold().red());

        for cause in err.chain() {
            _ = writeln!(stderr, "  {}: {}", "Cause".bold(), cause);
        }

        ExitCode::FAILURE
    })
}

fn try_main() -> anyhow::Result<ExitCode> {
    setup_panic_hook();

    let args = Parser::parse();
    let level = args.verbosity.level();
    setup_tracing(level);

    let mut ctx = Context::default();
    ctx = ctx.with_level(level);
    args.subcommand.run(&mut ctx).map(|()| ExitCode::SUCCESS)
}

fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        original_hook(panic_info);

        let package_name = env!("CARGO_PKG_NAME");
        let repository = env!("CARGO_PKG_REPOSITORY");
        let operating_system = env::consts::OS;
        let architecture = env::consts::ARCH;
        let package_version = env!("CARGO_PKG_VERSION");
        let args = env::args().collect::<Vec<_>>();

        eprintln!();
        eprintln!("------------------------------------------------------------------------------");
        eprintln!("{package_name} has panicked. This is a bug. Please report this at:");
        eprintln!("  {repository}/issues/new");
        eprintln!();
        eprintln!("If you can reliably reproduce this panic, include the reproduction steps");
        eprintln!("and re-run with the RUST_BACKTRACE=1 environment variable set. Please include");
        eprintln!("the backtrace in your report.");
        eprintln!();
        eprintln!("Thank you for taking the time to help us make {package_name} better!");
        eprintln!();
        eprintln!("Platform: {operating_system} {architecture}");
        eprintln!("Version: {package_version}");
        eprintln!("Args: {args:?}");
        eprintln!("------------------------------------------------------------------------------");
    }));
}

fn setup_tracing(level: VerbosityLevel) {
    use tracing_subscriber::prelude::*;

    let level_filter = level.level_filter();
    let filter = EnvFilter::default()
        .add_directive(format!("ani={level_filter}").parse().unwrap())
        .add_directive(
            format!("{}={level_filter}", env!("CARGO_CRATE_NAME"))
                .parse()
                .unwrap(),
        );

    let registry = tracing_subscriber::registry().with(filter);

    if level.is_trace() {
        let subscriber = registry.with(
            tracing_subscriber::fmt::layer()
                .event_format(tracing_subscriber::fmt::format().pretty())
                .with_thread_ids(true)
                .with_writer(io::stderr),
        );

        subscriber.init();
    } else {
        let subscriber = registry.with(tracing_subscriber::fmt::layer().with_writer(io::stderr));

        subscriber.init();
    }
}
