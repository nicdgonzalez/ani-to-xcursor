use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use serde::{Deserialize as _, Deserializer};

// Used when converting strings or indices to `CursorKind`.
pub use strum::ParseError;

/// Cursors are ordered the same as they appear in the Windows Registry.
///
/// If your theme builds successfully but a cursor is not showing up as expected, it is likely
/// because the names here don't match what your system/application is looking for when displaying
/// the cursor. To fix it, you need to find out which cursor name you need and add it as an alias
/// to the respective cursor.
///
/// NOTE: Each alias needs to be _unique_. Duplicate names will cause the cursor theme building
/// process to fail with errors involving symbolic links.
///
/// I was unable to find any /official/ Windows to X cursor mappings, so the following is
/// a combination of various resources, including other similar projects such as:
///
/// - [JeffHathford/cursor_win2lin]
/// - [quantum5/win2xcur]
///
/// [JeffHathford/cursor_win2lin]: https://github.com/JeffHathford/cursor_win2lin/blob/7afd265c16463c5dfe28abff0d86a81ea5275b37/mappings.txt
/// [quantum5/win2xcur]: https://github.com/quantum5/win2xcur/blob/feadbe284f502387b6d00fdd688138f6b0faa202/win2xcur/theme.py
pub static CURSORS_DEFAULT: LazyLock<[Cursor; 17]> = LazyLock::new(|| {
    [
        // Arrow
        Cursor {
            kind: CursorKind::Default,
            aliases: vec![
                "arrow".to_owned(),
                "left_ptr".to_owned(),
                "top_left_arrow".to_owned(),
                "X_cursor".to_owned(),
                "mouse".to_owned(),
            ],
            path: PathBuf::from("Default.ani"),
        },
        // Help
        Cursor {
            kind: CursorKind::Help,
            aliases: vec![
                "question_arrow".to_owned(),
                "whats_this".to_owned(),
                "left_ptr_help".to_owned(),
                "5c6cd98b3f3ebcb1f9c7f1c204630408".to_owned(),
                "d9ce0ab605698f320427677b458ad60b".to_owned(),
            ],
            path: PathBuf::from("Help.ani"),
        },
        // AppStarting
        Cursor {
            kind: CursorKind::Progress,
            aliases: vec!["watch".to_owned()],
            path: PathBuf::from("Working.ani"),
        },
        // Wait
        Cursor {
            kind: CursorKind::Wait,
            aliases: vec![
                "half-busy".to_owned(),
                "left_ptr_watch".to_owned(),
                "3ecb610c1bf2410f44200f48c40d3599".to_owned(),
                "08e8e1c95fe2fc01f976f1e063a24ccd".to_owned(),
                "00000000000000020006000e7e9ffc3f".to_owned(),
            ],
            path: PathBuf::from("Busy.ani"),
        },
        // Crosshair
        Cursor {
            kind: CursorKind::Crosshair,
            aliases: vec![
                "cross".to_owned(),
                "cross_reverse".to_owned(),
                "diamond_cross".to_owned(),
                "tcross".to_owned(),
                "plus".to_owned(),
            ],
            path: PathBuf::from("Crosshair.ani"),
        },
        // IBeam
        Cursor {
            kind: CursorKind::Text,
            aliases: vec!["xterm".to_owned(), "ibeam".to_owned()],
            path: PathBuf::from("Text.ani"),
        },
        // NWPen
        Cursor {
            kind: CursorKind::Hand,
            aliases: vec!["pencil".to_owned(), "draft".to_owned()],
            path: PathBuf::from("Hand.ani"),
        },
        // No
        Cursor {
            kind: CursorKind::Unavailable,
            aliases: vec![
                "not-allowed".to_owned(),
                "no-drop".to_owned(),
                "dnd-no-drop".to_owned(),
                "circle".to_owned(),
                "crossed_circle".to_owned(),
                "forbidden".to_owned(),
                "03b6e0fcb3499374a867c041f52298f0".to_owned(),
            ],
            path: PathBuf::from("Unavailable.ani"),
        },
        // SizeNS
        Cursor {
            kind: CursorKind::NsResize,
            aliases: vec![
                "top_side".to_owned(),
                "bottom_side".to_owned(),
                "n-resize".to_owned(),
                "s-resize".to_owned(),
                "row-resize".to_owned(),
                "size_ver".to_owned(),
                "size-ver".to_owned(),
                "split_v".to_owned(),
                "double_arrow".to_owned(),
                "v_double_arrow".to_owned(),
                "sb_v_double_arrow".to_owned(),
                "00008160000006810000408080010102".to_owned(),
                "2870a09082c103050810ffdffffe0204".to_owned(),
            ],
            path: PathBuf::from("Vertical.ani"),
        },
        // SizeWE
        Cursor {
            kind: CursorKind::EwResize,
            aliases: vec![
                "left_side".to_owned(),
                "right_side".to_owned(),
                "sb_h_double_arrow".to_owned(),
                "w-resize".to_owned(),
                "e-resize".to_owned(),
                "size_hor".to_owned(),
                "h_double_arrow".to_owned(),
                "size-hor".to_owned(),
                "col-resize".to_owned(),
                "split_h".to_owned(),
                "14fef782d02440884392942c11205230".to_owned(),
                "028006030e0e7ebffc7f7070c0600140".to_owned(),
            ],
            path: PathBuf::from("Horizontal.ani"),
        },
        // SizeNWSE
        Cursor {
            kind: CursorKind::NwseResize,
            aliases: vec![
                "bd_double_arrow".to_owned(),
                "bottom_right_corner".to_owned(),
                "top_left_corner".to_owned(),
                "se-resize".to_owned(),
                "nw-resize".to_owned(),
                "size_fdiag".to_owned(),
            ],

            path: PathBuf::from("Diagonal1.ani"),
        },
        // SizeNESW
        Cursor {
            kind: CursorKind::NeswResize,
            aliases: vec![
                "bottom_left_corner".to_owned(),
                "fd_double_arrow".to_owned(),
                "top_right_corner".to_owned(),
                "sw-resize".to_owned(),
                "ne-resize".to_owned(),
                "size_bdiag".to_owned(),
                "size-bdiag".to_owned(),
                "fcf1c3c7cd4491d801f1e1c78f100000".to_owned(),
            ],

            path: PathBuf::from("Diagonal2.ani"),
        },
        // SizeAll
        Cursor {
            kind: CursorKind::Move,
            aliases: vec![
                "cell".to_owned(),
                "fleur".to_owned(),
                "size_all".to_owned(),
                "all-scroll".to_owned(),
                "grabbing".to_owned(),
                "closedhand".to_owned(),
                "dnd-move".to_owned(),
                "dnd-none".to_owned(),
                "dnd-ask".to_owned(),
                "4498f0e0c1937ffe01fd06f973665830".to_owned(),
                "9081237383d90e509aa00f00170e968f".to_owned(),
                "fcf21c00b30f7e3f83fe0dfd12e71cff".to_owned(),
            ],

            path: PathBuf::from("Move.ani"),
        },
        // UpArrow
        Cursor {
            kind: CursorKind::Alternate,
            aliases: vec!["alias".to_owned(), "up_arrow".to_owned()],

            path: PathBuf::from("Alternate.ani"),
        },
        // Hand
        Cursor {
            kind: CursorKind::Link,
            aliases: vec![
                "pointer".to_owned(),
                "pointing_hand".to_owned(),
                "hand1".to_owned(),
                "hand2".to_owned(),
                "9d800788f1b08800ae810202380a0822".to_owned(),
                "e29285e634086352946a0e7090d73106".to_owned(),
            ],

            path: PathBuf::from("Link.ani"),
        },
        // Location
        Cursor {
            kind: CursorKind::Pin,
            aliases: vec![],
            path: PathBuf::from("Location.ani"),
        },
        // Person
        Cursor {
            kind: CursorKind::Person,
            aliases: vec![],
            path: PathBuf::from("Person.ani"),
        },
    ]
});

