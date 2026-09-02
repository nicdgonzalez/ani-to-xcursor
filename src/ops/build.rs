use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::thread;

use ani::Ani;
use ani2xcur_core::cursor::Cursor;
use ani2xcur_core::package::Package;
use ani2xcur_core::size::Size;
use anyhow::{Context, bail};
use tracing::{error, error_span};

use crate::ops::convert::{ConvertCursorRequest, convert_cursor};

pub struct BuildPackageRequest {
    pub path: PathBuf,
    pub sizes: Vec<Size>,
}

pub fn build_package(request: BuildPackageRequest) -> anyhow::Result<()> {
    let package = Package::new(request.path);

    let is_initialized = package
        .is_initialized()
        .context("failed to check if package is already initialized")?;

    if !is_initialized {
        bail!("Package not initialized; try running the `init` command first");
    }

    let manifest = package.manifest().context("failed to open manifest")?;

    package
        .theme()
        .initialize(manifest.theme())
        .context("failed to create theme output directory")?;

    thread::scope(|scope| {
        let handles = manifest
            .cursors()
            .iter()
            .map(|cursor| {
                // Attach context so we know which thread is emitting the events.
                let span = error_span!("", cursor = ?cursor.kind());

                let package = &package;
                let sizes = &request.sizes;
                let handle = scope.spawn(move || {
                    span.in_scope(move || build_package_handler(cursor, package, sizes))
                });

                (cursor.kind(), handle)
            })
            .collect::<Vec<_>>();

        let errors = handles
            .into_iter()
            .filter_map(|(name, handle)| match handle.join() {
                Ok(Ok(())) => None,
                Ok(Err(err)) => Some((name, err)),
                Err(err) => panic!("thread for {name:?} panicked: {err:?}"),
            })
            .collect::<Vec<_>>();

        let error_count = errors.len();

        for (name, error) in errors {
            let chain = error
                .chain()
                .map(|cause| format!("  Cause: {cause}"))
                .collect::<Vec<_>>()
                .join("\n");

            error!("failed to convert cursor: {name}:\n{chain}");
        }

        if error_count > 0 {
            bail!("failed to create ({error_count}) cursors");
        }

        Ok(())
    })
}

fn build_package_handler(cursor: &Cursor, package: &Package, sizes: &[Size]) -> anyhow::Result<()> {
    let input = package.path().join(cursor.path());
    let ani = Ani::open(&input).context("failed to decode ANI file")?;

    let request = ConvertCursorRequest { ani: &ani, sizes };
    let xcursor = convert_cursor(request).context("failed to convert cursor")?;

    let cursors = package.theme().cursors();
    let output = cursors.join(cursor.kind().to_string());

    assert!(
        cursors.try_exists().unwrap_or(false),
        "expected cursors directory to exist"
    );

    xcursor.save(&output).context("failed to save Xcursor")?;

    let missing_aliases = cursor
        .aliases()
        .iter()
        .map(|alias| cursors.join(alias))
        .filter(|path| !path.try_exists().unwrap_or(false));

    for alias in missing_aliases {
        symlink(&output, &alias).context("failed to symlink alias")?;
    }

    Ok(())
}
