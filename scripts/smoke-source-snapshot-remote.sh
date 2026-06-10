#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FOREMAN_BIN="${FOREMAN_BIN:-$ROOT/target/debug/foreman}"
REMOTE_HOST="${FOREMAN_REMOTE_DEV_HOST:-remote-dev}"
REMOTE_FOREMAN="${FOREMAN_REMOTE_FOREMAN:-foreman}"
REMOTE_SNAPSHOT="${FOREMAN_REMOTE_SNAPSHOT:-/tmp/foreman-workstation-source-snapshot.json}"
REMOTE_CONFIG="${FOREMAN_REMOTE_CONFIG:-/tmp/foreman-snapshot-config.toml}"
REMOTE_LOG_DIR="${FOREMAN_REMOTE_LOG_DIR:-/tmp/foreman-snapshot-logs}"
LOCAL_SNAPSHOT="$(mktemp -t foreman-workstation-source-snapshot).json"

cleanup() {
  rm -f "$LOCAL_SNAPSHOT"
}
trap cleanup EXIT

if ! ssh -o BatchMode=yes -o ConnectTimeout=5 "$REMOTE_HOST" 'true' >/dev/null 2>&1; then
  echo "skip: cannot reach $REMOTE_HOST with batch SSH" >&2
  exit 2
fi

cargo build --quiet --manifest-path "$ROOT/Cargo.toml" --bin foreman

"$FOREMAN_BIN" sources snapshot --source-id workstation --output "$LOCAL_SNAPSHOT" --json \
  | jq '{ok, sourceId, entryCount, path}'
scp -q "$LOCAL_SNAPSHOT" "$REMOTE_HOST:$REMOTE_SNAPSHOT"

ssh "$REMOTE_HOST" "REMOTE_FOREMAN='$REMOTE_FOREMAN' REMOTE_CONFIG='$REMOTE_CONFIG' REMOTE_LOG_DIR='$REMOTE_LOG_DIR' REMOTE_SNAPSHOT='$REMOTE_SNAPSHOT' bash -s" <<'REMOTE'
set -euo pipefail
cat >"$REMOTE_CONFIG" <<EOF
[sources]
default_scope = "all"
query_timeout_ms = 5000

[sources.workstation]
kind = "snapshot"
label = "Workstation"
path = "$REMOTE_SNAPSHOT"
enabled = true
EOF

"$REMOTE_FOREMAN" --config-file "$REMOTE_CONFIG" --log-dir "$REMOTE_LOG_DIR" sources doctor workstation --json \
  | jq '{ok, source, diagnostics}'
"$REMOTE_FOREMAN" --config-file "$REMOTE_CONFIG" --log-dir "$REMOTE_LOG_DIR" agents --sources all --json \
  > /tmp/foreman-remote-with-workstation-snapshot.json
jq '{entryCount:(.entries|length), by_source:(.entries|group_by(.sourceId)|map({source:.[0].sourceId,count:length})), diagnostics:.sourceDiagnostics}' \
  /tmp/foreman-remote-with-workstation-snapshot.json
jq -e 'any(.entries[]; .sourceId == "workstation")' /tmp/foreman-remote-with-workstation-snapshot.json >/dev/null
jq -e 'any(.entries[]; .sourceId == "local")' /tmp/foreman-remote-with-workstation-snapshot.json >/dev/null
REMOTE
