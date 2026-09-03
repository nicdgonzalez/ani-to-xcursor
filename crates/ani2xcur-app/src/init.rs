use std::path::PathBuf;
use std::{io, slice};

use ani2xcur_core::{CURSORS_DEFAULT, Cursor, Manifest, Package, THEME_DEFAULT};
use inf::util::expand_vars;
use inf::{AddRegistryEntry, Entry, Inf, Section, Value};

/// Request to initialize a package.
pub struct InitializePackageRequest {
    /// Path to package's target root directory.
    pub path: PathBuf,
    /// Whether to overwrite the existing manifest if it exists.
    pub overwrite: bool,
    /// Path to a Cursor Scheme's setup information (INF) file.
    pub inf: Option<PathBuf>,
    /// Theme name that overrides what may be in the INF file.
    pub theme: Option<String>,
}

/// Errors that can occur while initializing a package.
#[derive(Debug, thiserror::Error)]
pub enum InitializePackageError {
    // Failed to check if the package is already initialized (e.g., insufficient permissions).
    #[error("failed to check if package is already initialized")]
    CheckPackageInitialized(#[source] io::Error),

    /// Package is already initialized.
    #[error("package is already initialized")]
    AlreadyInitialized,

    /// Failed to open the INF file.
    #[error("failed to open INF file")]
    OpenInf(#[source] inf::ParseError),

    /// Failed to extract cursor information from the INF file.
    #[error("failed to parse INF file")]
    ParseInf(#[source] ParseInfError),

    /// Failed to create file handle to write manifest into.
    #[error("failed to create file handle to write manifest into")]
    CreateManifestFile(#[source] io::Error),

    /// Failed to save manifest file.
    #[error("failed to save manifest file")]
    SaveManifest(#[source] io::Error),
}

/// Initializes a new package at the given path.
///
/// A package is considered initialized if it contains a package manifest. This function extracts
/// information from the existing INF file to create the manifest, or generates a generic manifest
/// if no INF file is provided.
///
/// # Errors
///
/// Returns an error if:
///
/// - Unable to check if package is already initialized
/// - Package is already initialized and [`InitializePackageRequest::overwrite`] is `false`
/// - Failed to deserialize INF file
/// - INF file is syntactically valid, but there is an error evaluating it
/// - Failed to save package manifest
pub fn initialize_package(request: InitializePackageRequest) -> Result<(), InitializePackageError> {
    let package = Package::new(request.path);

    let is_initialized = package
        .is_initialized()
        .map_err(InitializePackageError::CheckPackageInitialized)?;

    if is_initialized && !request.overwrite {
        debug_assert!(package.manifest_path().try_exists().unwrap_or(false));
        return Err(InitializePackageError::AlreadyInitialized);
    }

    let inf = request
        .inf
        .map(Inf::open)
        .transpose()
        .map_err(InitializePackageError::OpenInf)?;

    let manifest = create_manifest(inf.as_ref(), request.theme)?;

    manifest
        .save(package.manifest_path())
        .map_err(InitializePackageError::SaveManifest)?;

    Ok(())
}

fn create_manifest(
    inf: Option<&Inf>,
    theme: Option<String>,
) -> Result<Manifest, InitializePackageError> {
    let mut manifest = match inf {
        Some(inf) => manifest_from_inf(inf)?,
        None => Manifest::default(),
    };

    if let Some(theme) = theme {
        manifest = manifest.with_theme(theme);
    }

    Ok(manifest)
}

fn manifest_from_inf(inf: &Inf) -> Result<Manifest, InitializePackageError> {
    let (scheme_name, cursors) = parse_inf(inf).map_err(InitializePackageError::ParseInf)?;
    let theme = scheme_name.unwrap_or_else(|| THEME_DEFAULT.to_owned());
    Ok(Manifest::new(theme, cursors))
}

/// Errors that can occur while extracting cursor scheme information from an INF file.
#[derive(Debug, thiserror::Error)]
pub enum ParseInfError {
    /// `DefaultInstall` section not found.
    #[error("section 'DefaultInstall' not found")]
    DefaultInstallNotFound,

    /// `AddReg` entry not found.
    #[error("entry 'AddReg' not found")]
    AddRegDirectiveNotFound,

    /// Cursor Schemes entry not found in `AddReg`-named section.
    #[error("entry modifying the cursor schemes registry not found")]
    CursorSchemesEntryNotFound,

    /// Entry does not follow the `AddReg` section format.
    ///
    /// <https://learn.microsoft.com/en-us/windows-hardware/drivers/install/inf-addreg-directive>
    #[error("invalid entry")]
    InvalidEntry(#[source] inf::InvalidAddRegistryEntry),

    /// Failed to expand variable.
    #[error("failed to expand variable")]
    ExpandVars(#[source] inf::util::ExpandVarsError),
}

/// Extracts relevant cursor scheme information from `inf`.
fn parse_inf(inf: &Inf) -> Result<(Option<String>, Vec<Cursor>), ParseInfError> {
    let entry = get_cursor_scheme_entry(inf)?;
    let strings = inf.strings();

    let theme = if entry.entry_name.is_empty() {
        None
    } else {
        let theme = expand_vars(entry.entry_name, &strings).map_err(ParseInfError::ExpandVars)?;
        if theme.is_empty() { None } else { Some(theme) }
    };

    let cursors = split_value_into_path_bufs(entry.value, &strings)?
        .into_iter()
        .zip(CURSORS_DEFAULT.iter())
        .filter_map(|(path, cursor)| (!path.is_empty()).then(|| cursor.clone().with_path(path)))
        .collect::<Vec<Cursor>>();

    Ok((theme, cursors))
}

/// Returns the entry that modifies the Cursor Scheme registry.
fn get_cursor_scheme_entry(inf: &Inf) -> Result<AddRegistryEntry<'_>, ParseInfError> {
    // `DefaultInstall` is the main entry point to a setup information (INF) file.
    let default_install = inf
        .get("DefaultInstall")
        .ok_or(ParseInfError::DefaultInstallNotFound)?;

    // The `AddReg` directive lists the names of sections that modify the Windows registry.
    let section_names = find_addreg_entry(default_install)
        .map(|v| match v {
            Value::Raw(value) => slice::from_ref(value),
            Value::List(values) => values.as_slice(),
        })
        .ok_or(ParseInfError::AddRegDirectiveNotFound)?;

    for section_name in section_names {
        let Some(section) = inf.get(section_name) else {
            // No need to validate the INF file; if `AddReg` named a section that does not exist,
            // we can just skip it.
            continue;
        };

        for entry in section.as_add_registry_section().entries() {
            let entry = entry.map_err(ParseInfError::InvalidEntry)?;

            if is_cursor_scheme_entry(entry) {
                return Ok(entry);
            }
        }
    }

    Err(ParseInfError::CursorSchemesEntryNotFound)
}

/// Searches for an item entry whose key matches `AddReg` and returns its value.
fn find_addreg_entry(section: &Section) -> Option<&Value> {
    section.entries().iter().find_map(|e| match e {
        Entry::Item(k, v) if k.as_str() == "AddReg" => Some(v),
        _ => None,
    })
}

/// Checks whether an entry modifies the Cursor Schemes registry.
fn is_cursor_scheme_entry(entry: AddRegistryEntry<'_>) -> bool {
    entry.subkey == r"Control Panel\Cursors\Schemes"
}

/// Splits and expands the variables in the Cursor Scheme entry's `value` field into [`PathBuf`]s.
fn split_value_into_path_bufs(
    value: &str,
    strings: &Section,
) -> Result<Vec<PathBuf>, ParseInfError> {
    value
        .split(',')
        .map(|s| -> Result<_, ParseInfError> {
            let path = s
                .split('\\')
                .skip(2)
                .map(|v| inf::util::expand_vars(v, strings).map_err(ParseInfError::ExpandVars))
                .collect::<Result<Vec<_>, ParseInfError>>()?
                .into_iter()
                .collect::<PathBuf>();

            Ok(path)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use ani2xcur_core::CursorKind;

    use super::*;

    static INF_DEFAULT: &str = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
CopyFiles = Scheme.Cur
AddReg = Scheme.Reg

[DestinationDirs]
Scheme.Cur = 10,"%CUR_DIR%"

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,"%10%\%CUR_DIR%\%pointer%,%10%\%CUR_DIR%\%help%,%10%\%CUR_DIR%\%work%,%10%\%CUR_DIR%\%busy%,%10%\%CUR_DIR%\%cross%,%10%\%CUR_DIR%\%Text%,%10%\%CUR_DIR%\%Hand%,%10%\%CUR_DIR%\%unavailable%,%10%\%CUR_DIR%\%Vert%,%10%\%CUR_DIR%\%Horz%,%10%\%CUR_DIR%\%Dgn1%,%10%\%CUR_DIR%\%Dgn2%,%10%\%CUR_DIR%\%move%,%10%\%CUR_DIR%\%alternate%,%10%\%CUR_DIR%\%link%"

[Scheme.Cur]
"Pointer.ani"
"Help.ani"
"Working.ani"
"Busy.ani"
"Crosshair.ani"
"Text.ani"
"Hand.ani"
"Unavailable.ani"
"Vertical.ani"
"Horizontal.ani"
"Diagonal1.ani"
"Diagonal2.ani"
"Move.ani"
"Alternate.ani"
"Link.ani"

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
    fn manifest_from_inf_parses_ok() {
        let reader = io::Cursor::new(INF_DEFAULT);
        let inf = Inf::from_reader(reader).unwrap();

        let manifest = manifest_from_inf(&inf).unwrap();
        assert_eq!(manifest.theme(), "My Cursor V1".to_owned());
        assert_eq!(manifest.cursors().len(), 15);

        for (cursor, (kind, path_buf)) in manifest.cursors().iter().zip([
            (CursorKind::Default, PathBuf::from("Pointer.ani")),
            (CursorKind::Help, PathBuf::from("Help.ani")),
            (CursorKind::Progress, PathBuf::from("Working.ani")),
            (CursorKind::Wait, PathBuf::from("Busy.ani")),
            (CursorKind::Crosshair, PathBuf::from("Crosshair.ani")),
            (CursorKind::Text, PathBuf::from("Text.ani")),
            (CursorKind::Hand, PathBuf::from("Hand.ani")),
            (CursorKind::Unavailable, PathBuf::from("Unavailable.ani")),
            (CursorKind::NsResize, PathBuf::from("Vertical.ani")),
            (CursorKind::EwResize, PathBuf::from("Horizontal.ani")),
            (CursorKind::NwseResize, PathBuf::from("Diagonal1.ani")),
            (CursorKind::NeswResize, PathBuf::from("Diagonal2.ani")),
            (CursorKind::Move, PathBuf::from("Move.ani")),
            (CursorKind::Alternate, PathBuf::from("Alternate.ani")),
            (CursorKind::Link, PathBuf::from("Link.ani")),
        ]) {
            assert_eq!(cursor.kind(), kind);
            assert_eq!(cursor.path(), path_buf.as_path());
        }
    }

    #[test]
    fn scheme_name_expands() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,

[Strings]
SCHEME_NAME = "My Cursor V1"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let manifest = manifest_from_inf(&inf).unwrap();
        assert_eq!(manifest.theme(), "My Cursor V1".to_owned());
    }

    #[test]
    fn scheme_name_expands_to_empty_returns_default() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,

[Strings]
SCHEME_NAME = ""
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let manifest = manifest_from_inf(&inf).unwrap();
        assert_eq!(manifest.theme(), THEME_DEFAULT.to_owned());
    }

    #[test]
    fn empty_scheme_name_returns_default() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes",,,
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let manifest = manifest_from_inf(&inf).unwrap();
        assert_eq!(manifest.theme(), THEME_DEFAULT.to_owned());
    }

    #[test]
    fn missing_default_install_returns_error() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let error = parse_inf(&inf).unwrap_err();

        assert!(matches!(error, ParseInfError::DefaultInstallNotFound));
    }

    #[test]
    fn missing_addreg_directive_returns_error() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let error = parse_inf(&inf).unwrap_err();

        assert!(matches!(error, ParseInfError::AddRegDirectiveNotFound));
    }

    #[test]
    fn missing_addreg_section_is_skipped() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.DoesNotExist, Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,

[Strings]
SCHEME_NAME = "My Cursor V1"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let (theme, cursors) = parse_inf(&inf).unwrap();

        assert_eq!(theme, Some("My Cursor V1".to_owned()));
        assert_eq!(cursors, vec![]);
    }

    #[test]
    fn no_cursor_scheme_entry_returns_error() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let error = parse_inf(&inf).unwrap_err();

        assert!(matches!(error, ParseInfError::CursorSchemesEntryNotFound));
    }

    #[test]
    fn invalid_cursor_scheme_entry_returns_error() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
; NOTE: The `flags` value is missing. Flags is usually empty, so I can foresee someone
; accidentally forgetting to add that extra comma.
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%","%10%\%CUR_DIR%\%pointer%"

[Strings]
CUR_DIR = "Cursors\My Cursor V1"
SCHEME_NAME = "My Cursor V1"
pointer = "Pointer.ani"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let error = parse_inf(&inf).unwrap_err();

        assert!(matches!(error, ParseInfError::InvalidEntry(_)));
    }

