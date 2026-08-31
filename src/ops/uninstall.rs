use std::io::ErrorKind;
use std::path::PathBuf;
use std::{fs, io};

use ani2xcur_core::manifest::ManifestError;
use ani2xcur_core::package::Package;
use tracing::info;

use crate::ops::install::get_icons_dir;

/// Request to uninstall a package.
pub struct UninstallRequest {
    pub path: PathBuf,
}

/// Errors that can occur while uninstalling a package.
#[derive(Debug, thiserror::Error)]
pub enum UninstallError {
    #[error("failed to check if package is already initialized")]
    CheckPackageInitialized(#[source] io::Error),

    #[error("not in a package directory")]
    NotAPackage,

    #[error("failed to open manifest file")]
    OpenManifestFailed(#[source] ManifestError),

    #[error("failed to get `icons` directory")]
    GetIconsDir(#[source] anyhow::Error),

    #[error("failed to remove theme from `icons` directory")]
    DeleteThemeFailed(#[source] io::Error),
}

/// Removes a package from the system.
pub fn uninstall_package(request: UninstallRequest) -> Result<String, UninstallError> {
    let package = Package::new(request.path);

    let is_initialized = package
        .is_initialized()
        .map_err(UninstallError::CheckPackageInitialized)?;

    if !is_initialized {
        return Err(UninstallError::NotAPackage);
    }

    let manifest = package
        .manifest()
        .map_err(UninstallError::OpenManifestFailed)?;

    let mut icons = get_icons_dir().map_err(UninstallError::GetIconsDir)?;
    icons.push(manifest.theme());

    match fs::remove_file(&icons) {
        Ok(()) => info!("file deleted: {}", icons.display()),
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(UninstallError::DeleteThemeFailed(err)),
    }

    Ok(manifest.theme().to_owned())
}
