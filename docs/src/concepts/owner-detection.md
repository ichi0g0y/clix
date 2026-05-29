# Git owner detection

Several rungs of the [routing ladder](routing.md) depend on the **owner** of the
current repository. clix derives it the same way everywhere: it runs

```bash
git remote get-url origin
```

and parses the owner out of the resulting URL.

## Supported remote URL forms

The owner is extracted from any of these shapes:

- `git@github.com:owner/repo.git`
- `git@<ssh-host-alias>:owner/repo.git` — when `ssh -G <ssh-host-alias>`
  resolves to `github.com`
- `https://github.com/owner/repo.git`
- `https://github.com/owner/repo`

In each case `owner` is the trigger that gets matched against your profiles and
mappings.

## What the owner resolves to

The owner is a *trigger*, not an account by itself. What it maps to depends on
the tool:

- **ghx** — if the owner matches a configured `gh` user it is used directly;
  if it is an organization, ghx checks `~/.config/ghx/accounts.yml`, then falls
  back to detecting org membership via the GitHub API.
- **flyx** — the owner is matched against each profile's `org_slugs`.
- **wranglerx** — the owner is tried when no `account_id` is found in a project
  config file.
- **stripex** — the owner is looked up in `~/.config/stripex/projects.yml` to
  pick a Stripe project name.

## Assumptions and limits

- Only the **`origin`** remote is inspected.
- Owner resolution is based on the **remote URL**, not deeper repository
  metadata.
- ghx currently assumes a **GitHub** remote.

If the current directory has no `origin` remote (or no git repository at all),
the owner rung is skipped and routing continues down the ladder to the default
profile / passthrough.
