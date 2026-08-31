use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize as _, Deserializer};

use crate::cursor::{CURSORS_DEFAULT, Cursor, TomlCursor};
use crate::size::{Size, TomlSizes, default_sizes};

/// Generic theme name for when the user doesn't provide one.
pub const THEME_DEFAULT: &str = "Unnamed Theme";

#[derive(Debug, Clone)]
pub struct Manifest {
    theme: String,
    sizes: Vec<Size>,
    cursors: Vec<Cursor>,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("failed to open file")]
    Open(#[source] io::Error),

    #[error("failed to read from reader")]
    Read(#[source] io::Error),

    #[error("failed to deserialize buffer")]
    Parse(#[source] toml::de::Error),
}

impl Manifest {
    /// Creates a new manifest.
    #[must_use]
    pub const fn new(theme: String, sizes: Vec<Size>, cursors: Vec<Cursor>) -> Self {
        Self {
            theme,
            sizes,
            cursors,
        }
    }

    /// Reads the file at `path` and parses its contents.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - File cannot be opened (e.g., does not exist, insufficient permissions, etc.)
    /// - Contents cannot be read
    /// - Contents do not contain a valid manifest
    pub fn open<P>(path: P) -> Result<Self, ManifestError>
    where
        P: AsRef<Path>,
    {
        File::open(path.as_ref())
            .map_err(ManifestError::Open)
            .and_then(Self::from_reader)
    }

    /// Reads `reader` into a string, then parses its contents.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - Contents cannot be read
    /// - Contents do not contain a valid manifest
    pub fn from_reader<R>(mut reader: R) -> Result<Self, ManifestError>
    where
        R: Read,
    {
        let mut buffer = String::new();
        reader
            .read_to_string(&mut buffer)
            .map_err(ManifestError::Read)?;

        buffer
            .parse::<TomlManifest>()
            .map_err(ManifestError::Parse)
            .map(Manifest::from)
    }

    /// Target name for the cursor theme.
    #[must_use]
    pub fn theme(&self) -> &str {
        &self.theme
    }

    /// Target name for the cursor theme.
    #[must_use]
    pub fn sizes(&self) -> &[Size] {
        &self.sizes
    }

    /// Cursor mappings.
    #[must_use]
    pub fn cursors(&self) -> &[Cursor] {
        &self.cursors
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            theme: THEME_DEFAULT.to_owned(),
            sizes: default_sizes().to_vec(),
            cursors: CURSORS_DEFAULT.to_vec(),
        }
    }
}

impl From<TomlManifest> for Manifest {
    fn from(value: TomlManifest) -> Self {
        Self {
            theme: value.theme,
            sizes: value.sizes.into_inner(),
            cursors: value.cursors.into_iter().map(Cursor::from).collect(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct TomlManifest {
    #[serde(deserialize_with = "non_empty_string")]
    theme: String,

    #[serde(default)]
    sizes: TomlSizes,

    #[serde(rename = "cursor", default = "Vec::new")]
    cursors: Vec<TomlCursor>,
}

impl FromStr for TomlManifest {
    type Err = toml::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s)
    }
}

fn non_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;

    if value.is_empty() {
        Err(serde::de::Error::custom("string must not be empty"))
    } else {
        Ok(value)
    }
}

impl From<Manifest> for TomlManifest {
    fn from(value: Manifest) -> Self {
        Self {
            theme: value.theme,
            sizes: TomlSizes(value.sizes),
            cursors: value.cursors.into_iter().map(TomlCursor::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn empty_theme() {
        let buffer = r#"
        theme = ""
        sizes = [32, 64]
        "#;

        let reader = Cursor::new(buffer);
        let error = Manifest::from_reader(reader).unwrap_err();

        assert!(matches!(error, ManifestError::Parse(_)));
    }

    #[test]
    fn empty_sizes() {
        let buffer = r#"
        theme = "My Theme"
        sizes = []
        "#;

        let reader = Cursor::new(buffer);
        let error = Manifest::from_reader(reader).unwrap_err();

        assert!(matches!(error, ManifestError::Parse(_)));
    }

    #[test]
    fn empty_cursors() {
        let buffer = r#"
        theme = "My Theme"
        sizes = [32, 64]
        "#;

        let reader = Cursor::new(buffer);
        let manifest = Manifest::from_reader(reader).unwrap();

        assert_eq!(manifest.cursors(), &[]);
    }

    #[test]
    fn invalid_cursor_name() {
        let buffer = r#"
        theme = "My Theme"
        sizes = [32, 64]

        [[cursor]]
        name = "definitely_invalid"
        aliases = []
        input = "Invalid.ani"
        "#;

        let reader = Cursor::new(buffer);
        let error = Manifest::from_reader(reader).unwrap_err();

        assert!(matches!(error, ManifestError::Parse(_)));
    }
}
