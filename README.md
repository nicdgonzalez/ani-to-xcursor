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

## Quickstart

From the directory containing the `Install.inf` file, run:

```bash
ani-to-xcursor init && ani-to-xcursor install
```

If you're *really* in a hurry, you can do:

```bash
ani-to-xcursor init > /dev/null && ani-to-xcursor install 2> /dev/null | xargs -I{} bash -c "{}"
```

The `install` command will try to figure out which command you need to set up
the cursor on your system and output it as the last line of stdout; if you pipe
it into `bash`, it should set the cursor automatically.

I did my best to include as many systems as I could, but I personally use
GNOME/gsettings, so that one is the only one that I am able to test. The
commands are found [here](./src/commands/install.rs) if you'd like to open a
pull request and add/correct the command for your system.

## Usage

First, `cd` to the directory containing the `Install.inf` file. Then, run the
`init` command to generate the `Cursor.toml` file:

```bash
ani-to-xcursor init
```

> [!NOTE]\
> If you can't get the command to work, the `Install.inf` is either missing or
> not formatted correctly. You will have to copy the template
> [`Cursor.toml`](./Cursor.toml) and fill it out manually.

Then, to generate the cursors:

```bash
ani-to-xcursor build
```

Finally, install the theme:

```bash
ani-to-xcursor install
```

For convenience, the `install` command calls also calls `build`. It is
separated into two steps in case you want to inspect the build output.

## How it works

A cursor package on Windows typically contains a file called `Install.inf`.
This is how Windows knows how to load the cursors. This project uses the
information inside of that file to generate the necessary files on Linux.

If you are missing the `Install.inf` file or the cursors are not at their
original paths anymore, I go over how to resolve any issues manually as I walk
through how to use this tool below.

The process has three steps:

1. Initialize the *package* (use `Install.inf` to create `Cursor.toml`)
1. Build the necessary files (use `Cursor.toml` to create `build`)
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

The `[cursors]` subdirectory is in square brackets because it depends on where
the cursors actually are. Best effort is made to find the cursors, since this
is the most tedious part of the process!
