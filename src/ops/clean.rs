#![expect(dead_code)]

use std::path::PathBuf;
use std::{fs, io};

use ani2xcur_core::Package;
use tracing::info;

use crate::ops::uninstall::{UninstallError, UninstallRequest, uninstall_package};

/// Request to clean build artifacts.
pub struct CleanRequest {
    pub path: PathBuf,
}

/// Errors that can occur while cleaning up build artifacts.
#[derive(Debug, thiserror::Error)]
pub enum CleanError {
    #[error("failed to check if package is already initialized")]
    CheckPackageInitialized(#[source] io::Error),

    #[error("Cursor.toml not found; aborting")]
    NotInitialized,

    #[error("failed to uninstall theme")]
    Uninstall(#[source] UninstallError),

    #[error("failed to remove theme directory")]
    Theme(#[source] io::Error),

    #[error("failed to remove manifest file")]
    Manifest(#[source] io::Error),
}

/// Removes all build artifacts, reverting a cursor scheme back to its original state.
pub fn clean_artifacts(request: CleanRequest) -> Result<(), CleanError> {
    let package = Package::new(request.path);

    // Don't touch anything if we are not in an initialized package.
    let is_initialized = package
        .is_initialized()
        .map_err(CleanError::CheckPackageInitialized)?;

    if !is_initialized {
        return Err(CleanError::NotInitialized);
    }

    // If the theme is installed in `icons`, uninstall it first.
    let request = UninstallRequest {
        path: package.path().to_owned(),
    };
    _ = uninstall_package(request).map_err(CleanError::Uninstall)?;

    // Delete the theme directory.
    let theme = package.theme();
    fs::remove_dir_all(theme.path()).map_err(CleanError::Theme)?;
    info!("deleted directory: {}", theme.path().display());

    // Delete the package manifest.
    let manifest_path = package.manifest_path();
    fs::remove_file(&manifest_path).map_err(CleanError::Manifest)?;
    info!("deleted file: {}", manifest_path.display());

    Ok(())
}
