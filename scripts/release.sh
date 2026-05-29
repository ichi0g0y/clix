#!/usr/bin/env bash
# release.sh — local per-CLI release pipeline; see RELEASING.md for the full guide.
#
# Usage: scripts/release.sh <tool> [--from=<stage>] [--to=<stage>] [--dry-run] [--notes=<text>]
#   <tool> ∈ {ghx, flyx, wranglerx, stripex}
#   stages: preflight -> test -> build -> package -> tag -> publish -> verify
#
# Reads the version from origin/main:crates/<tool>/Cargo.toml and tags origin/main,
# so the release matches main regardless of the local branch. test/build/package
# run in a throwaway origin/main worktree; tag/publish/verify use git + dist/ only,
# so --from=publish resumes from already-built dist/ assets. Publishes to
# nantokaworks/clix Releases + nantokaworks/homebrew-tap. Needs gh auth
# (contents:write on both) and a running Docker daemon for `cross`.

set -euo pipefail

TOOL=""
DRY_RUN=0
FROM_STAGE="preflight"
TO_STAGE="verify"
NOTES_OVERRIDE=""

usage() {
  cat <<'USAGE'
Usage: scripts/release.sh <tool> [--from=<stage>] [--to=<stage>] [--dry-run] [--notes=<text>]

  <tool>            One of: ghx, flyx, wranglerx, stripex.
Stages (in order): preflight, test, build, package, tag, publish, verify.
  --from=<stage>    Start at this stage (default: preflight).
  --to=<stage>      Stop after this stage (default: verify).
  --dry-run         Print side-effecting actions without performing them.
  --notes=<text>    Override the GitHub Release notes body (default: auto-generated).
USAGE
}

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    --from=*) FROM_STAGE="${arg#--from=}" ;;
    --to=*) TO_STAGE="${arg#--to=}" ;;
    --notes=*) NOTES_OVERRIDE="${arg#--notes=}" ;;
    -h|--help) usage; exit 0 ;;
    -*) echo "error: unknown flag: $arg" >&2; usage >&2; exit 2 ;;
    *)
      if [ -n "$TOOL" ]; then echo "error: unexpected argument: $arg" >&2; usage >&2; exit 2; fi
      TOOL="$arg"
      ;;
  esac
done

case "$TOOL" in
  ghx|flyx|wranglerx|stripex) ;;
  "") echo "error: missing <tool> argument" >&2; usage >&2; exit 2 ;;
  *) echo "error: unknown tool: $TOOL (expected ghx|flyx|wranglerx|stripex)" >&2; exit 2 ;;
esac

for stage in "$FROM_STAGE" "$TO_STAGE"; do
  case "$stage" in
    preflight|test|build|package|tag|publish|verify) ;;
    *) echo "error: invalid stage: $stage" >&2; exit 2 ;;
  esac
done

# Per-tool Homebrew metadata. Binary name == tool name; test command is
# "<tool> version" for all four. Kept verbatim-equivalent to the existing
# release-<tool>.yml so the regenerated formula diffs cleanly against the tap.
case "$TOOL" in
  ghx)       FORMULA_CLASS="Ghx";       DESC="Thin wrapper around gh for multi-account GitHub usage" ;;
  flyx)      FORMULA_CLASS="Flyx";      DESC="Multi-account Fly.io CLI wrapper, powered by fly" ;;
  wranglerx) FORMULA_CLASS="Wranglerx"; DESC="Multi-account Cloudflare Wrangler CLI, powered by wrangler" ;;
  stripex)   FORMULA_CLASS="Stripex";   DESC="Multi-project Stripe CLI, powered by stripe" ;;
esac

WORKSPACE_ROOT="$(git rev-parse --show-toplevel)"
cd "$WORKSPACE_ROOT"

DIST_DIR="$WORKSPACE_ROOT/dist"
RELEASES_REPO="nantokaworks/clix"
HOMEBREW_TAP_REPO="nantokaworks/homebrew-tap"
FORMULA_PATH="Formula/${TOOL}.rb"
HOMEPAGE="https://github.com/nantokaworks/clix"

