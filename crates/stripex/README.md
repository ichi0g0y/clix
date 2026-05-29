# stripex

[![CI](https://github.com/nantokaworks/clix/actions/workflows/ci.yml/badge.svg)](https://github.com/nantokaworks/clix/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/nantokaworks/clix?filter=stripex-*&label=stripex)](https://github.com/nantokaworks/clix/releases?q=stripex-)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Automatically select the right Stripe project based on the directory you're in.**

> 📖 Full docs: <https://nantokaworks.github.io/clix/tools/stripex.html>

> **Prerequisite:** `stripex` is a wrapper around [`stripe`](https://docs.stripe.com/stripe-cli). Install the Stripe CLI first and sign in with `stripe login` for each project you want to use.

If you work across personal, work, or client Stripe projects, `stripex` lets each repository carry the project context. It detects the Git remote owner for the current directory, maps that owner to a Stripe project name, injects `-p <name>`, and delegates to `stripe`.

## Installation

### Homebrew (macOS / Linux)

```bash
brew install nantokaworks/tap/stripex
```

### Cargo (all platforms)

```bash
cargo install --git https://github.com/nantokaworks/clix stripex
```

### Binary download

Pre-built binaries for macOS, Linux, and Windows are available on the [Releases](https://github.com/nantokaworks/clix/releases) page.

## Usage

Run `stripex` the same way you would run `stripe`:

```bash
stripex customers list
stripex listen
stripex whoami
```

Version commands print the `stripex` banner:

```bash
stripex version
stripex --version
```

Use `--dry-run` to inspect project resolution without running `stripe`:

```bash
stripex --dry-run customers list
```

## How Routing Works

1. If you pass `-p` / `--project-name`, `stripex` forwards the command unchanged.
2. If you pass `--api-key`, or set `STRIPE_API_KEY` / `STRIPE_SECRET_KEY`, `stripex` forwards the command unchanged.
3. Otherwise, `stripex` reads the current Git remote owner, looks up that owner in `~/.config/stripex/projects.yml`, and injects `-p <project>`.
4. If no Git owner mapping exists and a default project is configured with `stripex x use <project>`, that default is used when no Git remote owner can be detected.
5. If no project can be resolved, `stripex` delegates to `stripe` unchanged so the Stripe CLI uses its own default project.

## Project Management

`stripex` reads available project names from Stripe's own `~/.config/stripe/config.toml`, then stores only owner-to-project routing preferences:

```bash
stripex x list                         # show Stripe CLI projects and mappings
stripex x bind <project> <trigger>     # map a git remote owner to a project
stripex x unbind <trigger>             # delete a mapping
stripex x use <project>                # set the fallback project
stripex x whoami [<project>]           # show project authentication details
```

Example:

```bash
stripex x bind work acme-corp
stripex --dry-run customers list
```

`stripex login` passes through to `stripe login`. When it succeeds with a project flag such as `stripex login -p work`, `stripex` automatically binds the current Git remote owner to that project if no mapping already exists.

Mappings live in `~/.config/stripex/projects.yml`:

```yaml
default: personal
mappings:
  acme-corp: work
  personal-org: personal
```

## Security

`stripex` stores no API keys, tokens, or secrets. It only stores the non-secret owner-to-project-name mapping in `~/.config/stripex/projects.yml`.

Stripe's own `~/.config/stripe/config.toml` and keychain-backed credentials remain authoritative for authentication. Explicit credentials supplied through `--api-key`, `STRIPE_API_KEY`, or `STRIPE_SECRET_KEY` always pass through untouched.

## Requirements

- [`stripe`](https://docs.stripe.com/stripe-cli) installed and available on `PATH`
- Stripe CLI projects created with `stripe login`

## Build

```bash
cargo build --release
```

Or with the included task:

```bash
task build
```

## Release

GitHub Release is triggered by pushing a `stripex-v<version>` tag. `task release:stripex` fetches `origin/main`, reads the version from `crates/stripex/Cargo.toml` on that branch, creates a temporary worktree at `origin/main`, runs `cargo test -p stripex` there, then creates and pushes the tag.

```bash
task release:stripex
```

## Development

Run tests:

```bash
task test
```

Run in development mode:

```bash
task dev:stripex -- --dry-run customers list
```