/// Mapping information for ANI files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cursor {
    kind: CursorKind,
    aliases: Vec<String>,
    path: PathBuf,
}

impl Cursor {
    /// Creates a new cursor mapping.
    #[must_use]
    pub const fn new(kind: CursorKind, aliases: Vec<String>, path: PathBuf) -> Self {
        Self {
            kind,
            aliases,
            path,
        }
    }

    /// Creates a new cursor mapping with the given `kind` from an existing one.
    #[must_use]
    pub fn with_kind(self, kind: CursorKind) -> Self {
        Self { kind, ..self }
    }

    /// Creates a new cursor mapping with the given `aliases` from an existing one.
    #[must_use]
    pub fn with_aliases(self, aliases: Vec<String>) -> Self {
        Self { aliases, ..self }
    }

    /// Creates a new cursor mapping with the given `path` from an existing one.
    #[must_use]
    pub fn with_path(self, path: PathBuf) -> Self {
        Self { path, ..self }
    }

    /// Cursor kind that this mapping defines.
    #[must_use]
    pub const fn kind(&self) -> CursorKind {
        self.kind
    }

    /// Alternative names for compatibility with different programs.
    #[must_use]
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    /// Path to the target ANI file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl From<TomlCursor> for Cursor {
    fn from(value: TomlCursor) -> Self {
        Self {
            kind: value.name,
            aliases: value.aliases,
            path: value.input,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
    strum::FromRepr,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum CursorKind {
    Default,
    Help,
    Wait,
    Progress,
    Crosshair,
    Text,
    Hand,
    Unavailable,
    NsResize,
    EwResize,
    NwseResize,
    NeswResize,
    Move,
    Alternate,
    Link,
    Pin,
    Person,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct TomlCursor {
    pub name: CursorKind,

    pub aliases: Vec<String>,

    #[serde(deserialize_with = "non_empty_path_buf")]
    pub input: PathBuf,
}

fn non_empty_path_buf<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let value = PathBuf::deserialize(deserializer)?;

    if value.is_empty() {
        Err(serde::de::Error::custom("path must not be empty"))
    } else {
        Ok(value)
    }
}

impl From<Cursor> for TomlCursor {
    fn from(value: Cursor) -> Self {
        Self {
            name: value.kind,
            aliases: value.aliases,
            input: value.path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_kind_parse_valid() {
        const EXPECTED: [(&str, CursorKind); 17] = [
            ("default", CursorKind::Default),
            ("help", CursorKind::Help),
            ("progress", CursorKind::Progress),
            ("wait", CursorKind::Wait),
            ("crosshair", CursorKind::Crosshair),
            ("text", CursorKind::Text),
            ("hand", CursorKind::Hand),
            ("unavailable", CursorKind::Unavailable),
            ("ns-resize", CursorKind::NsResize),
            ("ew-resize", CursorKind::EwResize),
            ("nwse-resize", CursorKind::NwseResize),
            ("nesw-resize", CursorKind::NeswResize),
            ("move", CursorKind::Move),
            ("alternate", CursorKind::Alternate),
            ("link", CursorKind::Link),
            ("pin", CursorKind::Pin),
            ("person", CursorKind::Person),
        ];

        for (value, kind) in EXPECTED {
            assert_eq!(value.parse::<CursorKind>().unwrap(), kind, "value: {value}");
        }
    }

    #[test]
    fn cursor_kind_parse_invalid() {
        let kind = "definitely_invalid".parse::<CursorKind>().unwrap_err();
        assert!(matches!(kind, strum::ParseError::VariantNotFound));
    }
}
