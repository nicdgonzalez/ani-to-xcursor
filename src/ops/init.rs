use std::path::PathBuf;
use std::{io, slice};

use ani2xcur_core::cursor::CURSORS_DEFAULT;
use ani2xcur_core::manifest::{Manifest, THEME_DEFAULT};
use ani2xcur_core::package::Package;
use ani2xcur_core::size::Size;
use anyhow::{Context as _, bail};
use inf::{Entry, Inf, Section, Value};
use tracing::error;

/// Request to initialize a package.
pub struct InitializeRequest {
    pub path: PathBuf,
    pub overwrite: bool,
    pub skip_inf: bool,
    pub inf: Option<PathBuf>,
    pub theme: Option<String>,
    pub sizes: Vec<Size>,
}

#[derive(Debug, thiserror::Error)]
pub enum InitializeError {
    #[error("failed to check if package is already initialized")]
    CheckPackageInitialized(#[source] io::Error),
}

pub fn initialize_package(request: InitializeRequest) -> anyhow::Result<()> {
    let package = Package::new(request.path);

    let is_initialized = package
        .is_initialized()
        .context("failed to check if package is already initialized")?;

    if is_initialized && !request.overwrite {
        bail!("Cursor.toml file already exists. Use --overwrite to replace the existing file");
    }

    let manifest = if request.skip_inf {
        Manifest::default()
    } else {
        let path = request
            .inf
            .unwrap_or_else(|| package.path().join("Install.inf"));

        let inf = Inf::open(path).context("failed to parse INF file")?;

        create_manifest_from_inf(&inf, request.theme, request.sizes)
            .context("failed to create config")?
    };

    package
        .save_manifest(manifest)
        .context("failed to save manifest file")?;

    Ok(())
}

fn create_manifest_from_inf(
    inf: &Inf,
    theme_override: Option<String>,
    sizes: Vec<Size>,
) -> anyhow::Result<Manifest> {
    let cursor_scheme_entry = get_cursor_scheme_entry(inf)?;
    let strings = inf
        .get("Strings")
        .context("section 'Strings' not found in INF")?;

    // TODO: Remove when `get_scheme_name` propagates the error - I don't like the "silent" failure
    let theme = theme_override.unwrap_or_else(|| {
        get_scheme_name(cursor_scheme_entry, strings)
            .inspect_err(|err| error!("failed to resolve scheme name: {err}"))
            .unwrap_or_else(|_| THEME_DEFAULT.to_owned())
    });

    let cursors = get_cursor_paths(cursor_scheme_entry, strings)
        .context("failed to get cursor paths")?
        .into_iter()
        .enumerate()
        .map(|(i, path)| CURSORS_DEFAULT[i].clone().with_path(path))
        .collect();

    Ok(Manifest::new(theme, sizes, cursors))
}

fn get_cursor_scheme_entry(inf: &Inf) -> anyhow::Result<&[String]> {
    // The `DefaultInstall` section is required in all INF files;
    // it is the main entry point to the setup file.
    let default_install = inf
        .get("DefaultInstall")
        .context("section 'DefaultInstall' not found in INF")?;

    // The 'AddReg' entry tells us which section(s) define our cursors.
    let values = get_addreg_values(default_install)
        .context("entry 'AddReg' not found in 'DefaultInstall' section")?;

    values
        .iter()
        .find_map(|name| {
            let section = inf.get(name)?;

            section.entries().iter().find_map(|entry| {
                let Entry::Value(Value::List(values)) = entry else {
                    return None;
                };

                values
                    .get(1)
                    .is_some_and(|subkey| subkey == r"Control Panel\Cursors\Schemes")
                    .then_some(values.as_ref())
            })
        })
        .context("cursor scheme entry not found")
}

/// Returns the values of the `AddReg` directive given a `DefaultInstall` section.
///
/// An `AddReg` directive is used to modify or create registry information.
/// Cursor schemes are managed in the registry `Control Panel\Cursors\Schemes`.
///
/// This function returns `None` if the section does not contain an `"AddReg"` entry.
fn get_addreg_values(default_install: &Section) -> Option<&[String]> {
    default_install
        .entries()
        .iter()
        .find_map(|entry| match entry {
            Entry::Item(key, v) if key.as_str() == "AddReg" => match v {
                Value::Raw(value) => Some(slice::from_ref(value)),
                Value::List(values) => Some(values.as_slice()),
            },
            _ => None,
        })
}

fn get_cursor_paths(
    cursor_scheme_entry: &[String],
    strings: &Section,
) -> anyhow::Result<Vec<PathBuf>> {
    let value = cursor_scheme_entry
        .get(4)
        .context("missing value for cursor paths")?;

    let paths = value
        .split_terminator(',')
        .map(|v| -> anyhow::Result<_> {
            // `str::split` always returns a value, even if it's just the original string.
            //
            // TODO: (Delete this when we are no longer assuming the final component is the path to
            // the cursor) - Figure out how to strip the unnecessary parts of the path to leave
            // only the path to the file name.
            let file_name = v.split('\\').next_back().unwrap();

            inf::util::expand_vars(file_name, strings)
                .context("failed to expand cursor path value")
                .map(PathBuf::from)

            // I suspect we can strip the first and second components (`Cursors`
            // and `<Scheme Name>`) and leave the rest as the path to the cursor.
            //
            // ```
            // let value = inf::util::expand_vars(v, strings).context("failed to expand cursor path")?;
            // let path = value.split_terminator('\\').skip(2).collect::<PathBuf>();
            // Ok(path)
            // ```
            //
            // If I can confirm whether the first two components are mandatory, I can replace with
            // the code above.
        })
        .collect::<anyhow::Result<Vec<PathBuf>>>()?;

    Ok(paths)
}

fn get_scheme_name(cursor_scheme_entry: &[String], strings: &Section) -> anyhow::Result<String> {
    let value = cursor_scheme_entry
        .get(2)
        .context("missing value for scheme name")?;
    let scheme_name =
        inf::util::expand_vars(value, strings).context("failed to expand scheme name")?;

    Ok(scheme_name)
}
