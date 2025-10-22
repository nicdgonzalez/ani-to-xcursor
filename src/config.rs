use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Context as _;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    theme: String,

    #[serde(rename = "cursor")]
    cursors: Vec<Cursor>,
}

impl FromStr for Config {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).context("failed to parse configuration")
    }
}

impl Config {
    pub fn from_file(path: &Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path).context("failed to read configuration file")?;
        contents.parse()
    }

    pub fn theme(&self) -> &str {
        &self.theme
    }

    pub fn cursors(&self) -> &[Cursor] {
        &self.cursors
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    name: String,

    #[serde(default = "Vec::new")]
    aliases: Vec<String>,

    input: PathBuf,
}

impl Cursor {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    pub fn input(&self) -> &Path {
        &self.input
    }
}
