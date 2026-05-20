# ani2xcur

> [!NOTE]\
> This project recieved a full rewrite and was renamed from `ani-to-xcursor` to
> `ani2xcur` on March 10, 2026.

A command-line tool for installing Windows animated cursor schemes on Unix-like
operating systems running the [X Window System].

## Overview

Windows animated cursors use the [ANI file format], a container format that
stores multiple animation frames along with metadata such as frame timing and
frame order.

Related cursors are grouped into *cursor schemes*. A cursor scheme is typically
distributed together with an `Install.inf` file, which contains the metadata
required to install the scheme.

`ani2xcur` uses this information to convert each animated cursor into [Xcursor]
format and installs the resulting files in the locations expected by the X
Window System.

While many larger projects now parse INF files to perform bulk cursor
conversions, this project was the first to introduce that approach.

Originally created to fill the gap, I'm now focusing on providing an ergonomic
solution: a single command-line interface with intentionally designed options
and subcommands, optimized for speed and correctness. See
[Benchmarks](#Benchmarks) for comparisons against similar projects.

## Installation

| Requirement | Version | Description                                  |
| :---------- | :------ | :------------------------------------------- |
| cargo       | 1.94.0  | Build and install the command-line interface |

Install from GitHub using `cargo`:

```bash
cargo install --git https://github.com/nicdgonzalez/ani2xcur
```

Or, download a pre-built binary from the [Releases] page on GitHub.

## Quickstart

> [!TIP]\
> Need a cursor to start with? Try NOiiRE's [Hornet Cursor] from Hollow Knight:
> Silksong.

From the directory containing the `Install.inf` file, run:

```bash
ani2xcur install --default-init
```

## Usage

From the directory containing the `Install.inf` file, run:

> [!TIP]\
> If your INF file has a different name, use the `--inf` flag instead of
> renaming the existing file.
>
> ```bash
> ani2xcur init --inf Other.inf
> ```

```bash
ani2xcur init
```

This command parses the INF file and extracts the information needed to decode
each `.ani` file. The results are written to an intermediate `Cursor.toml`
file.

Next, build the cursor theme:

```bash
ani2xcur build
```

This command parses each `.ani` file and generates animated cursors in
**Xcursor** format. The cursors are placed in a theme directory using the
standard X cursor naming conventions.

Then, install the theme:

```bash
ani2xcur install
```

This creates the necessary links so X can locate and use the newly created
cursor theme.

Finally, enable the theme using your system's cursor settings. The exact
process varies by distribution, but most desktop environments provide a
command-line tool or a graphical settings panel.

Enjoy your new cursors!

### Convert individual ANI files

> [!TIP]\
> Do NOT use this command if you are converting a cursor theme that does not
> have an INF file. Instead, use the `--skip-inf` flag on the `init` command to
> create a generic manifest that can be manually edited. This way, you can
> still use the `build` and `install` commands which do a lot of the heavy
> lifting.
>
> ```bash
> ani2xcur init --skip-inf
> ```

If you only want to convert a single ANI file:

```bash
ani2xcur convert Default.ani
```

## Benchmarks

Benchmarked using [hyperfine] against similar projects on GitHub solving the
same problem.

| Project       | Version |
| :------------ | :------ |
| [ani2xcur]    | 0.1.3   |
| [ani2xcursor] | 1.5.0   |
| [win2xcur]    | 0.2.0   |

```bash
hyperfine \
    --warmup 15 \
    --setup 'ani2xcur uninstall || rm -r ./theme ./Cursor.toml || true' \
    'ani2xcur init && ani2xcur build' \
    --conclude 'rm -r ./theme ./Cursor.toml' \
    'ani2xcursor --size 32,48,64,96 --out ./theme .' \
    --conclude 'rm -r ./theme' \
    --prepare 'mkdir --parents ./theme' \
    'win2xcur ./*.ani --output-dir ./theme' \
    --conclude 'rm -r ./theme'
```

```console
Benchmark 1: ani2xcur init && ani2xcur build
  Time (mean ± σ):      67.8 ms ±   3.1 ms    [User: 337.6 ms, System: 176.5 ms]
  Range (min … max):    60.8 ms …  73.3 ms    32 runs

Benchmark 2: ani2xcursor --size 32,48,64,96 --out ./theme .
  Time (mean ± σ):     279.0 ms ±   8.8 ms    [User: 201.9 ms, System: 64.4 ms]
  Range (min … max):   268.3 ms … 298.7 ms    10 runs

Benchmark 3: win2xcur ./*.ani --output-dir ./theme
  Time (mean ± σ):     616.8 ms ±   9.7 ms    [User: 1789.6 ms, System: 304.6 ms]
  Range (min … max):   603.5 ms … 641.0 ms    10 runs

Summary
  ani2xcur init && ani2xcur build ran
    4.12 ± 0.23 times faster than ani2xcursor --size 32,48,64,96 --out ./theme .
    9.10 ± 0.44 times faster than win2xcur ./*.ani --output-dir ./theme
```

## Roadmap

- [x] Automatically scale cursors to standard sizes.
- [x] Remove `xcursorgen` dependency.
- [x] Remove need for `build` directory for the `convert` subcommand.
- [ ] Interactive mode to convert individual cursors with Linux remappings.
- [ ] Graphical User Interface

[ani file format]: https://en.wikipedia.org/wiki/ANI_(file_format)
[ani2xcur]: https://github.com/nicdgonzalez/ani2xcur
[ani2xcursor]: https://github.com/yuzujr/ani2xcursor
[hornet cursor]: https://ko-fi.com/s/2e08ca3a58
[hyperfine]: https://github.com/sharkdp/hyperfine
[releases]: https://github.com/nicdgonzalez/ani-to-xcursor/releases
[win2xcur]: https://github.com/quantum5/win2xcur
[x window system]: https://en.wikipedia.org/wiki/X_Window_System
[xcursor]: https://www.x.org/releases/current/doc/man/man3/Xcursor.3.xhtml
