use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::str::FromStr;

use serde::{Deserialize as _, Deserializer};

use crate::cursor::{CURSORS_DEFAULT, Cursor, TomlCursor};
use crate::size::TomlSizes;

/// Generic theme name for when the user doesn't provide one.
pub const THEME_DEFAULT: &str = "Unnamed Theme";

#[derive(Debug, Clone)]
pub struct Manifest {
    theme: String,
    cursors: Vec<Cursor>,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("failed to open file")]
    Open(#[source] io::Error),

    #[error("failed to read from reader")]
    Read(#[source] io::Error),

    #[error("failed to deserialize buffer")]
    Deserialize(#[source] toml::de::Error),
}

impl Manifest {
    /// Creates a new manifest.
    #[must_use]
    pub const fn new(theme: String, cursors: Vec<Cursor>) -> Self {
        Self { theme, cursors }
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
            .map_err(ManifestError::Deserialize)
            .map(Manifest::from)
    }

    /// Creates a new manifest with the given `theme` from an existing manifest.
    #[must_use]
    pub fn with_theme(self, theme: String) -> Self {
        Self { theme, ..self }
    }

    /// Creates a new manifest with the given `cursors` from an existing manifest.
    #[must_use]
    pub fn with_cursors(self, cursors: Vec<Cursor>) -> Self {
        Self { cursors, ..self }
    }

    /// Writes the manifest to `writer` in TOML format.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if we are unable to write into the writer.
    #[expect(clippy::missing_panics_doc, reason = "lib's invariant to uphold")]
    pub fn write<W>(self, mut writer: W) -> io::Result<()>
    where
        W: Write,
    {
        let value = TomlManifest::from(self);
        let contents = toml::to_string_pretty(&value).expect("manifest not serializable");
        writer.write_all(contents.as_bytes())
    }

    /// Writes the manifest to the file at `path` in TOML format.
    ///
    /// This is a convenience function for opening a file and calling [`Self::write`] on it.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if we are unable to write to the file at `path` (e.g., insufficient
    /// permissions).
    pub fn save<P>(self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        File::open(path.as_ref()).and_then(|f| self.write(f))
    }

    /// Target name for the cursor theme.
    #[must_use]
    pub fn theme(&self) -> &str {
        &self.theme
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
            cursors: CURSORS_DEFAULT.to_vec(),
        }
    }
}

impl From<TomlManifest> for Manifest {
    fn from(value: TomlManifest) -> Self {
        Self {
            theme: value.theme,
            cursors: value.cursors.into_iter().map(Cursor::from).collect(),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct TomlManifest {
    #[serde(deserialize_with = "non_empty_string")]
    theme: String,

    // Plan to remove this in a future version; sizes will be an argument to `build` instead.
    #[expect(dead_code, reason = "will be removed eventually")]
    #[serde(default, skip_serializing)]
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
            sizes: TomlSizes(vec![]),
            cursors: value.cursors.into_iter().map(TomlCursor::from).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn empty_theme_returns_error() {
        let buffer = r#"
        theme = ""
        sizes = [32, 64]
        "#;

        let reader = Cursor::new(buffer);
        let error = Manifest::from_reader(reader).unwrap_err();

        assert!(matches!(error, ManifestError::Deserialize(_)));
    }

    #[test]
    fn empty_sizes_returns_error() {
        let buffer = r#"
        theme = "My Theme"
        sizes = []
        "#;

        let reader = Cursor::new(buffer);
        let error = Manifest::from_reader(reader).unwrap_err();

        assert!(matches!(error, ManifestError::Deserialize(_)));
    }

    #[test]
    fn empty_cursors_is_ok() {
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

        assert!(matches!(error, ManifestError::Deserialize(_)));
    }

    #[test]
    fn missing_sizes_parses_ok() {
        let buffer = r#"
        theme = "My Theme"
        "#;

        let reader = Cursor::new(buffer);
        let manifest = Manifest::from_reader(reader).unwrap();

        assert_eq!(manifest.theme, "My Theme".to_owned());
        assert_eq!(manifest.cursors, vec![]);
    }
}
