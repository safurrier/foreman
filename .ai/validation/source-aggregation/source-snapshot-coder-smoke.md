# Source snapshot Coder smoke

Date: 2026-06-09

Command:

```bash
scripts/smoke-source-snapshot-coder.sh
```

Purpose:

- Build the local Foreman candidate.
- Write a Mac source snapshot with `foreman sources snapshot --source-id mac`.
- Copy the snapshot to the configured Coder host.
- Use the installed Coder Foreman candidate with a temporary config containing a read-only `snapshot` source.
- Verify Coder Foreman can render both Coder-local rows and Mac snapshot rows.

Output:

```json
{
  "ok": true,
  "sourceId": "mac",
  "entryCount": 10,
  "path": "/var/folders/kf/js4h91w14pl7zwfgnvj896b00000gq/T/foreman-mac-source-snapshot.teP5oR4qg9.json"
}
{
  "ok": true,
  "source": {
    "displayLabel": "Mac",
    "enabled": true,
    "id": "mac",
    "kind": "snapshot",
    "label": "Mac",
    "showLabel": true
  },
  "diagnostics": []
}
{
  "entryCount": 11,
  "by_source": [
    {
      "source": "local",
      "count": 1
    },
    {
      "source": "mac",
      "count": 10
    }
  ],
  "diagnostics": []
}
```

Result: pass.
