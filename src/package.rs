use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Package {
    path: PathBuf,
    build: Build,
}

impl Package {
    pub fn new(path: PathBuf) -> Self {
        let build = Build::new(path.join("build"));
        Self { path, build }
    }

    pub fn config(&self) -> PathBuf {
        self.path.join("Cursor.toml")
    }

    pub const fn build(&self) -> &Build {
        &self.build
    }
}

#[derive(Debug, Clone)]
pub struct Build {
    path: PathBuf,
    theme: Theme,
}

impl Build {
    pub fn new(path: PathBuf) -> Self {
        let theme = Theme::new(path.join("theme"));

        Self { path, theme }
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn frames(&self) -> PathBuf {
        self.path.join("frames")
    }

    pub const fn theme(&self) -> &Theme {
        &self.theme
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    path: PathBuf,
}

impl Theme {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn cursors(&self) -> PathBuf {
        self.path.join("cursors")
    }

    pub fn index_theme(&self) -> PathBuf {
        self.path.join("index.theme")
    }
}
