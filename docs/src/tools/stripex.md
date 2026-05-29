# stripex

**Wraps [`stripe`](https://docs.stripe.com/stripe-cli); switches by git remote
owner → project.**

`stripex` detects the git remote owner for the current directory, maps that
owner to a Stripe project name, injects `-p <name>`, and delegates to `stripe`.
For the shared resolution model see [How routing works](../concepts/routing.md).

> **Prerequisite:** install the Stripe CLI and sign in with `stripe login` for
> each project you want to use.

## Usage

Run `stripex` exactly like `stripe`:

```bash
stripex customers list
stripex listen
stripex whoami
```

Version commands print the `stripex` banner:

```bash
stripex version
stripex --version
```

Use `--dry-run` to inspect project resolution without running `stripe`:

```bash
stripex --dry-run customers list
```

## Resolution order

1. If you pass `-p` / `--project-name`, `stripex` forwards the command
   unchanged.
2. If you pass `--api-key`, or set `STRIPE_API_KEY` / `STRIPE_SECRET_KEY`,
   `stripex` forwards the command unchanged.
3. Otherwise it reads the current git remote owner, looks it up in
   `~/.config/stripex/projects.yml`, and injects `-p <project>`.
4. If no git owner mapping exists and a default project is configured with
   `stripex x use <project>`, that default is used when no git remote owner can
   be detected.
5. If no project can be resolved, `stripex` delegates to `stripe` unchanged so
   the Stripe CLI uses its own default project.

## Project management

`stripex` reads available project names from Stripe's own
`~/.config/stripe/config.toml`, then stores only owner → project routing
preferences:

```bash
stripex x list                         # show Stripe CLI projects and mappings
stripex x bind <project> <trigger>     # map a git remote owner to a project
stripex x unbind <trigger>             # delete a mapping
stripex x use <project>                # set the fallback project
stripex x whoami [<project>]           # show project authentication details
```

Example:

```bash
stripex x bind work acme-corp
stripex --dry-run customers list
```

`stripex login` passes through to `stripe login`. When it succeeds with a
project flag such as `stripex login -p work`, `stripex` automatically binds the
current git remote owner to that project if no mapping already exists.

Mappings live in `~/.config/stripex/projects.yml`:

```yaml
default: personal
mappings:
  acme-corp: work
  personal-org: personal
```

## Security

`stripex` stores **no API keys, tokens, or secrets**. It only stores the
non-secret owner → project-name mapping in `~/.config/stripex/projects.yml`.

Stripe's own `~/.config/stripe/config.toml` and keychain-backed credentials
remain authoritative for authentication. Explicit credentials supplied through
`--api-key`, `STRIPE_API_KEY`, or `STRIPE_SECRET_KEY` always pass through
untouched.
