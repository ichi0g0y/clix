# wranglerx

**Wraps [`wrangler`](https://developers.cloudflare.com/workers/wrangler/);
switches by `wrangler.toml` `account_id` / git remote owner.**

`wranglerx` snapshots the OAuth tokens from `wrangler login`, detects
`account_id` from `wrangler.toml` / `wrangler.jsonc` (or falls back to the
GitHub remote owner / a default profile), refreshes expired tokens
automatically, and runs `wrangler` with the matching `CLOUDFLARE_API_TOKEN` and
`CLOUDFLARE_ACCOUNT_ID`. For the shared resolution model see
[How routing works](../concepts/routing.md).

> **Prerequisite:** install Wrangler, sign in with `wrangler login` for each
> account, and snapshot each session with `wranglerx x save <profile>`.

## Usage

Run `wranglerx` exactly like `wrangler`:

```bash
wranglerx deploy
wranglerx dev
wranglerx whoami
```

Bootstrap commands are forwarded without account resolution:

```bash
wranglerx help
wranglerx login
wranglerx logout
```

Version commands print the `wranglerx` banner:

```bash
wranglerx version
wranglerx --version
```

Use `--dry-run` to inspect the selected account without running `wrangler`:

```bash
wranglerx --dry-run deploy
```

## Resolution order

1. If `CLOUDFLARE_ACCOUNT_ID` is already set, `wranglerx` passes through to
   `wrangler` unchanged (CI workflows).
2. Otherwise it walks up from the current directory and reads top-level
   `account_id` from the nearest `wrangler.toml` or `wrangler.jsonc`.
3. If no project account id is found, it tries the GitHub owner from
   `git remote get-url origin`.
4. If neither source yields a hit, it falls back to the `default` profile (set
   via `wranglerx x use <profile>`).
5. The trigger key is matched against `mappings`, then against each profile's
   `account_id` / `account_ids`.

If the resolved profile's `expiration_time` is past (or within 60 seconds),
`wranglerx` automatically refreshes the OAuth token using `refresh_token` and
rewrites `profiles.yml` before invoking `wrangler`.

## Profile management

Sign in with vanilla `wrangler login`, then snapshot the OAuth credentials into
a named profile:

```bash
wrangler login                           # browser-based OAuth flow for account A
wranglerx x save personal                # snapshot to "personal" profile

wrangler logout
wrangler login                           # browser-based OAuth flow for account B
wranglerx x save work                    # snapshot to "work" profile
```

Subcommands:

```bash
wranglerx x list                         # show profiles, account_ids, expirations
wranglerx x use <profile>                # set the default fallback profile
wranglerx x bind <profile> <trigger>     # map a trigger (account_id) to a profile
wranglerx x unbind <trigger>             # delete a mapping
wranglerx x remove <profile>             # delete a profile
wranglerx x refresh <profile>            # force OAuth refresh
wranglerx x whoami [<profile>]           # show profile details
```

Snapshots live in `~/.config/wranglerx/profiles.yml`:

```yaml
default: personal
profiles:
  personal:
    access_token: <oauth-access-token>
    refresh_token: <oauth-refresh-token>
    expiration_time: 2026-05-03T13:34:56Z
    account_id: 1234abcd
    account_ids: [1234abcd]
    scopes: [account:read, workers:write, ...]
  work:
    access_token: ...
mappings:
  1234abcd: personal
  myorg: work
```

`wranglerx login` and `wranglerx logout` pass through to vanilla `wrangler`
without touching the profile store, so the OAuth flow remains untouched.

## Env vars

```bash
export WRANGLERX_NO_UPDATE_CHECK=1   # silence the version banner update check
```

In CI, set `CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN` directly —
`wranglerx` passes them through to `wrangler` without touching the profile
store.