# target_triple:build_tool. macOS builds native via cargo; Linux + Windows
# cross-compile via `cross` (Docker). Windows is the GNU ABI target.
TARGETS=(
  "aarch64-apple-darwin:cargo"
  "x86_64-apple-darwin:cargo"
  "x86_64-unknown-linux-gnu:cross"
  "aarch64-unknown-linux-gnu:cross"
  "x86_64-pc-windows-gnu:cross"
)

# Local sccache routing (host macOS). The cross arm unsets RUSTC_WRAPPER
# per-invocation so cross containers don't try to resolve a Linux sccache.
if command -v sccache >/dev/null 2>&1; then
  export RUSTC_WRAPPER=sccache
  export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}"
fi

log() { printf '[release] %s\n' "$*"; }
die() { printf '[release] error: %s\n' "$*" >&2; exit 1; }

run() {
  # Caller passes a single shell-quoted command string; in dry-run it is printed
  # instead of executed so subshell (cd && cmd) forms work uniformly.
  if [ "$DRY_RUN" -eq 1 ]; then
    printf '[dry-run] %s\n' "$1"
  else
    bash -c "$1"
  fi
}

stage_active() {
  local stage="$1" order=(preflight test build package tag publish verify)
  local start_idx=-1 end_idx=-1 cur_idx=-1 i
  for i in "${!order[@]}"; do
    [ "${order[$i]}" = "$FROM_STAGE" ] && start_idx=$i
    [ "${order[$i]}" = "$TO_STAGE" ] && end_idx=$i
    [ "${order[$i]}" = "$stage" ] && cur_idx=$i
  done
  [ "$cur_idx" -ge "$start_idx" ] && [ "$cur_idx" -le "$end_idx" ]
}

bin_filename() { case "$1" in *windows*) printf '%s.exe' "$TOOL" ;; *) printf '%s' "$TOOL" ;; esac; }
asset_filename() { case "$1" in *windows*) printf '%s-v%s-%s.zip' "$TOOL" "$VERSION" "$1" ;; *) printf '%s-v%s-%s.tar.gz' "$TOOL" "$VERSION" "$1" ;; esac; }

# origin/main worktree (created lazily; only test/build/package need it).
WORKTREE=""
cleanup() { [ -n "$WORKTREE" ] && git worktree remove --force "$WORKTREE" >/dev/null 2>&1 || true; }
trap cleanup EXIT

ensure_worktree() {
  [ -n "$WORKTREE" ] && return 0
  if [ "$DRY_RUN" -eq 1 ]; then
    WORKTREE="<origin/main-worktree>"
    log "dry-run: would create detached worktree at origin/main"
    return 0
  fi
  WORKTREE="$(mktemp -d "${TMPDIR:-/tmp}/${TOOL}-release.XXXXXX")"
  log "creating detached worktree at origin/main: $WORKTREE"
  git worktree add --detach "$WORKTREE" origin/main >/dev/null
}

# ---------------------------------------------------------------------------
git fetch origin main --tags --prune >/dev/null 2>&1 || die "git fetch origin failed"
VERSION="$(git show "origin/main:crates/${TOOL}/Cargo.toml" | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/')"
[ -n "$VERSION" ] || die "could not read version from origin/main:crates/${TOOL}/Cargo.toml"
TAG="${TOOL}-v${VERSION}"
log "tool=${TOOL} version=${VERSION} tag=${TAG} root=${WORKSPACE_ROOT}"
[ "$DRY_RUN" -eq 1 ] && log "DRY-RUN: side-effecting commands will be printed only"

