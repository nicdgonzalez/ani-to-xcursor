# ANI to Xcursor

A command-line tool for converting Windows animated cursors to Linux.

## Installation

**Dependencies**:

- cargo 1.87.0
- xcursorgen 1.0.8

Install from Git using cargo:

```bash
cargo install --git https://github.com/nicdgonzalez/ani-to-xcursor
```

## How it works

A cursor package on Windows typically contains a file called `Install.inf`.
This is how Windows knows how to load the cursors. This project uses the
information inside of that file to generate the necessary files on Linux.

If you are missing the `Install.inf` file or the cursors are not at their
original paths anymore, I go over how to resolve any issues manually as I walk
through how to use this tool below.

The process has three steps:

1. Initialize the *package* (`Install.inf` -> `Cursor.toml`)
1. Build the necessary files (`Cursor.toml` -> `build`)
1. Install the theme (`build/theme` -> `$HOME/.local/share/icons/Theme-Name`)

By the end, the directory will look something like this:

```
Theme-Name
├── build
│   ├── frames
│   └── theme
│       ├── cursors
│       └── index.theme
├── [cursors]
├── Install.inf
└── Cursor.toml
```

Note the "Theme-Name" at the top; it represents the name you will use to
activate the cursor theme at the end. It is one level above the `Install.inf`
file. Before getting started, make sure it doesn't conflict with other theme
names.

The `[cursors]` subdirectory is in square brackets because it depends on what
is defined inside of `Install.inf`.

## Getting started

First, `cd` to the directory containing the `Install.inf` file. Then, run the
`init` command to generate the `Cursor.toml` file:

```bash
ani-to-xcursor init
```

> [!NOTE]\
> If you can't get the command to work, the `Install.inf` is either missing or
> not formatted correctly. You will have to copy this template
> [`Cursor.toml`](./Cursor.toml) and fill it out manually.

Then, to generate the cursors:

```bash
ani-to-xcursor build
```

Finally, install the theme:

```bash
ani-to-xcursor install
```

You're done!
