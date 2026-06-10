# Source companion phase 2-6 smoke evidence

Branch: `feature/source-companion-prewarmer`
PR: #27

## Passing checks

```bash
cargo test --lib --quiet
# 336 passed

scripts/source_companion_live_smoke.py companion-local \
  --artifact-dir .ai/validation/source-companion/companion-local \
  --json --no-build
```

Result after rebuilding the candidate binary:

```json
{
  "ok": true,
  "scenario": "companion-local",
  "result": {
    "counts": {"local": 30, "mac-live": 30},
    "entryCount": 60
  }
}
```

```bash
scripts/source_companion_live_smoke.py coder-snapshot \
  --artifact-dir .ai/validation/source-companion/coder-snapshot \
  --json --no-build
```

Result:

```json
{
  "ok": true,
  "scenario": "coder-snapshot",
  "result": {
    "entryCount": 35,
    "bySource": {"local": 5, "mac": 30},
    "diagnostics": []
  }
}
```

```bash
scripts/source_companion_live_smoke.py reverse-actions \
  --artifact-dir .ai/validation/source-companion/reverse-actions \
  --install-coder \
  --json --no-build
```

Result:

```json
{
  "ok": true,
  "scenario": "reverse-actions",
  "result": {
    "remoteEndpoint": "127.0.0.1:46631",
    "paneId": "%71",
    "counts": {"local": 5, "mac-live": 31},
    "focusOk": true,
    "sendOk": true
  }
}
```

## Reverse tunnel finding

Initial `reverse-actions` attempts looked like a Coder SSH proxy problem because
`ssh -R` reported success while a later Foreman request saw `Connection refused`.
The root cause was the harness readiness probe: it opened and closed the
companion port without sending a JSON-line request, which could consume/stall the
single-threaded companion server's first request. The fixed harness probes with a
valid companion request before running Foreman through the reverse tunnel. With
that fix, live Coder → Mac inventory, focus, and trusted send all passed.
