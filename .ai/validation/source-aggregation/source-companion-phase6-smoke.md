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

## Reverse tunnel finding

The harness also attempted `reverse-actions` with OpenSSH `-R` from Mac to
Coder. The Coder SSH proxy accepted the remote-forward request in verbose SSH
logs, but the forwarded port was not reachable from inside the Coder workspace
(`Connection refused`). This means the product code now has a companion live
action transport and validation harness, but the current Coder proxy path still
needs a working tunnel/port-forward mechanism before the live Coder→Mac proof can
be marked complete.
