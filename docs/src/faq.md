# FAQ

## Do I have to replace `gh` / `fly` / `wrangler` / `stripe`?

No. clix tools are thin wrappers — use `ghx` where you would use `gh`, and so
on. Anything the upstream CLI can do, the wrapper forwards. Bootstrap commands
(`help`, `auth …`, `login`) pass straight through.

## What are the requirements?

Each tool needs its upstream CLI installed and on `PATH`, plus at least one
authenticated account:

| Tool | Needs | Auth |
|---|---|---|
| ghx | [`gh`](https://cli.github.com/) | `gh auth login` |
| flyx | [`fly`](https://fly.io/docs/flyctl/install/) | `flyx auth login` (or `flyx x save-token`) |
| wranglerx | [`wrangler`](https://developers.cloudflare.com/workers/wrangler/) | `wrangler login` + `wranglerx x save <profile>` |
| stripex | [`stripe`](https://docs.stripe.com/stripe-cli) | `stripe login` |

## Does clix store my secrets?

It depends on how the upstream CLI manages credentials:

- **ghx** and **stripex** store **no tokens/secrets** — they reuse the upstream
  CLI's own credential storage and keep only non-secret routing maps.
- **flyx** and **wranglerx** snapshot access/OAuth tokens into
  `~/.config/<tool>/profiles.yml` so they can switch between them; wranglerx also
  refreshes expired OAuth tokens automatically.

See [Profiles and mappings](concepts/profiles.md) for details.

## Is clix safe to use in CI?

Yes. For flyx, wranglerx, and stripex: if the upstream credential env var is
already set, the wrapper passes the command through untouched and never reads
the profile store. **ghx is the exception** — it always resolves the account
from `gh`'s `hosts.yml` (and overrides `GH_TOKEN`), so in CI authenticate `gh`
rather than relying on a preset `GH_TOKEN`. See the CI table in
[Troubleshooting](troubleshooting.md).

## What is the version banner / update notice?

When a tool prints its banner (no arguments, or `--version`), it checks the
GitHub API for a newer release at most once every 24 hours and shows an upgrade
hint matched to your install method:

```
│ update available: 0.3.0 → 0.3.1
│ brew upgrade ghx
```

Disable it per tool with `<TOOL>_NO_UPDATE_CHECK=1` (e.g.
`export GHX_NO_UPDATE_CHECK=1`).

## How are the tools versioned and released?

Each tool is released independently; tags follow `<tool>-v<semver>` (e.g.
`ghx-v0.4.0`). Sharing a version number across tools is coincidental — they
advance on their own cadence.
