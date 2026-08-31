use std::path::{Path, PathBuf};
use std::{fs, io};

use tracing::info;

/// Directory containing the built cursor theme.
#[derive(Debug, Clone)]
pub struct Theme {
    path: PathBuf,
}

impl Theme {
    /// Creates a new theme directory.
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Path to the theme's root directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Path to the directory containing the built cursors.
    #[must_use]
    pub fn cursors(&self) -> PathBuf {
        self.path.join("cursors")
    }

    /// Path to the theme's index file.
    #[must_use]
    pub fn index(&self) -> PathBuf {
        self.path.join("index.theme")
    }

    /// Creates the theme directory plus all of its subdirectories and files.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required files or subdirectories are unable to be created
    /// (e.g., insufficient permissions).
    pub fn initialize(&self, theme_name: &str) -> Result<(), InitializeThemeError> {
        fs::create_dir_all(&self.path).map_err(InitializeThemeError::Base)?;
        info!("created directory: {}", self.path.display());

        let cursors = self.cursors();
        fs::create_dir_all(&cursors).map_err(InitializeThemeError::Cursors)?;
        info!("created directory: {}", cursors.display());

        let index = self.index();
        let index_contents = Self::index_contents(theme_name);
        fs::write(&index, index_contents).map_err(InitializeThemeError::Index)?;
        info!("created file: {}", index.display());

        Ok(())
    }

    fn index_contents(theme_name: &str) -> String {
        format!(
            "[Icon Theme]\n\
            Name = {theme_name}\n\
            Inherits = Adwaita"
        )
    }
}

/// Errors that can occur while trying to initialize the theme directory.
#[derive(Debug, thiserror::Error)]
pub enum InitializeThemeError {
    #[error("failed to create base theme directory")]
    Base(#[source] io::Error),

    #[error("failed to create cursors directory")]
    Cursors(#[source] io::Error),

    #[error("failed to create index file")]
    Index(#[source] io::Error),
}
