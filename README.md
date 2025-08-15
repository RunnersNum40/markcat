# markcat

Convert a project directory to markdown.
Respects `.gitignore` by default.
Filter by extension, exact filename, or files without extensions.
Output to stdout or a file.

## Install

Run `cargo build --release` or `cargo install --path .`.

## Usage

`markcat [OPTIONS] [DIR]`
`DIR` may be given positionally or via `-p/--path`. Default is `.`

### Options

- `-p, --path <DIR>` — Directory to convert. Positional `<DIR>` also supported
- `-i, --ignore-gitignore` — Do not apply `.gitignore` or standard ignore filters
- `-t, --trim` — Trim leading and trailing whitespace in file contents
- `-w, --whitelist <ITEMS>` — Comma-separated allow-list
- `-b, --blacklist <ITEMS>` — Comma-separated deny-list
- `-o, --output <FILE>` — Write output to `<FILE>` instead of stdout (creates or truncates)

## Filtering syntax

`ITEMS` accepts:
- Extensions (case-insensitive), with or without a leading dot. Examples: `rs`, `.md`, `txt`
- Exact filenames (case-sensitive). Examples: `LICENSE`, `Makefile`, `Dockerfile`
- The token `noext` to match files without an extension

Whitelist passes if any item matches. Blacklist blocks if any item matches. If a whitelist is provided, non-matching files are skipped even if not blacklisted.

## Output format

For each file:
1) Print the filepath in backticks, like `path/to/file`
2) Then print a fenced code block in the markdown output. The fence language is the file’s extension if present; otherwise no language

Example description: `src/main.rs` emits a path line followed by a Rust code fence; `LICENSE` emits a path line followed by a plain fence.

## Examples

- Default current directory: `markcat`
- Specific directory: `markcat src/` or `markcat -p src/`
- Ignore `.gitignore`: `markcat -i`
- Only Rust, Markdown, plus exact `LICENSE`: `markcat -w rs,md,LICENSE`
- Exclude logs and all extensionless files: `markcat -b log,noext`
- Write to a file: `markcat -o out.md src/`
- Trim contents: `markcat -t`
