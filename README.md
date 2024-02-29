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

```console
$ wownow
[
  {
    "product": "wow",
    "version": "10.2.5",
    "build": "53441",
    "created_at": "2024-02-23T21:29:01Z"
  },
  {
    "product": "wow_beta",
    "version": "10.0.2",
    "build": "47120",
    "created_at": "2022-12-17T09:46:02Z"
  },
  ...
]
```
