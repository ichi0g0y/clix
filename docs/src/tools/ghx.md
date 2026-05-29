# ghx

**Wraps [`gh`](https://cli.github.com/); switches by git remote owner.**

`ghx` detects the repository owner from the `origin` remote, resolves the right
GitHub account, and runs `gh` with the matching token — no manual `gh auth
switch` when you `cd` between personal, work, and client repos. For the shared
resolution model see [How routing works](../concepts/routing.md).

> **Prerequisite:** log in to each account with `gh auth login` beforehand —
> `ghx` uses the tokens `gh` already manages and stores no tokens itself.

## Usage

Run `ghx` exactly like `gh`:

```bash
ghx pr status
ghx issue list
ghx repo view
```

Bootstrap commands are forwarded without account resolution, so `ghx` never
blocks basic `gh` usage when a repo or config is unavailable:

```bash
ghx help
ghx auth status
ghx auth login
```

Version commands print the `ghx` banner instead of forwarding:

```bash
ghx version
ghx --version
```

## Resolution order

For repository-aware commands `ghx`:

1. Runs `git remote get-url origin` and extracts the GitHub owner.
2. Reads `gh` config (`$GH_CONFIG_DIR`, `$XDG_CONFIG_HOME/gh`,
   `%APPDATA%/GitHub CLI` on Windows, or `~/.config/gh`) and loads `hosts.yml`.
3. Uses the owner if it matches a configured user under `github.com.users`.
4. Otherwise checks `~/.config/ghx/accounts.yml` for an explicit
   organization → account mapping.
5. Otherwise checks each configured user's org membership via the GitHub API.
6. Falls back to the active `github.com.user` if there is no match.
7. Runs `gh auth token -u <resolved-user>` and executes `gh` with `GH_TOKEN`
   set.

See [Git owner detection](../concepts/owner-detection.md) for the remote URL
forms that are understood.

## Organization mapping

When a repo is owned by an organization, `ghx` auto-detects which account is a
member via the GitHub API. To skip the API call and map explicitly, use the
`ghx x` namespace:

```bash
ghx x list                   # show gh accounts and current owner mappings
ghx x bind alice my-org      # map a gh user to an owner
ghx x unbind my-org          # delete a mapping
ghx x whoami                 # show resolution for the current repo's origin owner
```

`ghx x bind` writes `~/.config/ghx/accounts.yml`:

```yaml
accounts:
  my-org: my-username
  another-org: another-username
```

Explicit mappings take priority over API auto-detection.

## Example

If your `gh` config has accounts for `alice` and `acme-inc`:

- in `origin = git@github.com:alice/tooling.git` → `ghx` uses `alice`
- in `origin = git@github.com:acme-inc/backend.git` → `ghx` uses `acme-inc`

## Limitations

- Only the `origin` remote is inspected.
- Only GitHub remotes are supported.
- Owner resolution is based on the remote URL, not deeper repo metadata.
- Organization auto-detection requires network access to the GitHub API.

Running into errors? See [Troubleshooting](../troubleshooting.md).
