use std::io;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

use ani2xcur_core::default_sizes;
use ani2xcur_core::manifest::ManifestError;
use ani2xcur_core::package::Package;
use anyhow::Context as _;

use crate::ops::build::{BuildPackageRequest, build_package};
use ani2xcur_app::init::{InitializePackageError, InitializePackageRequest, initialize_package};

/// Request to install a package.
pub struct InstallPackageRequest {
    pub input: PathBuf,
    pub default_init: bool,
}

/// Errors that can occur while installing a package.
#[derive(Debug, thiserror::Error)]
pub enum InstallPackageError {
    #[error("failed to check if package is already initialized")]
    CheckPackageInitialized(#[source] io::Error),

    #[error("package not initialized; try running the `init` command first")]
    NotInitialized,

    #[error("failed to intitialize package")]
    InitFailed(#[source] InitializePackageError),

    #[error("failed to check if theme already exists")]
    CheckThemeExists(#[source] io::Error),

    #[error("failed to build package")]
    BuildFailed(#[source] anyhow::Error),

    #[error("failed to check if target theme already exists")]
    CheckTargetThemeExists(#[source] io::Error),

    #[error("failed to open manifest file")]
    OpenManifestFailed(#[source] ManifestError),

    #[error("failed to get `icons` directory")]
    GetIconsDir(#[source] anyhow::Error),

    #[error("theme {theme:?} is already installed")]
    AlreadyInstalled { theme: String },

    #[error("failed to symlink target theme")]
    SymlinkFailed(#[source] io::Error),
}

/// Links the cursor theme where the system is able to find it.
pub fn install_package(request: InstallPackageRequest) -> Result<String, InstallPackageError> {
    let package = Package::new(request.input);

    let is_initialized = package
        .is_initialized()
        .map_err(InstallPackageError::CheckPackageInitialized)?;

    if !is_initialized {
        if request.default_init {
            let request = InitializePackageRequest {
                path: package.path().to_owned(),
                overwrite: false,
                inf: None,
                theme: None,
            };

            initialize_package(request).map_err(InstallPackageError::InitFailed)?;
        } else {
            return Err(InstallPackageError::NotInitialized);
        }
    }

    let theme = package.theme();
    let theme_exists = theme
        .path()
        .try_exists()
        .map_err(InstallPackageError::CheckThemeExists)?;

    if !theme_exists {
        let request = BuildPackageRequest {
            path: package.path().to_owned(),
            sizes: default_sizes().to_vec(),
        };

        build_package(request).map_err(InstallPackageError::BuildFailed)?;
    }

    let manifest = package
        .manifest()
        .map_err(InstallPackageError::OpenManifestFailed)?;

    let mut target_theme = get_icons_dir().map_err(InstallPackageError::GetIconsDir)?;
    target_theme.push(manifest.theme());

    let target_theme_exists = target_theme
        .try_exists()
        .map_err(InstallPackageError::CheckTargetThemeExists)?;

    if target_theme_exists {
        Err(InstallPackageError::AlreadyInstalled {
            theme: manifest.theme().to_owned(),
        })
    } else {
        symlink(theme.path(), &target_theme).map_err(InstallPackageError::SymlinkFailed)?;
        Ok(manifest.theme().to_owned())
    }
}

pub(crate) fn get_icons_dir() -> anyhow::Result<PathBuf> {
    let mut legacy_path = dirs::home_dir().context("failed to get home directory")?;
    legacy_path.push(".icons");

    if legacy_path.exists() {
        return Ok(legacy_path);
    }

    let mut modern = dirs::data_local_dir().context("failed to get data directory")?;
    modern.push("icons");

    Ok(modern)
}
