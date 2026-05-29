# Troubleshooting

## "<cli> not found"

Each clix tool shells out to its upstream CLI, so that binary must be installed
and on your `PATH`. If it isn't, the tool tells you what to check. For example:

```bash
$ ghx pr status
ghx: gh not found
  Check: gh --version
  After installing, run: gh auth login
  https://cli.github.com/
```

Install the upstream CLI (`gh`, `fly`, `wrangler`, or `stripe`) and re-run.

## ghx: "gh config not found: ~/.config/gh/hosts.yml"

`gh` is installed but you have not logged in yet, so there are no accounts to
choose from:

```bash
$ ghx pr status
ghx: gh config not found: ~/.config/gh/hosts.yml
  Run: gh auth login
```

Run `gh auth login` for each account you want `ghx` to route between.

## The wrong account is being used

Confirm what the tool resolved before it runs the upstream CLI:

- **flyx / wranglerx / stripex** — add `--dry-run` to see the selected account
  without executing.
- **ghx** — run `ghx x whoami` to see the resolution for the current repo's
  `origin` owner.

Then walk the [resolution ladder](concepts/routing.md):

1. Is an upstream credential env var set? That wins and forces passthrough
   (common in CI). Unset it for local use.
2. Is there a project config (`fly.toml`, `wrangler.toml`) or `origin` remote in
   this directory? Check it points where you expect.
3. Is the owner / app / account id actually mapped to the profile you want?
   List the mappings (`<tool> x list`) and bind it if missing.

## No profile / mapping is saved

If routing falls all the way through to passthrough, you likely have not
registered the account yet. See
[Profiles and mappings](concepts/profiles.md) for how each tool stores accounts,
then bind a mapping (`<tool> x bind …`) or set a default (`<tool> x use …`).

## CI is picking the wrong (or no) account

For flyx, wranglerx, and stripex: set the upstream credential directly in CI and
the wrapper passes it through untouched — it never reads the profile store.
**ghx is the exception** — it always resolves from `gh`'s `hosts.yml` and sets
`GH_TOKEN` itself, so authenticate `gh` rather than relying on a preset token.

| Tool | Set in CI |
|---|---|
| ghx | run `gh auth login` so `hosts.yml` exists — a preset `GH_TOKEN` is **not** honored |
| flyx | `FLY_API_TOKEN` |
| wranglerx | `CLOUDFLARE_API_TOKEN` (+ `CLOUDFLARE_ACCOUNT_ID`) |
| stripex | `STRIPE_API_KEY` / `STRIPE_SECRET_KEY` (or `--api-key`) |
