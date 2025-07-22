# markcat

`markcat` is a CLI tool that converts the contents of a project directory into a markdown-formatted representation, respecting `.gitignore` files by default. It allows you to specify whitelisted and blacklisted file types for output.

## Features

- Converts directories and files into a structured markdown output.
- Respects `.gitignore` files by default (can be disabled with a flag).
- Whitelist or blacklist specific file extensions.
- Simple and easy-to-use CLI interface.

## Usage

`markcat [OPTIONS]`

### Options

- `-p, --path <DIR>`: Specify the directory to convert (defaults to the current directory).
- `-i, --ignore-gitignore`: Ignore `.gitignore` rules during conversion.
- `-t, --trim`: Trim leading and trailing whitespace from file contents.
- `-w, --whitelist <EXTENSIONS>`: Comma-separated list of whitelisted file extensions (e.g., `rs,md,txt`).
- `-b, --blacklist <EXTENSIONS>`: Comma-separated list of blacklisted file extensions (e.g., `log,tmp`).

## Examples

- Convert the current directory into a markdown tree:

  `markcat`

- Convert a specific directory into a markdown tree:

  `markcat -p /path/to/directory`

- Convert a directory into a markdown tree, ignoring `.gitignore` rules:

  `markcat -i`

- Convert a directory into a markdown tree, whitelisting specific file extensions:

  `markcat -w rs,md,txt`
