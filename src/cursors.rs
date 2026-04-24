pub const DEFAULT_FILE_NAMES: [&str; 17] = [
    "Default.ani",
    "Help.ani",
    "Busy.ani",
    "Working.ani",
    "Crosshair.ani",
    "Text.ani",
    "Hand.ani",
    "Unavailable.ani",
    "Vertical.ani",
    "Horizontal.ani",
    "Diagonal1.ani",
    "Diagonal2.ani",
    "Move.ani",
    "Alternate.ani",
    "Link.ani",
    "Location.ani",
    "Person.ani",
];

pub struct CursorInfo {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
}

/// Cursors are ordered the same as they appear in the Windows Registry.
///
/// If your theme builds successfully and a cursor is not showing up as expected, it is likely
/// because the names here don't match what your system is looking for when displaying the cursor.
/// To fix it, you need to find the target Linux cursor name, then add it here as an alias.
///
/// For additional coverage, I also included the names used in these repositories:
///
/// - [JeffHathford/cursor_win2lin]
/// - [quantum5/win2xcur]
///
/// [JeffHathford/cursor_win2lin]: https://github.com/JeffHathford/cursor_win2lin/blob/7afd265c16463c5dfe28abff0d86a81ea5275b37/mappings.txt
/// [quantum5/win2xcur]: https://github.com/quantum5/win2xcur/blob/feadbe284f502387b6d00fdd688138f6b0faa202/win2xcur/theme.py
pub const CURSORS: [CursorInfo; 17] = [
    // Arrow
    CursorInfo {
        name: "default",
        aliases: &["arrow", "left_ptr", "top_left_arrow", "X_cursor", "mouse"],
    },
    // Help
    CursorInfo {
        name: "help",
        aliases: &[
            "question_arrow",
            "whats_this",
            "left_ptr_help",
            "5c6cd98b3f3ebcb1f9c7f1c204630408",
            "d9ce0ab605698f320427677b458ad60b",
        ],
    },
    // AppStarting
    CursorInfo {
        name: "progress",
        aliases: &[
            "half-busy",
            "left_ptr_watch",
            "3ecb610c1bf2410f44200f48c40d3599",
            "08e8e1c95fe2fc01f976f1e063a24ccd",
            "00000000000000020006000e7e9ffc3f",
        ],
    },
    // Wait
    CursorInfo {
        name: "wait",
        aliases: &["watch"],
    },
    // Crosshair
    CursorInfo {
        name: "crosshair",
        aliases: &["cross", "cross_reverse", "diamond_cross", "tcross", "plus"],
    },
    // IBeam
    CursorInfo {
        name: "text",
        aliases: &["xterm", "ibeam"],
    },
    // NWPen
    CursorInfo {
        name: "hand",
        aliases: &["pencil", "draft"],
    },
    // No
    CursorInfo {
        name: "unavailable",
        aliases: &[
            "not-allowed",
            "no-drop",
            "dnd-no-drop",
            "circle",
            "crossed_circle",
            "forbidden",
            "03b6e0fcb3499374a867c041f52298f0",
        ],
    },
    // SizeNS
    CursorInfo {
        name: "ns-resize",
        aliases: &[
            "top_side",
            "bottom_side",
            "n-resize",
            "s-resize",
            "row-resize",
            "size_ver",
            "size-ver",
            "split_v",
            "double_arrow",
            "v_double_arrow",
            "sb_v_double_arrow",
            "00008160000006810000408080010102",
            "2870a09082c103050810ffdffffe0204",
        ],
    },
    // SizeWE
    CursorInfo {
        name: "ew-resize",
        aliases: &[
            "left_side",
            "right_side",
            "sb_h_double_arrow",
            "w-resize",
            "e-resize",
            "size_hor",
            "h_double_arrow",
            "size-hor",
            "col-resize",
            "split_h",
            "14fef782d02440884392942c11205230",
            "028006030e0e7ebffc7f7070c0600140",
        ],
    },
    // SizeNWSE
    CursorInfo {
        name: "nwse-resize",
        aliases: &[
            "bd_double_arrow",
            "bottom_right_corner",
            "top_left_corner",
            "se-resize",
            "nw-resize",
            "size_fdiag",
        ],
    },
    // SizeNESW
    CursorInfo {
        name: "nesw-resize",
        aliases: &[
            "bottom_left_corner",
            "fd_double_arrow",
            "top_right_corner",
            "sw-resize",
            "ne-resize",
            "size_bdiag",
            "size-bdiag",
            "fcf1c3c7cd4491d801f1e1c78f100000",
        ],
    },
    // SizeAll
    CursorInfo {
        name: "move",
        aliases: &[
            "cell",
            "fleur",
            "size_all",
            "all-scroll",
            "grabbing",
            "closedhand",
            "dnd-move",
            "dnd-none",
            "dnd-ask",
            "4498f0e0c1937ffe01fd06f973665830",
            "9081237383d90e509aa00f00170e968f",
            "fcf21c00b30f7e3f83fe0dfd12e71cff",
        ],
    },
    // UpArrow
    CursorInfo {
        name: "alternate",
        aliases: &["alias", "up_arrow"],
    },
    // Hand
    CursorInfo {
        name: "link",
        aliases: &[
            "pointer",
            "pointing_hand",
            "hand1",
            "hand2",
            "9d800788f1b08800ae810202380a0822",
            "e29285e634086352946a0e7090d73106",
        ],
    },
    // Location
    CursorInfo {
        name: "pin",
        aliases: &[],
    },
    // Person
    CursorInfo {
        name: "person",
        aliases: &[],
    },
];
