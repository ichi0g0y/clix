# Installation

Every clix tool is a standalone binary. Install only the ones you need; they
share no runtime state beyond their own config under `~/.config/<tool>/`.

Each tool **wraps an upstream CLI**, so install that first (see each tool page
for the prerequisite). clix shells out to the upstream binary for every real
operation.

## Homebrew (macOS / Linux)

```bash
brew install nantokaworks/tap/ghx
brew install nantokaworks/tap/flyx
brew install nantokaworks/tap/wranglerx
brew install nantokaworks/tap/stripex
```

## Cargo (all platforms)

```bash
cargo install --git https://github.com/nantokaworks/clix ghx
cargo install --git https://github.com/nantokaworks/clix flyx
cargo install --git https://github.com/nantokaworks/clix wranglerx
cargo install --git https://github.com/nantokaworks/clix stripex
```

## Binary download

Pre-built binaries for macOS, Linux, and Windows are attached to each release on
the [Releases](https://github.com/nantokaworks/clix/releases) page. Tags are
prefixed per tool (`ghx-v*`, `flyx-v*`, `wranglerx-v*`, `stripex-v*`).

## Shell script (ghx only)

The one-line installer currently provisions **`ghx`** specifically:

```bash
curl -fsSL https://raw.githubusercontent.com/nantokaworks/clix/main/install.sh | sh
```

It downloads the latest `ghx` release for your OS/arch into `/usr/local/bin`
(or `~/.local/bin` if that is not writable). For the other tools, use Homebrew,
Cargo, or a binary download above.

## Verifying

Each tool prints a banner with its version when run with no arguments or
`--version`:

```bash
ghx --version
flyx --version
wranglerx --version
stripex --version
```
