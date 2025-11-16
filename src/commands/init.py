#!/usr/bin/env python3

"""
Helper script for generating a `Cursor.toml` file from an `Install.inf`.

# Usage

First, install my inf parser + tomli_w:

```bash
python3 -m pip install tomli_w git+https://github.com/nicdgonzalez/inf.git
```

Then, execute this script on the target directory:

```bash
python3 ./main.py --input /path/to/Install.inf
```

The generated TOML configuration will be output to stdout, which you can
capture or redirect into a file. Enjoy!

"""

import argparse
import functools
import logging
import pathlib
import sys
from typing import Any, Iterable, NamedTuple, Sequence

import inf
import tomli_w
from inf.utils import expand_vars

logger = logging.getLogger("ani-to-xcursor")


class Cursors(NamedTuple):
    default: str  # Arrow [0]
    help: str  # Help [1]
    progress: str  # AppStarting [2]
    wait: str  # Wait [3]
    crosshair: str  # Crosshair [4]
    text: str  # IBeam [5]
    hand: str  # NWPen [6]
    unavailable: str  # No [7]
    ns_resize: str  # SizeNS [8]
    ew_resize: str  # SizeWE [9]
    nwse_resize: str  # SizeNWSE [10]
    nesw_resize: str  # SizeNESW [11]
    move: str  # SizeAll [12]
    alternate: str  # UpArrow [13]
    link: str  # Hand [14]
    pin: str  # Location(?) [15]  I don't think Linux uses this one.
    person: str  # Person(?) [16]  I don't think Linux uses this one.


def setup_parser() -> argparse.ArgumentParser:
    """Configure the command-line argument parser."""
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-v",
        "--verbose",
        action="count",
        default=0,
        required=False,
        help="Use verbose output (or `-vv` for more verbose output)",
    )
    parser.add_argument(
        "-q",
        "--quiet",
        action="count",
        default=0,
        required=False,
        help="Use quiet output (or `-qq` for silent output)",
    )
    parser.add_argument(
        "-i",
        "--input",
        default=pathlib.Path.cwd(),
        type=pathlib.Path,
        required=True,
        help="Path to Install.inf file",
    )
    return parser


def main(argv: Sequence[str] = sys.argv[1:]) -> int:
    """The main entry point to the program.

    This function is intended to be wrapped by [`sys.exit`][sys.exit] so that
    its return value becomes the program's exit code.

    Returns
    -------
    int
        Zero indicates success; non-zero indicates failure.
    """
    parser = setup_parser()
    args = parser.parse_args()

    if (level := level_from_args(args.verbose, args.quiet)) is not None:
        handlers = [logging.StreamHandler(sys.stderr)]
        logging_subscriber(level, handlers)
    else:
        logging.disable()

    input: pathlib.Path = args.input.absolute()

    with open(input, "r") as f:
        buffer = f.read()
        document = inf.load(buffer)

    cursors = extract_cursors(document)
    config = create_config(theme_name=input.parent.name, cursors=cursors)

    config["cursor"] = [c for c in config["cursor"] if c["input"] != ""]

    text = tomli_w.dumps(config)
    sys.stdout.write(text)

    return 0