# ---------------------------------------------------------------------------
# preflight
# ---------------------------------------------------------------------------
if stage_active preflight; then
  log "stage: preflight"

  if git ls-remote --exit-code --tags origin "refs/tags/${TAG}" >/dev/null 2>&1; then
    die "tag ${TAG} already exists on origin; bump crates/${TOOL}/Cargo.toml first"
  fi

  command -v gh >/dev/null || die "gh CLI not found"
  gh auth status >/dev/null 2>&1 || die "gh not authenticated (run: gh auth login)"
  gh api "repos/${RELEASES_REPO}" --jq .full_name >/dev/null \
    || die "current gh token cannot read ${RELEASES_REPO}; check token scope (needs contents:write)"
  gh api "repos/${HOMEBREW_TAP_REPO}" --jq .full_name >/dev/null \
    || die "current gh token cannot read ${HOMEBREW_TAP_REPO}; check token scope (needs contents:write)"

  command -v cargo >/dev/null || die "cargo not found"
  command -v rustup >/dev/null || die "rustup not found"
  if ! command -v cross >/dev/null; then
    log "cross not found; installing via: cargo install cross --git https://github.com/cross-rs/cross"
    run "cargo install cross --git https://github.com/cross-rs/cross"
    if [ "$DRY_RUN" -eq 0 ]; then
      cargo_bin="${CARGO_INSTALL_ROOT:-${CARGO_HOME:-$HOME/.cargo}}/bin"
      case ":${PATH}:" in *":${cargo_bin}:"*) ;; *) PATH="${cargo_bin}:${PATH}"; export PATH ;; esac
      command -v cross >/dev/null || die "cross install reported success but is not on PATH (expected ${cargo_bin}/cross)"
    fi
  fi

  if [ "$DRY_RUN" -eq 0 ] && ! docker info >/dev/null 2>&1; then
    die "Docker daemon not reachable; start Docker Desktop / colima before running (cross needs it)"
  fi

  for entry in "${TARGETS[@]}"; do
    triple="${entry%%:*}"
    if [ "$DRY_RUN" -eq 1 ]; then
      log "dry-run: would ensure rustup target ${triple}"
    elif ! rustup target list --installed | grep -qx "$triple"; then
      log "installing rust target ${triple}"
      rustup target add "$triple"
    fi
  done

  log "preflight OK"
fi

# ---------------------------------------------------------------------------
# test — build + run the test suite before any tag/publish (gate on a red main)
# ---------------------------------------------------------------------------
if stage_active test; then
  log "stage: test"
  ensure_worktree
  run "(cd \"${WORKTREE}\" && cargo test --locked -p ${TOOL})"
fi

# ---------------------------------------------------------------------------
# build
# ---------------------------------------------------------------------------
if stage_active build; then
  log "stage: build"
  ensure_worktree
  for entry in "${TARGETS[@]}"; do
    triple="${entry%%:*}"; tool="${entry##*:}"
    log "building ${triple} via ${tool}"
    if [ "$tool" = "cross" ]; then
      run "(cd \"${WORKTREE}\" && RUSTC_WRAPPER= cross build --release -p ${TOOL} --target ${triple})"
    else
      run "(cd \"${WORKTREE}\" && cargo build --release -p ${TOOL} --target ${triple})"
    fi
  done
fi

# ---------------------------------------------------------------------------
# package
# ---------------------------------------------------------------------------
if stage_active package; then
  log "stage: package"
  ensure_worktree
  run "mkdir -p \"${DIST_DIR}\""
  for entry in "${TARGETS[@]}"; do
    triple="${entry%%:*}"
    binfile="$(bin_filename "$triple")"
    asset="$(asset_filename "$triple")"
    bin_dir="${WORKTREE}/target/${triple}/release"
    if [ "$DRY_RUN" -eq 0 ] && [ ! -e "${bin_dir}/${binfile}" ]; then
      die "binary not found at ${bin_dir}/${binfile}; rerun with --from=build"
    fi
    log "packaging ${asset}"
    case "$triple" in
      *windows*) run "(cd \"${bin_dir}\" && zip -q \"${DIST_DIR}/${asset}\" \"${binfile}\")" ;;
      *)         run "tar -C \"${bin_dir}\" -czf \"${DIST_DIR}/${asset}\" \"${binfile}\"" ;;
    esac
  done

  checksums="${TOOL}-v${VERSION}-checksums.txt"
  log "writing ${checksums}"
  if [ "$DRY_RUN" -eq 1 ]; then
    printf '[dry-run] (cd %s && shasum -a 256 %s-v%s-* > %s)\n' "$DIST_DIR" "$TOOL" "$VERSION" "$checksums"
  else
    assets=()
    for entry in "${TARGETS[@]}"; do assets+=("$(asset_filename "${entry%%:*}")"); done
    (cd "$DIST_DIR" && shasum -a 256 "${assets[@]}" >"$checksums")
  fi
