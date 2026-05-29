# Profiles and mappings

clix tools switch between accounts you have already authenticated with the
upstream CLI. To do that, each tool keeps a small store under
`~/.config/<tool>/` describing the accounts it knows about and how to route to
them. The shape differs by tool, but the ideas are the same:

- A **profile / account** is one authenticated identity.
- A **mapping** routes a *trigger* (a git owner, an app, an account id) to a
  profile.
- A **default** is the fallback profile when nothing else matches.

You manage all of this through each tool's `x` subcommand namespace.

## The `x` subcommands

Each tool exposes management commands under `<tool> x …`. They follow a common
vocabulary:

| Intent | ghx | flyx | wranglerx | stripex |
|---|---|---|---|---|
| List profiles + mappings | `ghx x list` | `flyx x list` | `wranglerx x list` | `stripex x list` |
| Set the default | — | `flyx x use <p>` | `wranglerx x use <p>` | `stripex x use <p>` |
| Map a trigger → profile | `ghx x bind <user> <owner>` | (auto via cache) | `wranglerx x bind <p> <trigger>` | `stripex x bind <p> <trigger>` |
| Remove a mapping | `ghx x unbind <owner>` | — | `wranglerx x unbind <trigger>` | `stripex x unbind <trigger>` |
| Show resolution | `ghx x whoami` | `flyx x whoami` | `wranglerx x whoami` | `stripex x whoami` |

See each tool page for its full subcommand set; flyx and wranglerx add token
snapshot / refresh commands, for example.

## What is stored — and what is not

How much each tool stores depends on how the upstream CLI manages credentials:

- **ghx** stores **no tokens**. It relies on the tokens `gh` already manages and
  keeps only an owner → `gh` user map in `~/.config/ghx/accounts.yml`.
- **stripex** stores **no secrets**. Stripe's own `~/.config/stripe/config.toml`
  and keychain remain authoritative; stripex keeps only owner → project-name
  routing in `~/.config/stripex/projects.yml`.
- **flyx** snapshots Fly **access tokens** into `~/.config/flyx/profiles.yml`,
  plus an auto-built `mappings` cache of which app/org each token can see.
- **wranglerx** snapshots Cloudflare **OAuth tokens** (access + refresh) into
  `~/.config/wranglerx/profiles.yml` and refreshes them automatically when they
  expire.

> The `mappings` cache in flyx and wranglerx is rebuilt automatically (e.g.
> `flyx x refresh`). You don't need to hand-edit it.

## Disabling the update check

Each tool checks for a newer release when it prints its banner. To silence that,
set the per-tool env var:

```bash
export GHX_NO_UPDATE_CHECK=1
export FLYX_NO_UPDATE_CHECK=1
export WRANGLERX_NO_UPDATE_CHECK=1
export STRIPEX_NO_UPDATE_CHECK=1
```
