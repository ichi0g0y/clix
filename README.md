# clix

[![CI](https://github.com/nantokaworks/clix/actions/workflows/ci.yml/badge.svg)](https://github.com/nantokaworks/clix/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**The clix toolkit — directory-aware account switchers for popular dev CLIs.**

> 📖 **Docs: <https://nantokaworks.github.io/clix/>** — installation, the shared
> routing model, and a page per tool.

`clix` is a Cargo workspace that hosts a small family of CLI wrappers. Each
member detects which account to use from your current directory (git remote,
project config) and forwards the command to the upstream CLI with the right
credentials injected — into only the child process it runs, then it gets out of
the way. No more manual `auth switch` between personal and work accounts.

![ghx demo](.github/demo.gif)

## Member tools

| Tool | Wraps | Switches by | Status |
|---|---|---|---|
| [ghx](crates/ghx/) | [`gh`](https://cli.github.com/) | git remote owner | shipped |
| [flyx](crates/flyx/) | [`fly`](https://fly.io/docs/flyctl/) | `fly.toml` `app` / git remote owner | shipped |
| [wranglerx](crates/wranglerx/) | [`wrangler`](https://developers.cloudflare.com/workers/wrangler/) | `wrangler.toml` `account_id` / git remote owner | shipped |
| [stripex](crates/stripex/) | [`stripe`](https://docs.stripe.com/stripe-cli) | git remote owner → project | shipped |

Each tool is released independently. Tags follow the `<tool>-v<semver>`
convention (e.g. `ghx-v0.4.0`).

## Quick example

`ghx` reads the current repository's GitHub owner and runs `gh` with the
matching token:

```bash
cd ~/src/work-api
ghx pr status
```

Everything else — install, the shared resolution ladder, per-tool commands,
troubleshooting — lives in the **[docs site](https://nantokaworks.github.io/clix/)**
and each tool's README.

## Workspace layout

```
crates/
├── core/        # clix-core: shared git remote parser, exec, banner, update check
├── ghx/         # gh wrapper
├── flyx/        # fly wrapper
├── wranglerx/   # wrangler wrapper
└── stripex/     # stripe wrapper
```

## Install

See [Installation](https://nantokaworks.github.io/clix/concepts/installation.html),
or each member tool's README:
[ghx](crates/ghx/README.md) ·
[flyx](crates/flyx/README.md) ·
[wranglerx](crates/wranglerx/README.md) ·
[stripex](crates/stripex/README.md).

## Build

```bash
cargo build --release --workspace
```

## Release

Releases are built and published locally (no CI) with `task release:<tool>`.
See [RELEASING.md](RELEASING.md) for prerequisites and the full workflow.

## License

MIT
