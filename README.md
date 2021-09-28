# mdbabel

A very simple tool to execute code blocks inside of markdown documents. Inspired
by [Org Babel](https://orgmode.org/worg/org-contrib/babel/).

## Installation

First clone the repo, then use cargo to install.

<!-- mdbabel :name cargo-install -->
```shell
cargo install --path .
```

## Usage

```shell
mdbabel <path to markdown file>
```

`mdbabel` executes code blocks that have the following properites:
- Has a special comment immediately preceding the code block in the form of
  `<!-- mdbabel :name <some-name> -->`.
- The code block has an associated language with it. Supported languages are
  `sh`, `bash`, and `shell`. `shell` just calls out to `sh` for now.

View the raw version of this readme to see an example usage.

## Testing

Run tests with cargo.

<!-- mdbabel :name cargo-test -->
```shell
cargo test
```

## Bugs

Probably.
