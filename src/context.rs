use crate::config::Config;
use crate::package::Package;
use crate::verbosity::VerbosityLevel;

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub config: Option<Config>,
    pub package: Option<Package>,
    pub level: VerbosityLevel,
}

impl Context {
    pub fn with_level(self, level: VerbosityLevel) -> Self {
        Self { level, ..self }
    }
}
