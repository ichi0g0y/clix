# flyx

**Wraps [`fly`](https://fly.io/docs/flyctl/); switches by `fly.toml` app / git
remote owner.**

`flyx` snapshots your Fly tokens, reads the `app` from `fly.toml` (or the `-a` /
`-o` flag), figures out which account owns it, and runs `fly` with the matching
`FLY_API_TOKEN`. For the shared resolution model see
[How routing works](../concepts/routing.md).

> **Prerequisite:** install flyctl first — `flyx` shells out to it for every
> operation that talks to Fly.io.

## Quick start

```bash
flyx auth login                # OAuth in browser; flyx auto-snapshots the result
flyx deploy                    # picks the right token based on cwd's fly.toml
flyx --profile work logs       # one-off override
```

The profile name is auto-derived from your Fly email's local-part (e.g.
`you@example.com` → `you`). You can edit `~/.config/flyx/profiles.yml` directly
to rename it.

## Resolution order

`flyx` resolves which token to use in this order:

1. `FLY_API_TOKEN` / `FLY_ACCESS_TOKEN` already set in env → pass through
   unchanged (CI workflows).
2. `flyx --profile <name> <cmd>` → use that profile's token explicitly.
3. `-a <app>` / `--app <app>` → mappings cache → profile owning the app.
4. `fly.toml` `app` (walked up from cwd) → mappings cache → profile owning the
   app.
5. `-o <slug>` / `--org <slug>` → match against profile `org_slugs`.
6. git remote owner (`git remote get-url origin`) → match against profile
   `org_slugs`.
7. default profile (set with `flyx x use <profile>`).

The `mappings` cache is populated automatically when you run `flyx auth login`,
`flyx x refresh`, or `flyx x save-token` — `fly apps list --json` returns every
app the token can see, cached for offline routing.

## Commands

```bash
# Login / signup — these REPLACE manual snapshot steps.
flyx auth login                       # OAuth + auto-snapshot (name derived from email)
flyx auth signup                      # signup + auto-snapshot
flyx auth logout                      # passthrough; profile store untouched

# Profile management
flyx x list                           # list profiles + cached mappings (auto-syncs from ~/.fly/)
flyx x use <profile>                  # change the default profile
flyx x remove <profile>               # delete a profile
flyx x refresh [<profile>]            # re-probe orgs/apps via flyctl
flyx x save-token <name> <token>      # register a paste-in token (e.g. `fly tokens create org`)
flyx x import                         # explicit scan of ~/.fly/config*.yml
flyx x whoami [<profile>]             # show profile details
```

`flyx x list` and `flyx x whoami` automatically sync from `~/.fly/config*.yml`:
if you ran `fly auth login` outside of flyx, the new token is detected and
snapshotted on the next list/whoami invocation (matched by root macaroon, so
rotated discharges don't double-import).

## Profile YAML

Snapshots live in `~/.config/flyx/profiles.yml`:

```yaml
default: ichi
profiles:
  ichi:
    access_token: fm2_lJPECAA...
    email: you@example.com
    org_slug: personal
    org_slugs: [personal, nantokaworks]
  work:
    access_token: fo1_xxx...
    email: you@work.example.com
    org_slug: acme
    org_slugs: [acme]
mappings:                       # auto-populated cache; do not hand-edit
  some-app: ichi
  acme: work
```

Hand-editing `mappings` is unnecessary — `refresh` rebuilds it. To override
routing for a single invocation, use `flyx --profile <name>`.

## Env vars

```bash
export FLYX_NO_UPDATE_CHECK=1   # silence the version banner update check
```

In CI, set `FLY_API_TOKEN` directly — `flyx` passes it through to `fly` without
touching the profile store.
