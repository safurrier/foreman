# Source snapshot remote-host smoke

Branch: `feature/source-companion-prewarmer`
PR: #27

## Command

```bash
scripts/smoke-source-snapshot-remote.sh
```

The script is a compatibility wrapper for the read-only snapshot path:

- write a workstation source snapshot with `foreman sources snapshot --source-id workstation`
- copy the snapshot to a configured remote SSH host
- use a temporary remote config containing a `snapshot` source
- verify the remote Foreman can render both remote-local rows and workstation snapshot rows

For newer live transport coverage, prefer:

```bash
scripts/source_companion_live_smoke.py remote-snapshot --install-remote --json
scripts/source_companion_live_smoke.py connect-ssh --install-remote --json
```

## Representative result

```json
{
  "ok": true,
  "sourceId": "workstation",
  "entryCount": 30,
  "path": "/tmp/foreman-workstation-source-snapshot.json"
}
```

Remote agent response summary:

```json
{
  "entryCount": 35,
  "by_source": [
    {"source": "local", "count": 5},
    {"source": "workstation", "count": 30}
  ],
  "diagnostics": []
}
```