    #[test]
    fn missing_var_in_strings_returns_error() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes",,,"%10%\%CUR_DIR%\%pointer%"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let error = parse_inf(&inf).unwrap_err();

        assert!(matches!(error, ParseInfError::ExpandVars(_)));
    }

    #[test]
    fn unrelated_addreg_entries_are_skipped() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Something\Else",,,
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,

[Strings]
SCHEME_NAME = "My Cursor V1"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let (theme, cursors) = parse_inf(&inf).unwrap();

        assert_eq!(theme, Some("My Cursor V1".to_owned()));
        assert_eq!(cursors, vec![]);
    }

    #[test]
    fn cursor_paths_split_properly() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,"%10%\%CUR_DIR%\%pointer%,%10%\%CUR_DIR%\%help%,%10%\%CUR_DIR%\%work%"

[Strings]
CUR_DIR = "Cursors\My Cursor V1"
SCHEME_NAME = "My Cursor V1"
pointer = "Pointer.ani"
help = "Help.ani"
work = "Working.ani"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let (theme, cursors) = parse_inf(&inf).unwrap();

        assert_eq!(theme, Some("My Cursor V1".to_owned()));

        for (cursor, path_buf) in cursors.iter().zip([
            PathBuf::from("Pointer.ani"),
            PathBuf::from("Help.ani"),
            PathBuf::from("Working.ani"),
        ]) {
            assert_eq!(cursor.path(), path_buf.as_path());
        }
    }

    #[test]
    fn missing_var_in_cursor_path_returns_error() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,"%10%\%CUR_DIR%\%DOES_NOT_EXIST%"

