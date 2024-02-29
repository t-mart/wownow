# wownow

A stupid CLI tool to get the current World of Warcraft versions.

Stupid, because it just scrapes the current version from the the homepage of
[Wago Tools](https://wago.tools/). There are probably more authoritative ways to
do this, but this is too easy to pass up.

This tool is written to support WoW addon development in CI pipelines, where new
addons may be released for the latest version(s) of the game.

## Installation

```bash
cargo install wownow
```

## Usage

```bash
wownow
```

```bash
wownow --help
```
