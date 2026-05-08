use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context as _, anyhow};

pub fn default_sizes() -> Vec<Size> {
    Size::VALID_VALUES
        .iter()
        .map(|&value| Size(value))
        .collect()
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    theme: String,

    #[serde(default = "default_sizes")]
    sizes: Vec<Size>,

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
    #[must_use]
    pub fn new(theme: String, sizes: Vec<Size>, cursors: Vec<Cursor>) -> Self {
        Self {
            theme,
            sizes,
            cursors,
        }
    }

    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path).context("failed to read configuration file")?;
        contents.parse()
    }

    #[must_use]
    pub fn theme(&self) -> &str {
        &self.theme
    }

    #[must_use]
    pub fn sizes(&self) -> &[Size] {
        &self.sizes
    }

    #[must_use]
    pub fn cursors(&self) -> &[Cursor] {
        &self.cursors
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Size(pub u8);

impl Size {
    pub const VALID_VALUES: &[u8] = &[24, 32, 48, 64, 96, 128];

    pub fn new(value: u32) -> anyhow::Result<Self> {
        match value {
            24 | 32 | 48 | 64 | 96 | 128 => Ok(Self(value.try_into().unwrap())),
            _ => Err(anyhow!("invalid cursor size {value}")),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Size {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as SerdeError;

        let value = u8::deserialize(deserializer)?;
        Self::new(u32::from(value)).map_err(|err| SerdeError::custom(err.to_string()))
    }
}

impl FromStr for Size {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s
            .parse::<u8>()
            .map_err(|_| anyhow::anyhow!("{s:?} is not a valid number"))?;

        Self::new(u32::from(value))
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
    #[must_use]
    pub fn new(name: String, aliases: Vec<String>, input: PathBuf) -> Self {
        Self {
            name,
            aliases,
            input,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    #[must_use]
    pub fn input(&self) -> &Path {
        &self.input
    }
}
