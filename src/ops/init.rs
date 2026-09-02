use std::path::PathBuf;
use std::{io, slice};

use ani2xcur_core::manifest::{Manifest, THEME_DEFAULT};
use ani2xcur_core::package::Package;
use ani2xcur_core::size::Size;
use ani2xcur_core::{CURSORS_DEFAULT, Cursor};
use anyhow::{Context as _, bail};
use inf::{AddRegistryEntry, Entry, Inf, Section, Value};

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

        let Extracted {
            scheme_name,
            cursors,
        } = from_inf(&inf).context("failed to extract required data from INF")?;

        let theme = request
            .theme
            .unwrap_or_else(|| scheme_name.unwrap_or_else(|| THEME_DEFAULT.to_owned()));

        Manifest::new(theme, request.sizes, cursors)
    };

    package
        .save_manifest(manifest)
        .context("failed to save manifest file")?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum FromInfError {
    #[error("section 'DefaultInstall' not found")]
    MissingDefaultInstall,

    #[error("entry 'AddReg' not found")]
    MissingAddReg,

    #[error("entry modifying the cursor scheme registry not found")]
    MissingSchemesEntry,

    #[error("invalid add registry entry")]
    InvalidAddRegEntry(#[source] inf::InvalidAddRegistryEntry),

    #[error("section '{name}' not found")]
    MissingSection { name: String },

    #[error("failed to expand vars")]
    ExpandVars(#[source] inf::util::ExpandVarsError),
}

struct Extracted {
    scheme_name: Option<String>,
    cursors: Vec<Cursor>,
}

fn from_inf(inf: &Inf) -> Result<Extracted, FromInfError> {
    let entry = get_cursor_scheme_entry(inf)?;
    let strings = inf.strings();

    let theme = if entry.value_entry_name.is_empty() {
        None
    } else {
        let theme = inf::util::expand_vars(entry.value_entry_name, &strings)
            .map_err(FromInfError::ExpandVars)?;

        Some(theme)
    };

    let cursors = get_paths(&entry, &strings)?
        .into_iter()
        .enumerate()
        .map(|(index, path)| CURSORS_DEFAULT[index].clone().with_path(path))
        .collect::<Vec<Cursor>>();

    Ok(Extracted {
        scheme_name: theme,
        cursors,
    })
}

fn get_cursor_scheme_entry(inf: &Inf) -> Result<AddRegistryEntry<'_>, FromInfError> {
    let add_registry_sections = get_add_registry_sections(inf)?;

    for section_name in add_registry_sections {
        let section = inf
            .get(section_name)
            .ok_or_else(|| FromInfError::MissingSection {
                name: section_name.clone(),
            })?;

        for entry in section.as_add_registry().entries() {
            let entry = entry.map_err(FromInfError::InvalidAddRegEntry)?;

            if entry.subkey == r"Control Panel\Cursors\Schemes" {
                return Ok(entry);
            }
        }
    }

    Err(FromInfError::MissingSchemesEntry)
}

fn get_add_registry_sections(inf: &Inf) -> Result<&[String], FromInfError> {
    let default_install = inf
        .get("DefaultInstall")
        .ok_or(FromInfError::MissingDefaultInstall)?;

    default_install
        .entries()
        .iter()
        .find_map(|entry| match entry {
            Entry::Item(k, v) if k.as_str() == "AddReg" => match v {
                Value::Raw(value) => Some(slice::from_ref(value)),
                Value::List(values) => Some(values.as_slice()),
            },
            _ => None,
        })
        .ok_or(FromInfError::MissingAddReg)
}

fn get_paths(
    entry: &AddRegistryEntry<'_>,
    strings: &Section,
) -> Result<Vec<PathBuf>, FromInfError> {
    entry
        .value
        .split(',')
        .map(|s| -> Result<_, FromInfError> {
            let path = s
                .split('\\')
                .skip(2)
                .map(|v| inf::util::expand_vars(v, strings).map_err(FromInfError::ExpandVars))
                .collect::<Result<Vec<_>, FromInfError>>()?
                .into_iter()
                .collect::<PathBuf>();

            Ok(path)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    static INF: &str = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
CopyFiles = Scheme.Cur, Scheme.Txt
AddReg = Scheme.Reg

[DestinationDirs]
Scheme.Cur = 10,"%CUR_DIR%"
Scheme.Txt = 10,"%CUR_DIR%"

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,"%10%\%CUR_DIR%\%pointer%,%10%\%CUR_DIR%\%help%,%10%\%CUR_DIR%\%work%,%10%\%CUR_DIR%\%busy%,%10%\%CUR_DIR%\%cross%,%10%\%CUR_DIR%\%Text%,%10%\%CUR_DIR%\%Hand%,%10%\%CUR_DIR%\%unavailable%,%10%\%CUR_DIR%\%Vert%,%10%\%CUR_DIR%\%Horz%,%10%\%CUR_DIR%\%Dgn1%,%10%\%CUR_DIR%\%Dgn2%,%10%\%CUR_DIR%\%move%,%10%\%CUR_DIR%\%alternate%,%10%\%CUR_DIR%\%link%"

[Scheme.Cur]
Pointer.ani
Help.ani
Working.ani
Busy.ani
Crosshair.ani
Text.ani
Hand.ani
Unavailable.ani
Vertical.ani
Horizontal.ani
Diagonal1.ani
Diagonal2.ani
Move.ani
Alternate.ani
Link.ani

[Strings]
CUR_DIR = "Cursors\My Cursor V1"
SCHEME_NAME = "My Cursor V1"
pointer = "Pointer.ani"
help = "Help.ani"
work = "Working.ani"
busy = "Busy.ani"
cross = "Crosshair.ani"
text = "Text.ani"
hand = "Hand.ani"
unavailable = "Unavailable.ani"
vert = "Vertical.ani"
horz = "Horizontal.ani"
dgn1 = "Diagonal1.ani"
dgn2 = "Diagonal2.ani"
move = "Move.ani"
alternate = "Alternate.ani"
link = "Link.ani"
"#;

    #[test]
    fn from_inf() {
        let reader = Cursor::new(INF);
        let inf = Inf::from_reader(reader).unwrap();

        let e = get_cursor_scheme_entry(&inf).unwrap();

        assert_eq!(e.registry_root, "HKCU");
        assert_eq!(e.subkey, r"Control Panel\Cursors\Schemes");
        assert_eq!(e.value_entry_name, "%SCHEME_NAME%");
        assert_eq!(
            e.value,
            r"%10%\%CUR_DIR%\%pointer%,%10%\%CUR_DIR%\%help%,%10%\%CUR_DIR%\%work%,%10%\%CUR_DIR%\%busy%,%10%\%CUR_DIR%\%cross%,%10%\%CUR_DIR%\%Text%,%10%\%CUR_DIR%\%Hand%,%10%\%CUR_DIR%\%unavailable%,%10%\%CUR_DIR%\%Vert%,%10%\%CUR_DIR%\%Horz%,%10%\%CUR_DIR%\%Dgn1%,%10%\%CUR_DIR%\%Dgn2%,%10%\%CUR_DIR%\%move%,%10%\%CUR_DIR%\%alternate%,%10%\%CUR_DIR%\%link%"
        );
    }
}