[Strings]
CUR_DIR = "Cursors\My Cursor V1"
SCHEME_NAME = "My Cursor V1"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let error = parse_inf(&inf).unwrap_err();

        assert!(matches!(error, ParseInfError::ExpandVars(_)));
    }

    #[test]
    fn extra_cursors_get_truncated() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
CopyFiles = Scheme.Cur
AddReg = Scheme.Reg

[DestinationDirs]
Scheme.Cur = 10,"%CUR_DIR%"

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,"%10%\%CUR_DIR%\%pointer%,%10%\%CUR_DIR%\%help%,%10%\%CUR_DIR%\%work%,%10%\%CUR_DIR%\%busy%,%10%\%CUR_DIR%\%cross%,%10%\%CUR_DIR%\%Text%,%10%\%CUR_DIR%\%Hand%,%10%\%CUR_DIR%\%unavailable%,%10%\%CUR_DIR%\%Vert%,%10%\%CUR_DIR%\%Horz%,%10%\%CUR_DIR%\%Dgn1%,%10%\%CUR_DIR%\%Dgn2%,%10%\%CUR_DIR%\%move%,%10%\%CUR_DIR%\%alternate%,%10%\%CUR_DIR%\%link%,%10%\%CUR_DIR%\Location.ani,%10%\%CUR_DIR%\Person.ani,%10%\%CUR_DIR%\Extra1.ani,%10%\%CUR_DIR%\Extra2.ani"

