# wownow

A ~~stupid~~ CLI tool to get the current versions of World of Warcraft.

This tool is written to support WoW addon development in CI pipelines. For
example, you could use it to check that your addon is declares the current
version of the game, and not an outdated one.

wownow issues [TACT](https://wowdev.wiki/TACT) requests to Blizzard's version
servers, parses the response, and outputs a JSON object stating the current
version and build for each region of each product. See an output example
[below](#usage).

wownow used to be "stupid" because it just scraped from the the homepage of
[Wago Tools](https://wago.tools/), even though there was a JSON HTTP API
endpoint (see [#1](https://github.com/t-mart/wownow/issues/1)). Now, it's less
stupid.

## Installation

Generally, build the project from source:

```bash
cargo install wownow
```

For Linux platforms, you can also download a pre-built binary from the
[releases](https://github.com/t-mart/wownow/releases) page. Or, you can use the
[`binstall`](https://github.com/cargo-bins/cargo-binstall) cargo tool to install it:

```bash
cargo binstall wownow
```

## Usage

```console
$ wownow
{
  "retrieval_datetime": "2024-03-14T17:56:25.593962700Z",
  "products": [
    {
      "name": "wow",
      "versions": [
        {
          "region": "us",
          "version": "10.2.5",
          "build": "53584"
        },
        {
          "region": "eu",
          "version": "10.2.5",
          "build": "53584"
        },
        ...
      ]
    },
    {
      "name": "wow_classic",
      "versions": [
        {
          "region": "us",
          "version": "3.4.3",
          "build": "53622"
        },
        {
          "region": "eu",
          "version": "3.4.3",
          "build": "53622"
        },
        ...
      ]
    },
    {
      "name": "wow_classic_era",
      "versions": [
        {
          "region": "us",
          "version": "1.15.1",
          "build": "53623"
        },
        {
          "region": "eu",
          "version": "1.15.1",
          "build": "53623"
        },
        ...
      ]
    }
  ]
}
```

### Leverage with `jq`

You can use the [`jq`](https://jqlang.github.io/jq/) tool to filter the output
to only the information you need.

For example, to get the current version of retail (`wow` product) World of Warcraft in the
`us` region:

```console
$ wownow | jq -r --arg product "wow" --arg region "us" '.products[] | select(.name == $product).versions[] | select(.region == $region).version'
10.2.5
```
