# Releasing

Per-CLI releases are built and published **locally**, not in CI. A single
script — [`scripts/release.sh`](scripts/release.sh), wrapped by the
`task release:<tool>` tasks — builds every target, packages the binaries,
creates the GitHub Release on `nantokaworks/clix`, and updates the Homebrew
formula on `nantokaworks/homebrew-tap`.

This replaces the old tag-triggered `release-<tool>.yml` matrix builds, which
burned ~120 GitHub Actions minutes per tool (macOS runners bill at 10×). The CI
workflows are kept as a **manual-only fallback** (`workflow_dispatch`); see the
bottom of this doc.

## Prerequisites

- **Rust + rustup** (`cargo` on `PATH`). The build targets are installed
  automatically by the preflight stage.
- **Docker** (Docker Desktop or colima) **running** — the Linux and Windows
  targets cross-compile via [`cross`](https://github.com/cross-rs/cross).
  `cross` itself is auto-installed by preflight if missing.
- **macOS host** — the two macOS targets build natively.
- **`gh` authenticated** (`gh auth login`) with **Contents: Read & Write** on
  both `nantokaworks/clix` (to create the Release) and `nantokaworks/homebrew-tap`
  (to update the formula). The local script uses your own `gh` token — the CI
  `HOMEBREW_TAP_TOKEN` / `RELEASE_TAG_PAT` secrets are only for the fallback
  workflows.
- Optional: [`sccache`](https://github.com/mozilla/sccache) speeds up the native
  macOS builds; it is used automatically when present.

## Targets

| Triple | How | Asset |
|---|---|---|
| `aarch64-apple-darwin` | native `cargo` | `<tool>-v<ver>-aarch64-apple-darwin.tar.gz` |
| `x86_64-apple-darwin` | native `cargo` | `<tool>-v<ver>-x86_64-apple-darwin.tar.gz` |
| `x86_64-unknown-linux-gnu` | `cross` (Docker) | `<tool>-v<ver>-x86_64-unknown-linux-gnu.tar.gz` |
| `aarch64-unknown-linux-gnu` | `cross` (Docker) | `<tool>-v<ver>-aarch64-unknown-linux-gnu.tar.gz` |
| `x86_64-pc-windows-gnu` | `cross` (Docker) | `<tool>-v<ver>-x86_64-pc-windows-gnu.zip` |

> **Windows note:** the Windows target is now the **GNU ABI**
> (`x86_64-pc-windows-gnu`), not the previous MSVC ABI. The asset name changed
> from `…-windows-msvc.zip` to `…-windows-gnu.zip`. `install.sh` and the Homebrew
> formula do not reference the Windows asset, so this only affects direct
> downloads. Past `…-msvc.zip` assets on already-published releases are untouched.

## How to release

1. Bump the version in `crates/<tool>/Cargo.toml` and merge it to `main`.
   The script reads the version from `origin/main` and tags `origin/main`, so
   the release always matches `main` regardless of your local branch.
2. Make sure Docker is running, then run the pipeline:

   ```bash
   task release                     # release every tool whose main version is not yet tagged
   ```

   `task release` scans all four tools and releases only the ones whose
   `origin/main` version has no matching `<tool>-v<version>` tag yet; already
   released versions are skipped. To release a single tool explicitly:

   ```bash
   task release:ghx                 # full pipeline for ghx only
   task release:flyx
   task release:wranglerx
   task release:stripex
   ```

The pipeline runs seven stages per tool: `preflight → test → build → package →
tag → publish → verify`. The `test` stage runs `cargo test --locked -p <tool>`
against `origin/main`, so a failing test suite blocks the tag and publish.

> `task release` forwards flags too: `task release -- --dry-run` shows exactly
> which tools would be released and what each would do, without side effects.
> Resuming a single tool that failed mid-release (e.g. after the tag was pushed
> but the upload failed) is done per-tool: `task release:<tool> -- --from=publish`.

### Flags

Pass flags after `--`:

```bash
task release:ghx -- --dry-run         # rehearse: print every side-effecting action, do nothing
task release:ghx -- --to=build        # build all targets and stop (validate the toolchain)
task release:ghx -- --from=package --to=package   # repackage dist/ from an existing build
task release:ghx -- --from=publish    # re-run upload + Homebrew from existing dist/ (resume)
task release:ghx -- --notes="..."     # override the auto-generated release notes
```

You can also call the script directly: `bash scripts/release.sh <tool> [flags]`.

### Resume semantics

- `check` / `build` / `package` run inside a throwaway detached worktree at
  `origin/main`, created on demand and removed when the script exits. Re-running
  one of these stages on its own rebuilds from scratch (the worktree is
  ephemeral).
- `tag` / `publish` / `verify` operate on git and `dist/` only, so
  `--from=publish` cleanly resumes a release whose upload or Homebrew step failed,
  reusing the already-built `dist/` assets. The Release upload uses `--clobber`,
  so re-running `publish` is idempotent.

## CI fallback (manual only)

The matrix-build workflows still exist but no longer trigger on tag push. To
build a release in CI in a pinch (e.g. you cannot run Docker locally), dispatch
the workflow against an existing tag:

```bash
gh workflow run release-ghx.yml --ref ghx-v0.4.4
```

`auto-tag.yml` is likewise `workflow_dispatch`-only; tagging is otherwise handled
by the local pipeline's `tag` stage.
