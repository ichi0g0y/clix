# clix

**Directory-aware account switchers for popular dev CLIs.**

`clix` is a family of small command-line wrappers. Each one detects which
account to use from your current directory — the git remote, a project config
file — and forwards your command to the upstream CLI with the right credentials
injected. No more manual `auth switch` when you `cd` between personal, work, and
client projects.

The trick is that clix tools never change a global "active account". They
inspect the current directory, resolve the matching account, inject credentials
into **only the child process** they are about to run, and then get out of the
way.

## Member tools

| Tool | Wraps | Switches by | Status |
|---|---|---|---|
| [ghx](tools/ghx.md) | [`gh`](https://cli.github.com/) | git remote owner | shipped |
| [flyx](tools/flyx.md) | [`fly`](https://fly.io/docs/flyctl/) | `fly.toml` `app` / git remote owner | shipped |
| [wranglerx](tools/wranglerx.md) | [`wrangler`](https://developers.cloudflare.com/workers/wrangler/) | `wrangler.toml` `account_id` / git remote owner | shipped |
| [stripex](tools/stripex.md) | [`stripe`](https://docs.stripe.com/stripe-cli) | git remote owner → project | shipped |

Each tool is released independently. Tags follow the `<tool>-v<semver>`
convention (e.g. `ghx-v0.4.0`).

## Quick examples

`ghx` reads the current repository's GitHub owner and runs `gh` with the
matching token:

```bash
cd ~/src/work-api
ghx pr status
```

`wranglerx` reads the current project's Cloudflare `account_id` and runs
`wrangler` with the matching API token:

```bash
cd ~/src/worker
wranglerx --dry-run deploy
wranglerx deploy
```

## Where to go next

- **[Installation](concepts/installation.md)** — install any of the tools.
- **[How routing works](concepts/routing.md)** — the shared resolution model
  every tool follows. Read this once and the per-tool pages get much shorter.
- **[Profiles and mappings](concepts/profiles.md)** — how each tool stores the
  account context it switches between.
- The **Tools** section has a page per CLI with its specific commands.
