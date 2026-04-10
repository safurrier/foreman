# foreman

TUI for managing AI agents across tmux

## Quick Start

```bash
# Install dependencies
mise run setup

# Run quality checks
mise run check

# Start development
mise run dev
```

## Current Status

- `foreman --config-path` and `foreman --init-config` are live.
- `foreman --debug` enables debug-level run logging without changing the normal
  interactive startup path.
- Normal startup bootstraps config, logging, tmux inventory, native Claude
  overlays, and header-level system stats.
- Config now controls notification cooldowns, backend order, startup profile,
  and Claude native-vs-compatibility preference.
- `foreman-claude-hook` bridges official Claude Code hook events into Foreman's
  per-pane native status files, with a default state path and optional
  config-level override.
- Normal startup now enters the interactive dashboard loop with live tmux
  polling, direct input, popup focus-close behavior, and binary-level tmux E2E
  coverage.

## Claude Code Native Hooks

Foreman now ships a small companion binary, `foreman-claude-hook`, for Claude
Code native status integration. The bridge reads official Claude hook JSON on
stdin, uses `TMUX_PANE` to identify the pane, and writes the per-pane signal
file that Foreman overlays in native mode.

By default the bridge writes to the state-path sibling of the run logs. You can
override that path with `[integrations.claude_code].native_dir` in
`config.toml`.

Example `.claude/settings.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-claude-hook"
          }
        ]
      }
    ],
    "Notification": [
      {
        "matcher": "permission_prompt|elicitation_dialog",
        "hooks": [
          {
            "type": "command",
            "command": "foreman-claude-hook"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-claude-hook"
          }
        ]
      }
    ],
    "StopFailure": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-claude-hook"
          }
        ]
      }
    ]
  }
}
```

## Starting New Work

```bash
git checkout -b feat/<slug>
mise run plan -- <slug>
```

## Task Reference

| Command | Purpose |
|---------|---------|
| `mise run setup` | Install dependencies and prepare the environment |
| `mise run fmt` | Auto-format code |
| `mise run lint` | Run lint checks (non-modifying) |
| `mise run typecheck` | Run static type analysis |
| `mise run test` | Run Rust unit, integration, and smoke tests |
| `mise run build` | Build artifacts |
| `mise run check` | Fast quality gate (fmt + lint + typecheck + test) |
| `mise run dev` | Start local development |
| `mise run ci` | CI entrypoint (= check) |
| `mise run plan -- <slug>` | Create a plan directory for a unit of work |
| `mise run verify` | Heavy validation (integration, docker, security) |

## Project Structure

```
foreman/
├── src/                    # Source code
│   ├── main.rs             # Entry point
│   └── lib.rs              # Library with examples
├── .mise.toml              # Task runner config
├── Cargo.toml              # Package manifest
├── rustfmt.toml            # Formatter config
├── Dockerfile              # Multi-stage build
└── README.md
```

## Development

This project uses [mise](https://mise.jdx.dev/) as the task runner. All quality gates
are accessible via `mise run <task>`.

CI calls a single entrypoint: `mise run ci`.

If your local Docker context points at Colima, `mise run verify` expects the
active Colima profile to be running before the Docker build step starts.
