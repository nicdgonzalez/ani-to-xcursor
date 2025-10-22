use clap::ArgAction;
use tracing::level_filters::LevelFilter;

#[derive(Debug, Clone, clap::Args)]
pub struct Verbosity {
    #[clap(
        short,
        long,
        action = ArgAction::Count,
        help = "Use verbose output (or `-vv` and `-vvv` for more verbose output)",
        global = true,
        overrides_with = "quiet",
    )]
    verbose: u8,

    #[clap(
        short,
        long,
        action = ArgAction::Count,
        help = "Use quiet output (or `-qq` for silent output)",
        global = true,
        overrides_with = "verbose",
    )]
    quiet: u8,
}

impl Verbosity {
    /// Returns a verbosity level based on the number of `-v` and `-q` flags provided.
    pub fn level(&self) -> VerbosityLevel {
        match self.quiet {
            0 => {}
            1 => return VerbosityLevel::Quiet,
            _ => return VerbosityLevel::Silent,
        }

        match self.verbose {
            0 => VerbosityLevel::Default,
            1 => VerbosityLevel::Verbose,
            2 => VerbosityLevel::ExtraVerbose,
            _ => VerbosityLevel::Trace,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum VerbosityLevel {
    /// Silence all logging output.
    Silent,

    /// Shows events up to [`ERROR`](tracing::Level::ERROR).
    Quiet,

    /// Shows events up to [`WARN`](tracing::Level::WARN).
    #[default]
    Default,

    /// Shows events up to [`INFO`](tracing::Level::INFO).
    Verbose,

    /// Shows events up to [`DEBUG`](tracing::Level::DEBUG).
    ExtraVerbose,

    /// Shows events up to [`TRACE`](tracing::Level::TRACE).
    Trace,
}

impl VerbosityLevel {
    pub fn level_filter(self) -> LevelFilter {
        match self {
            Self::Silent => LevelFilter::OFF,
            Self::Quiet => LevelFilter::ERROR,
            Self::Default => LevelFilter::WARN,
            Self::Verbose => LevelFilter::INFO,
            Self::ExtraVerbose => LevelFilter::DEBUG,
            Self::Trace => LevelFilter::TRACE,
        }
    }

    pub fn is_trace(self) -> bool {
        matches!(self, Self::Trace)
    }
}
