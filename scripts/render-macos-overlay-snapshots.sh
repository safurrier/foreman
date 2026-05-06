#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${FOREMAN_OVERLAY_SNAPSHOT_DIR:-$ROOT/.ai/validation/macos-overlay/headless-snapshots}"
STATE="${FOREMAN_OVERLAY_SNAPSHOT_STATE:-all}"

swift build --package-path "$ROOT/apps/macos-overlay" >/dev/null
SNAPSHOT_BIN="$ROOT/apps/macos-overlay/.build/debug/foreman-overlay-snapshot"
"$SNAPSHOT_BIN" \
  --repo-root "$ROOT" \
  --state "$STATE" \
  --output-dir "$OUT_DIR"

if [ "$STATE" = "all" ]; then
  while IFS= read -r state; do
    [ -n "$state" ] || continue
    if [ ! -s "$OUT_DIR/$state.png" ]; then
      echo "missing headless snapshot: $OUT_DIR/$state.png" >&2
      exit 1
    fi
  done < <("$SNAPSHOT_BIN" --repo-root "$ROOT" --list-states)
else
  if [ ! -s "$OUT_DIR/$STATE.png" ]; then
    echo "missing headless snapshot: $OUT_DIR/$STATE.png" >&2
    exit 1
  fi
fi

swift "$ROOT/scripts/verify-macos-overlay-snapshot-text.swift" "$OUT_DIR"

echo "Headless macOS overlay snapshots written to $OUT_DIR"