[Scheme.Cur]
"Pointer.ani"
"Help.ani"
"Working.ani"
"Busy.ani"
"Crosshair.ani"
"Text.ani"
"Hand.ani"
"Unavailable.ani"
"Vertical.ani"
"Horizontal.ani"
"Diagonal1.ani"
"Diagonal2.ani"
"Move.ani"
"Alternate.ani"
"Link.ani"

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

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let (_, cursors) = parse_inf(&inf).unwrap();

        // Length should be clamped to however many cursor kinds + aliases we have defined.
        assert_eq!(cursors.len(), CURSORS_DEFAULT.len());
    }

    #[test]
    fn empty_cursor_path_skips_that_cursor() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
CopyFiles = Scheme.Cur
AddReg = Scheme.Reg

[DestinationDirs]
Scheme.Cur = 10,"%CUR_DIR%"

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,"%10%\%CUR_DIR%\%pointer%,,%10%\%CUR_DIR%\%work%"

[Scheme.Cur]
"Pointer.ani"
"Working.ani"

[Strings]
CUR_DIR = "Cursors\My Cursor V1"
SCHEME_NAME = "My Cursor V1"
pointer = "Pointer.ani"
work = "Working.ani"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let (_, cursors) = parse_inf(&inf).unwrap();

        for (cursor, expected) in cursors
            .iter()
            .zip([PathBuf::from("Pointer.ani"), PathBuf::from("Working.ani")])
        {
            assert_eq!(cursor.path(), expected.as_path());
        }
    }

    #[test]
    fn explicit_theme_overrides_inf() {
        let buffer = r#"
[Version]
signature="$CHICAGO$"

[DefaultInstall]
AddReg = Scheme.Reg

[Scheme.Reg]
HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,

[Strings]
SCHEME_NAME = "My Cursor V1"
"#;

        let reader = io::Cursor::new(&buffer);
        let inf = Inf::from_reader(reader).unwrap();

        let manifest = create_manifest(Some(&inf), Some("Our Cursor V1".to_owned())).unwrap();

        assert_eq!(manifest.theme(), "Our Cursor V1");
    }

    #[test]
    fn skip_inf_generates_default_manifest() {
        let manifest = create_manifest(None, None).unwrap();
        assert_eq!(manifest.theme(), THEME_DEFAULT);
    }
}
