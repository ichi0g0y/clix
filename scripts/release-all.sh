#!/usr/bin/env bash
# release-all.sh — release every tool whose origin/main version is not yet tagged.
#
# Scans ghx / flyx / wranglerx / stripex: a tool is "releasable" when the version
# in origin/main:crates/<tool>/Cargo.toml has no matching <tool>-v<version> tag on
# origin. Already-tagged tools are skipped. Then runs scripts/release.sh for each
# releasable tool, forwarding all flags (e.g. --dry-run, --to=build).
#
# Resume of a partially-failed single release is per-tool (the tag stage pushes
# the tag, after which the tool is no longer "releasable" here): use
# `task release:<tool> -- --from=publish` for that.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLS=(ghx flyx wranglerx stripex)

log() { printf '[release-all] %s\n' "$*"; }

git fetch origin main --tags --prune >/dev/null 2>&1 || { echo "[release-all] error: git fetch origin failed" >&2; exit 1; }

releasable=()
log "scanning origin/main versions vs existing tags:"
for tool in "${TOOLS[@]}"; do
  version="$(git show "origin/main:crates/${tool}/Cargo.toml" | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/')"
  if [ -z "$version" ]; then
    log "  ${tool}: could not read version; skipping"
    continue
  fi
  tag="${tool}-v${version}"
  if git ls-remote --exit-code --tags origin "refs/tags/${tag}" >/dev/null 2>&1; then
    log "  ${tool}: ${version} already released (${tag}); skip"
  else
    log "  ${tool}: ${version} -> ${tag} (releasable)"
    releasable+=("$tool")
  fi
done

if [ "${#releasable[@]}" -eq 0 ]; then
  log "nothing to release; every tool version is already tagged on origin."
  exit 0
fi

log "releasing: ${releasable[*]}"
for tool in "${releasable[@]}"; do
  log "===== ${tool} ====="
  bash "${SCRIPT_DIR}/release.sh" "$tool" "$@"
done
log "all done: ${releasable[*]}"
