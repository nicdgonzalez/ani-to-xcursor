use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use tracing::info;

/// Represents a Windows "Cursor Scheme", manifest (`Cursor.toml`) and all related files.
///
/// # Examples
///
/// ```
/// # use std::{env, error};
/// # use crate::package::Package;
/// # fn main() -> Result<(), Box<dyn error::Error>> {
/// let current_dir = env::current_dir()?;
/// let package = Package::new(current_dir.clone());
/// assert_eq!(package.as_path(), &current_dir);
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Package {
    base: PathBuf,
}

impl Package {
    #[must_use]
    pub const fn new(base: PathBuf) -> Self {
        Self { base }
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.base
    }

    #[must_use]
    pub fn manifest(&self) -> PathBuf {
        self.base.join("Cursor.toml")
    }

    #[must_use]
    pub fn theme(&self) -> Theme {
        Theme::new(self.base.join("theme"))
    }
}

/// Represents the `theme` directory of a [Build].
#[derive(Debug)]
pub struct Theme {
    base: PathBuf,
}

impl Theme {
    #[must_use]
    pub const fn new(base: PathBuf) -> Self {
        Self { base }
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.base
    }

    #[must_use]
    pub fn cursors(&self) -> PathBuf {
        self.base.join("cursors")
    }

    #[must_use]
    pub fn index_theme(&self) -> PathBuf {
        self.base.join("index.theme")
    }

    pub fn create_all(&self, theme_name: &str) -> anyhow::Result<()> {
        fs::create_dir_all(self.as_path()).context("failed to create theme directory")?;
        info!("created directory: {:#}", self.as_path().display());

        let cursors = self.cursors();
        fs::create_dir_all(&cursors).context("failed to create theme directory")?;
        info!("created directory: {:#}", cursors.display());

        let index_theme = self.index_theme();
        let contents = format!(
            "[Icon Theme]\n\
            Name = {theme_name}\n\
            Inherits = Adwaita"
        );
        fs::write(&index_theme, &contents).context("failed to create index.theme file")?;
        info!("created file: {:#}", index_theme.display());

        Ok(())
    }
}
