---
id: providers-harness-kit
title: Harness Kit Provider
description: Install and operate the read-only Foreman extension provider for Harness Kit lifecycle state.
index:
  - id: overview
  - id: install
  - id: provider-contract
  - id: precedence
---

# Harness Kit Provider

Foreman extension providers are small read-only commands that turn repo or
workspace state into cards on the control API. The Harness Kit provider displays
HK lifecycle readiness for the selected pane's repo without making Foreman own
HK state.

## Overview

The boundary is intentional:

```text
HK       = lifecycle ledger, validation evidence, review records, handoff export
Foreman  = operator console that displays and routes work state
Provider = adapter that maps `hk brief/status --json` into Foreman cards
```

Foreman runs providers only when the control API is called with extensions:

```bash
foreman agents --json --extensions
```

The macOS overlay enables this by default and renders returned cards above the
selected pane's recent output. The terminal dashboard also fetches cards for the
currently selected workspace and renders them in the Details pane, avoiding
provider work for every visible row. HK cards use operator-facing labels such as
`NEEDS VALIDATION`, `NEEDS REVIEW`, `NEEDS SYNC`, `READY`, and `NO WORK` so the
first line points at the next lifecycle step instead of only saying whether the
slice is ready. The provider consumes HK's read-only `handoff_export` metadata
from `hk brief --json`, so Foreman can open an existing handoff README or copy
HK's explicit export/preview commands without guessing paths or writing repo
files. `NO WORK` cards stay intentionally minimal: they summarize that no HK
slice is active and offer a start command instead of rendering noisy `none` /
`not-started` rows.

## Install

The preferred install is user-level so the same HK card works in every repo:

```text
~/.config/foreman/providers/hk.toml
~/.config/foreman/providers/hk-provider.py
```

Foreman ships a copyable example:

```bash
mkdir -p ~/.config/foreman/providers
cp examples/providers/harness-kit/hk.toml ~/.config/foreman/providers/hk.toml
cp examples/providers/harness-kit/hk-provider.py ~/.config/foreman/providers/hk-provider.py
chmod +x ~/.config/foreman/providers/hk-provider.py
```


Use repo-local providers only for repo-specific behavior:

```text
<repo>/.foreman/providers/hk.toml
<repo>/.foreman/providers/hk-provider.py
```

## Provider Contract

A provider manifest is TOML:

```toml
id = "hk"
title = "Harness Kit"
scope = "repo"
command = "./hk-provider.py"
args = ["--repo", "{repo}"]
timeout-ms = 30000
```

Foreman runs the command from the manifest directory. It expands these tokens in
`command` and `args`:

| Token | Value |
|---|---|
| `{repo}` | discovered git repository root for the pane working directory |
| `{workspace}` | pane working directory |

Foreman also writes provider context JSON to stdin:

```json
{
  "schemaVersion": 1,
  "workspacePath": "/path/to/pane/cwd",
  "repositoryRoot": "/path/to/repo"
}
```

The command returns cards. This abbreviated example shows the most important
fields first; the provider may include additional rows and lower-priority copy
actions after these.

```json
{
  "cards": [
    {
      "id": "hk",
      "title": "Harness Kit",
      "status": "needs-validation",
      "statusLabel": "NEEDS VALIDATION",
      "summary": "Next: hk validate --check fast-gate --why 'Fast gate passes' --target /repo -- mise run check",
      "rows": [
        { "label": "Work", "value": "foreman-hk-provider-dots", "status": "info" },
        { "label": "Phase", "value": "finalizing", "status": "info" },
        { "label": "Export", "value": "missing", "status": "info" },
        { "label": "Validation", "value": "needs fast-gate", "status": "fail" }
      ],
      "actions": [
        { "id": "copy-next", "label": "Copy next", "kind": "copy", "value": "hk validate --check fast-gate --why 'Fast gate passes' --target /repo -- mise run check" },
        { "id": "copy-export", "label": "Copy export", "kind": "copy", "value": "hk export --format handoff-dir --output /repo/.ai/hk/2026-... --target /repo" },
        { "id": "copy-handoff-preview", "label": "Copy handoff preview", "kind": "copy", "value": "hk handoff --target /repo --json" },
        { "id": "open-evidence", "label": "Open evidence", "kind": "open-file", "value": "/repo/.harness-local/.../ev_....transcript.log" }
      ]
    }
  ]
}
```

Supported action kinds are intentionally safe:

| Kind | Behavior |
|---|---|
| `copy` | copy `value` to the clipboard |
| `open-file` | open `value` as a local file URL |
| `open-url` | open `value` as a URL |

The HK provider should stay read-only. It may copy commands such as
`hk validate ...` or `hk export ...`, but Foreman should not silently mutate HK
state. `Open handoff` appears only when HK reports an existing handoff README;
stale exports may also show `Open stale handoff` so operators can inspect the
current committed projection, while missing/stale exports surface copyable
`hk export` / `hk handoff --json` commands for explicit refresh or live preview.

## Precedence

Provider discovery order is low-to-high priority:

1. `~/.config/foreman/providers`
2. `<repo>/.foreman/providers`
3. `FOREMAN_EXTENSION_PROVIDER_DIRS`

If multiple manifests use the same `id`, the higher-priority provider wins. This
lets a normal user-level HK provider work everywhere, a repo override customize a
specific checkout, and an environment override support tests or experiments.
