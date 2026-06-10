# Source companion phase 2-6 smoke evidence

Branch: `feature/source-companion-prewarmer`
PR: #27

## Passing checks

The live smoke harness uses an isolated tmux socket/server for local workstation
panes so validation does not touch the operator's default tmux workspace.

```bash
cargo test --lib --quiet
# 339 passed

scripts/source_companion_live_smoke.py companion-local \
  --artifact-dir .ai/validation/source-companion/companion-local \
  --json --no-build
```

Representative result after rebuilding the candidate binary:

```json
{
  "ok": true,
  "scenario": "companion-local",
  "result": {
    "counts": {"local": 1, "workstation": 1},
    "entryCount": 2
  }
}
```

```bash
scripts/source_companion_live_smoke.py remote-snapshot \
  --artifact-dir .ai/validation/source-companion/remote-snapshot \
  --install-remote \
  --json --no-build
```

Representative result:

```json
{
  "ok": true,
  "scenario": "remote-snapshot",
  "result": {
    "entryCount": 6,
    "bySource": {"local": 5, "workstation": 1},
    "diagnostics": []
  }
}
```

```bash
scripts/source_companion_live_smoke.py reverse-actions \
  --artifact-dir .ai/validation/source-companion/reverse-actions \
  --install-remote \
  --json --no-build
```

Representative result:

```json
{
  "ok": true,
  "scenario": "reverse-actions",
  "result": {
    "remoteEndpoint": "127.0.0.1:<remote-port>",
    "paneId": "%0",
    "counts": {"local": 5, "workstation": 1},
    "focusOk": true,
    "sendOk": true
  }
}
```

```bash
scripts/source_companion_live_smoke.py connect-ssh \
  --artifact-dir .ai/validation/source-companion/connect-ssh \
  --remote-host "$FOREMAN_REMOTE_DEV_HOST" \
  --install-remote \
  --json --no-build
```

Representative result:

```json
{
  "ok": true,
  "scenario": "connect-ssh",
  "result": {
    "remoteHostLabel": "remote-dev",
    "remoteEndpoint": "127.0.0.1:<remote-port>",
    "paneId": "%0",
    "counts": {"local": 5, "workstation": 1},
    "focusOk": true,
    "sendOk": true,
    "activationOk": true
  }
}
```

## Reverse tunnel finding

Initial `reverse-actions` attempts looked like a remote SSH forwarding problem
because `ssh -R` reported success while a later Foreman request saw `Connection
refused`. The root cause was the harness readiness probe: it opened and closed
the companion port without sending a JSON-line request, which could consume or
stall the single-threaded companion server's first request. The fixed harness
probes with a valid companion request before running Foreman through the reverse
tunnel. With that fix, live remote-host → workstation inventory, focus, trusted
send, and source-local activation all passed.
