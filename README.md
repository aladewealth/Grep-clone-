# mgrep

A small `grep` clone written in Rust, using only the standard library — no external crates, no `Cargo.toml` required.

## Features

- Plain substring search across files, directories, or stdin
- Case-insensitive matching
- Line numbers
- Recursive directory search
- Invert match (show non-matching lines)
- Count-only mode
- Files-with-matches mode
- Combinable short flags (e.g. `-rin`)

## Build

```bash
rustc mgrep.rs -o mgrep
```

This produces a single binary, `mgrep`, in the current directory.

## Usage

```
mgrep [OPTIONS] PATTERN [FILE...]
```

Search for `PATTERN` in each `FILE`. If no file is given, `mgrep` reads from stdin.

### Options

| Flag | Description                                  |
|------|-----------------------------------------------|
| `-i` | Ignore case distinctions                      |
| `-n` | Print line numbers with output lines          |
| `-r` | Recursively search directories                |
| `-v` | Invert match (print non-matching lines)       |
| `-c` | Print only a count of matching lines per file |
| `-l` | Print only names of files with a match        |
| `-h` | Show the help message                         |

## Examples

Search a single file:

```bash
./mgrep "TODO" notes.txt
```

Case-insensitive search with line numbers:

```bash
./mgrep -in "error" server.log
```

Recursively search a directory:

```bash
./mgrep -r "fn main" src/
```

Pipe input from another command:

```bash
cat access.log | ./mgrep -i "500"
```

Count matches per file:

```bash
./mgrep -rc "TODO" src/
```

List only files containing a match:

```bash
./mgrep -rl "unsafe" src/
```

Combine flags:

```bash
./mgrep -rin "warning" logs/
```

## Exit codes

- `0` — at least one match was found
- `1` — no matches were found
- `2` — an error occurred (bad arguments, missing file, etc.)

## Limitations

`mgrep` matches literal substrings only — it does not support regular expressions (`.`, `*`, `[abc]`, capture groups, etc.). Adding real regex support would require the [`regex`](https://crates.io/crates/regex) crate and a proper Cargo project instead of a single `rustc`-compiled file.

## License

Use it, modify it, break it — no restrictions.
