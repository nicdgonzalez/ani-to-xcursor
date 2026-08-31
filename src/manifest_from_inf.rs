//! Temporary module for experimenting with a [`ani2xcur_core::Manifest::from_inf`] implementation.

use std::slice;

use ani2xcur_core::Manifest;
use inf::{Entry, Inf, Value};

#[derive(Debug, thiserror::Error)]
pub enum FromInfError {
    #[error("section 'DefaultInstall' not found")]
    MissingDefaultInstall,

    #[error("entry 'AddReg' not found")]
    MissingAddReg,

    #[error("entry modifying the cursor scheme registry not found")]
    MissingCursorSchemeEntry,

    #[error("missing value in cursor scheme entry")]
    InvalidCursorSchemeEntry,
}

pub fn manifest_from_inf(inf: &Inf) -> Result<Manifest, FromInfError> {
    // Read the `DefaultInstall` section, which is the main entry point to the file.
    //
    // The `AddReg` directive is used to modify or create registry information. Since we know that
    // Cursor Schemes are managed under the `Control Panel\Cursors\Schemes` registry, we can search
    // each section listed in the `AddReg` entry and see which one defines the cursors. Once found,
    // we can use this information to fill out our manifest file.
    let cursor_scheme_entry = get_cursor_scheme_entry(inf)?;
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorSchemeEntry<'a> {
    key: &'a str,
    subkey: &'a str,
    scheme_name: &'a str,
    _empty: &'a str,
    cursors: &'a str,
}

fn get_cursor_scheme_entry(inf: &Inf) -> Result<CursorSchemeEntry, FromInfError> {
    // The `DefaultInstall` section is the main entry point to the file.
    let default_install = inf
        .get("DefaultInstall")
        .ok_or(FromInfError::MissingDefaultInstall)?;

    // The `AddReg` directive is used to modify registry information. The registry requires
    // cursor paths to be placed in a specific order, so we can use this information to match
    // expected cursors to file names.
    let addreg = default_install
        .entries()
        .iter()
        .find_map(|entry| match entry {
            Entry::Item(k, v) if k.as_str() == "AddReg" => match v {
                Value::Raw(value) => Some(slice::from_ref(value)),
                Value::List(values) => Some(values.as_slice()),
            },
            _ => None,
        })
        .ok_or(FromInfError::MissingAddReg)?;

    let values = addreg
        .iter()
        .find_map(|section_name| {
            let section = inf.get(section_name)?;

            section.entries().iter().find_map(|entry| {
                // A valid cursor scheme registry entry has 5 values, so `Value::List` is required.
                let Entry::Value(Value::List(values)) = entry else {
                    return None;
                };

                values
                    .get(1)
                    .is_some_and(|subkey| subkey == r"Control Panel\Cursors\Schemes")
                    .then_some::<&[String]>(values.as_ref())
            })
        })
        .ok_or(FromInfError::MissingCursorSchemeEntry)?;

    let mut v = values.iter();

    Ok(CursorSchemeEntry {
        key: v.next().ok_or(FromInfError::InvalidCursorSchemeEntry)?,
        subkey: v.next().ok_or(FromInfError::InvalidCursorSchemeEntry)?,
        scheme_name: v.next().ok_or(FromInfError::InvalidCursorSchemeEntry)?,
        _empty: v.next().ok_or(FromInfError::InvalidCursorSchemeEntry)?,
        cursors: v.next().ok_or(FromInfError::InvalidCursorSchemeEntry)?,
    })
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

        let cursor_scheme_entry = get_cursor_scheme_entry(&inf).unwrap();
        assert_eq!(
            cursor_scheme_entry,
            CursorSchemeEntry {
                key: "HKCU",
                subkey: r"Control Panel\Cursors\Schemes",
                scheme_name: "%SCHEME_NAME%",
                _empty: "",
                cursors: r"%10%\%CUR_DIR%\%pointer%,%10%\%CUR_DIR%\%help%,%10%\%CUR_DIR%\%work%,%10%\%CUR_DIR%\%busy%,%10%\%CUR_DIR%\%cross%,%10%\%CUR_DIR%\%Text%,%10%\%CUR_DIR%\%Hand%,%10%\%CUR_DIR%\%unavailable%,%10%\%CUR_DIR%\%Vert%,%10%\%CUR_DIR%\%Horz%,%10%\%CUR_DIR%\%Dgn1%,%10%\%CUR_DIR%\%Dgn2%,%10%\%CUR_DIR%\%move%,%10%\%CUR_DIR%\%alternate%,%10%\%CUR_DIR%\%link%"
            }
        );
    }
}
