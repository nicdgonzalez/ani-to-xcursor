use crate::config::Config;
use crate::package::Package;

#[derive(Debug, Clone, Default)]
pub struct Context {
    pub config: Option<Config>,
    pub package: Option<Package>,
}