fi

# ---------------------------------------------------------------------------
# tag
# ---------------------------------------------------------------------------
if stage_active tag; then
  log "stage: tag"
  if git ls-remote --exit-code --tags origin "refs/tags/${TAG}" >/dev/null 2>&1; then
    log "remote tag ${TAG} already exists; skipping tag creation and push"
  else
    # Never push a stale same-name local tag as-is (e.g. left by an aborted
    # release pointing at an old commit): force it onto origin/main first, so
    # the released commit is always exactly origin/main.
    origin_main="$(git rev-parse origin/main)"
    if git rev-parse -q --verify "refs/tags/${TAG}" >/dev/null 2>&1 \
       && [ "$(git rev-parse "refs/tags/${TAG}^{commit}")" = "$origin_main" ]; then
      log "local tag ${TAG} already points at origin/main; keeping"
    else
      log "creating tag ${TAG} at origin/main (${origin_main})"
      run "git tag -f -a \"${TAG}\" origin/main -m \"Release ${TOOL} ${VERSION}\""
    fi
    run "git push origin \"refs/tags/${TAG}\""
  fi
fi

# ---------------------------------------------------------------------------
# publish
# ---------------------------------------------------------------------------
sha_for() { shasum -a 256 "$1" | awk '{print $1}'; }

render_formula() {
  local base_url="https://github.com/${RELEASES_REPO}/releases/download/${TAG}"
  local sha_macos_arm sha_macos_x86 sha_linux_arm sha_linux_x86
  sha_macos_arm="$(sha_for "${DIST_DIR}/${TOOL}-v${VERSION}-aarch64-apple-darwin.tar.gz")"
  sha_macos_x86="$(sha_for "${DIST_DIR}/${TOOL}-v${VERSION}-x86_64-apple-darwin.tar.gz")"
  sha_linux_arm="$(sha_for "${DIST_DIR}/${TOOL}-v${VERSION}-aarch64-unknown-linux-gnu.tar.gz")"
  sha_linux_x86="$(sha_for "${DIST_DIR}/${TOOL}-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz")"
  cat <<RUBY
class ${FORMULA_CLASS} < Formula
  desc "${DESC}"
  homepage "${HOMEPAGE}"
  license "MIT"

  livecheck do
    url :stable
    strategy :github_latest
  end

  on_macos do
    on_arm do
      url "${base_url}/${TOOL}-v${VERSION}-aarch64-apple-darwin.tar.gz"
      sha256 "${sha_macos_arm}"
    end
    on_intel do
      url "${base_url}/${TOOL}-v${VERSION}-x86_64-apple-darwin.tar.gz"
      sha256 "${sha_macos_x86}"
    end
  end

  on_linux do
    on_arm do
      url "${base_url}/${TOOL}-v${VERSION}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${sha_linux_arm}"
    end
    on_intel do
      url "${base_url}/${TOOL}-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${sha_linux_x86}"
    end
  end

  def install
    bin.install "${TOOL}"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/${TOOL} version 2>&1")
  end
end
RUBY
}