def create_config(theme_name: str, cursors: Cursors) -> dict[str, Any]:
    cwd = pathlib.Path.cwd()

    try:
        file_name = cursors.default.split("/")[-1]
        cursor_file = next(cwd.rglob(file_name))
    except StopIteration:
        # If we can't find the file, use the path provided by the Install file.
        cursor_dir = cursor_file.parent
    else:
        cursor_dir = cursor_file.parent

    as_input = functools.partial(as_input_impl, cursor_dir=cursor_dir, cwd=cwd)

    # TODO: Review cursors:
    # all-resize
    # all-scroll
    # cell
    # col-resize
    # context-menu
    # copy
    # grab
    # grabbing
    # pointer
    # row-resize
    # vertical-text
    # zoom-in
    # zoom-out

    # TODO: Review aliases:
    # fleur -> all-resize (crosshair?)
    # hand1 -> grab
    # hand2 -> pointer
    return {
        "theme": theme_name,
        "version": "0.1.0",
        "cursor": [
            {
                "name": "default",
                "input": as_input(cursors.default),
                "aliases": [
                    "arrow",
                    "dnd-move",
                    "left_ptr",
                    "move",
                    "top_left_arrow",
                    "X_cursor",
                ],
            },
            {
                "name": "help",
                "input": as_input(cursors.help),
                "aliases": ["question_arrow"],
            },
            {
                "name": "progress",
                "input": as_input(cursors.progress),
                "aliases": [],
            },
            {
                "name": "wait",
                "input": as_input(cursors.wait),
                "aliases": ["watch"],
            },
            {
                "name": "crosshair",
                "input": as_input(cursors.crosshair),
                "aliases": [
                    "cross",
                    "cross_reverse",
                    "diamond_cross",
                    "tcross",
                ],
            },
            {
                "name": "text",
                "input": as_input(cursors.text),
                "aliases": ["xterm"],
            },
            {
                "name": "hand",
                "input": as_input(cursors.hand),
                "aliases": [],
            },
            {
                "name": "unavailable",
                "input": as_input(cursors.unavailable),
                "aliases": ["not-allowed", "no-drop"],
            },
            {
                "name": "ns-resize",
                "input": as_input(cursors.ns_resize),
                "aliases": [
                    "bottom_side",
                    "sb_v_double_arrow",
                    "top_side",
                    "n-resize",
                    "s-resize",
                ],
            },
            {
                "name": "ew-resize",
                "input": as_input(cursors.ew_resize),
                "aliases": [
                    "left_side",
                    "right_side",
                    "sb_h_double_arrow",
                    "w-resize",
                    "e-resize",
                ],
            },
            {
                "name": "nwse-resize",
                "input": as_input(cursors.nwse_resize),
                "aliases": [
                    "bd_double_arrow",
                    "bottom_right_corner",
                    "top_left_corner",
                    "se-resize",
                    "nw-resize",
                ],
            },
            {
                "name": "nesw-resize",
                "input": as_input(cursors.nesw_resize),
                "aliases": [
                    "bottom_left_corner",
                    "fd_double_arrow",
                    "top_right_corner",
                    "sw-resize",
                    "ne-resize",
                ],
            },
            {
                "name": "move",
                "input": as_input(cursors.move),
                "aliases": [],
            },
            {
                "name": "alternate",
                "input": as_input(cursors.alternate),
                "aliases": ["alias"],
            },
            {
                "name": "link",
                "input": as_input(cursors.link),
                "aliases": [],
            },
            {
                "name": "pin",
                "input": as_input(cursors.pin),
                "aliases": [],
            },
            {
                "name": "person",
                "input": as_input(cursors.person),
                "aliases": [],
            },
        ],
    }


def as_input_impl(
    cursor: str,
    *,
    cursor_dir: pathlib.Path,
    cwd: pathlib.Path,
) -> str:
    file = pathlib.Path(cursor)
    file = cursor_dir.joinpath(file.name)
    file_str = file.as_posix().replace(cwd.as_posix(), "")

    if file_str.startswith("/"):
        return "." + file_str
    else:
        return file_str


def extract_cursors(document: dict[str, Any]) -> Cursors:
    # NOTE: Entry [3] is empty; entry [4] is a single string with commas.
    #
    #   [Scheme.Reg]
    #   HKCU,"Control Panel\Cursors\Schemes","%SCHEME_NAME%",,"cur1,cur2,..."
    #
    entries = document["Scheme.Reg"][""]
    _, _, _theme_name, _, c = entries[0]
    cursors: list[str] = c.split(",")

    strings = document["Strings"]

    for i, cursor in enumerate(cursors.copy()):
        # 10 is a Windows Directory ID meaning %SystemRoot% (e.g., `C:\Windows`).  # noqa: E501
        cursor = cursor.replace("%10%\\", "")
        cursor = expand_vars(cursor, strings=strings)
        cursor = cursor.replace("\\", "/")
        cursors[i] = cursor  # Update original list with the cleaned version.

    return Cursors(*cursors)


def level_from_args(verbose: int, quiet: int) -> int | None:
    match quiet:
        case n if n < 0:
            raise ValueError(f"value cannot be negative, got {n}")
        case 0:
            pass
        case 1:
            return logging.CRITICAL
        case _:
            return None

    match verbose:
        case n if n < 0:
            raise ValueError(f"value cannot be negative, got {n}")
        case 0:
            return logging.ERROR
        case 1:
            return logging.WARNING
        case 2:
            return logging.INFO
        case _:
            return logging.DEBUG


def logging_subscriber(
    level: int,
    handlers: Iterable[logging.Handler],
) -> None:
    global logger

    formatter = logging.Formatter(
        fmt="%(asctime)s %(levelname)s %(name)s: %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%S%z",  # ISO 8601 format
        style="%",
    )
    root = logger

    for handler in handlers:
        handler.setFormatter(fmt=formatter)
        root.addHandler(hdlr=handler)

    root.setLevel(level=level)
    logging.getLogger("inf").setLevel(level=level)


if __name__ == "__main__":
    sys.exit(main())
