# How routing works

Every clix tool answers the same question before it runs: **which account
should this command use?** They all resolve it with the same kind of ordered
ladder — the first rule that produces a match wins. Understanding the ladder
once means you understand all four tools.

## The resolution ladder

From highest priority to lowest:

1. **Explicit credentials in the environment** — if the upstream CLI's own
   credential env var is already set, the tool passes the command through
   untouched. This is what keeps clix safe in CI: set the token in the
   environment and clix stays out of the way. (ghx is the exception — see the
   note under the table below.)
2. **Explicit override flag** — a per-invocation flag (`--profile`, `-p`,
   `--api-key`, …) forces a specific account for that one command.
3. **Resource flag** — a flag naming the target resource (`-a <app>`, an
   account id, …) is looked up in the tool's mappings cache.
4. **Project config in the current directory** — the tool walks up from the
   working directory and reads an id from the nearest project file
   (`fly.toml`, `wrangler.toml` / `wrangler.jsonc`, …), then looks that up in
   its mappings.
5. **Git remote owner** — the owner parsed from `git remote get-url origin` is
   matched against profiles / mappings. See
   [Git owner detection](owner-detection.md).
6. **Default profile** — a configured fallback (set with `<tool> x use <name>`).
7. **Pass through** — if nothing matches, the command is forwarded to the
   upstream CLI unchanged, so its own default behavior applies.

Not every tool uses every rung — but they never reorder them. A tool only ever
*omits* rungs that don't apply to it.

## What each rung means per tool

| Rung | ghx | flyx | wranglerx | stripex |
|---|---|---|---|---|
| Env passthrough | — (see note) | `FLY_API_TOKEN` / `FLY_ACCESS_TOKEN` | `CLOUDFLARE_API_TOKEN` / bare `CLOUDFLARE_ACCOUNT_ID` | `--api-key`, `STRIPE_API_KEY`, `STRIPE_SECRET_KEY` |
| Override flag | — | `--profile <name>` | `--profile <name>` | `-p` / `--project-name` |
| Resource flag | — | `-a` / `--app`, `-o` / `--org` | `--account-id <id>` | — |
| Project config | — | `fly.toml` `app` | `wrangler.toml` / `.jsonc` `account_id` | — |
| Git owner | `origin` owner → `gh` user / `accounts.yml` / org API | `origin` owner → profile `org_slugs` | `origin` owner | `origin` owner → project |
| Default profile | active `github.com.user` | `flyx x use <profile>` | `wranglerx x use <profile>` | `stripex x use <project>` |

> **ghx has no environment-passthrough rung.** For repository commands it
> *always* resolves the account from `gh`'s `hosts.yml` and sets `GH_TOKEN`
> itself — a pre-set `GH_TOKEN` is **overridden, not honored**. In CI,
> authenticate `gh` (so `hosts.yml` exists) rather than passing a bare
> `GH_TOKEN`.

The exact, authoritative order for each tool lives on its own page
([ghx](../tools/ghx.md), [flyx](../tools/flyx.md),
[wranglerx](../tools/wranglerx.md), [stripex](../tools/stripex.md)), because the
tools add tool-specific rungs (e.g. flyx also matches `-o <org-slug>`, and
wranglerx puts `--profile` ahead of the project-config and git-owner rungs).

## Inspecting a decision without running anything

`flyx`, `wranglerx`, and `stripex` accept `--dry-run`, which resolves the
account and prints what *would* run without invoking the upstream CLI:

```bash
wranglerx --dry-run deploy
stripex --dry-run customers list
```

For `ghx`, `ghx x whoami` shows the resolution for the current repository's
owner.