if stage_active publish; then
  log "stage: publish"

  assets=()
  for entry in "${TARGETS[@]}"; do assets+=("${DIST_DIR}/$(asset_filename "${entry%%:*}")"); done
  assets+=("${DIST_DIR}/${TOOL}-v${VERSION}-checksums.txt")
  if [ "$DRY_RUN" -eq 0 ]; then
    for a in "${assets[@]}"; do [ -e "$a" ] || die "asset missing: $a; rerun with --from=build"; done
  fi

  if gh release view "${TAG}" --repo "${RELEASES_REPO}" >/dev/null 2>&1; then
    log "release ${TAG} exists on ${RELEASES_REPO}; uploading assets with --clobber"
    if [ "$DRY_RUN" -eq 1 ]; then
      printf '[dry-run] gh release upload %s --repo %s --clobber %s\n' "$TAG" "$RELEASES_REPO" "${assets[*]}"
    else
      gh release upload "$TAG" --repo "$RELEASES_REPO" --clobber "${assets[@]}"
    fi
  else
    log "creating GitHub Release ${TAG} on ${RELEASES_REPO}"
    if [ "$DRY_RUN" -eq 1 ]; then
      printf '[dry-run] gh release create %s --repo %s --title %s %s %s\n' "$TAG" "$RELEASES_REPO" "$TAG" \
        "$([ -n "$NOTES_OVERRIDE" ] && echo "--notes ..." || echo "--generate-notes")" "${assets[*]}"
    else
      notes_args=()
      if [ -n "$NOTES_OVERRIDE" ]; then notes_args=(--notes "$NOTES_OVERRIDE"); else notes_args=(--generate-notes); fi
      gh release create "$TAG" --repo "$RELEASES_REPO" --title "$TAG" --verify-tag "${notes_args[@]}" "${assets[@]}"
    fi
  fi

  log "updating Homebrew formula at ${HOMEBREW_TAP_REPO}/${FORMULA_PATH}"
  if [ "$DRY_RUN" -eq 1 ]; then
    printf '[dry-run] would PUT %s/%s with regenerated formula\n' "$HOMEBREW_TAP_REPO" "$FORMULA_PATH"
  else
    formula_tmp="$(mktemp)"
    render_formula >"$formula_tmp"
    current_sha="$(gh api "repos/${HOMEBREW_TAP_REPO}/contents/${FORMULA_PATH}" --jq .sha 2>/dev/null || true)"
    content_b64="$(base64 <"$formula_tmp" | tr -d '\n')"
    api_args=(-X PUT "repos/${HOMEBREW_TAP_REPO}/contents/${FORMULA_PATH}"
              -f "message=${TOOL} ${VERSION}" -f "content=${content_b64}")
    [ -n "$current_sha" ] && api_args+=(-f "sha=${current_sha}")
    gh api "${api_args[@]}" >/dev/null
    rm -f "$formula_tmp"
  fi
fi

# ---------------------------------------------------------------------------
# verify
# ---------------------------------------------------------------------------
if stage_active verify; then
  log "stage: verify"
  if [ "$DRY_RUN" -eq 1 ]; then
    log "dry-run: skipping live verification"
  else
    expected=$(( ${#TARGETS[@]} + 1 ))
    actual="$(gh release view "${TAG}" --repo "${RELEASES_REPO}" --json assets --jq '.assets | length' 2>/dev/null || echo 0)"
    log "release ${TAG} assets: ${actual} (expected ${expected})"
    [ "$actual" -ge "$expected" ] || log "warning: fewer assets than expected on ${TAG}"
    if command -v brew >/dev/null; then
      log "running brew update (may take a moment)"
      brew update >/dev/null || true
      # Homebrew strips the `homebrew-` prefix: nantokaworks/homebrew-tap -> nantokaworks/tap.
      brew info "nantokaworks/tap/${TOOL}" || log "warning: brew info failed (tap added? run: brew tap nantokaworks/tap)"
    else
      log "brew not installed; skipping Homebrew info check"
    fi
  fi
  log "release complete: ${TAG}"
fi
