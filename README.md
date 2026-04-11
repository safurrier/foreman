# foreman

TUI for managing AI agents across tmux

Current crate version: `1.0.0`

## Quick Start

```bash
# Fetch Rust dependencies and hooks
mise run setup

# Run the fast validation gate
mise run check

# Run the UX-focused validation lane and refresh screenshots/GIFs
mise run verify-ux

# Run the release-confidence operator gauntlet
mise run verify-release

# Run foreman from source
mise run dev
```

## Install Locally

Requirements:
- `tmux`
- Rust toolchain with `cargo`

Install the binaries from this checkout:

```bash
mise run install-local
```

That installs:
- `foreman`
- `foreman-claude-hook`
- `foreman-codex-hook`
- `foreman-pi-hook`

First-run setup:

```bash
foreman --init-config
foreman --config-path
foreman --help
```

If you want to try it without installing into `~/.cargo/bin`:

```bash
cargo run -- --help
mise run dev
```

To install into a temporary root for testing:

```bash
mise run install-local -- --root /tmp/foreman-install
```

Minimal tmux smoke run:

```bash
tmux new-session -d -s alpha "sh -lc \"printf '%s\n' 'Claude Code ready'; exec sleep 600\""
foreman
```

Common dashboard keys:
- `j` / `k` move through the tree
- `Tab` or `1` / `2` / `3` switches panel focus
- `i` composes for the actionable agent row
- `f` jumps tmux to the actionable pane
- `Enter` sends in compose mode and acts on the selected row in normal mode
- `/` starts search
- `s` starts flash jump
- `h` cycles only the visible harness families, then returns to `all`
- `H` / `P` reveal non-agent sessions or panes
- `t` cycles the active theme
- `?` opens help, the legend, and the harness mark map

Theme selection:

```toml
[ui]
theme = "catppuccin" # catppuccin | gruvbox | tokyo-night | nord | dracula | terminal | no-color
```

You can also cycle themes live with `t` in normal mode.

Contributor docs:
- [`docs/tour.md`](docs/tour.md) — repo quickstart and reading order
- [`docs/workflows.md`](docs/workflows.md) — durable workflow and validation notes

## Current Status

- `foreman --config-path` and `foreman --init-config` are live.
- `foreman --debug` enables debug-level run logging without changing the normal
  interactive startup path.
- Normal startup bootstraps config, logging, tmux inventory, native Claude,
  Codex, and Pi overlays, Gemini CLI and OpenCode compatibility detection, and
  header-level system stats.
- Config now controls notification cooldowns, backend order, startup profile,
  and per-harness native-vs-compatibility preference.
- `foreman-claude-hook` bridges official Claude Code hook events into Foreman's
  per-pane native status files, with a default state path and optional
  config-level override.
- `foreman-codex-hook` bridges official Codex hook events into the same
  per-pane native status model, with its own default state path and optional
  config-level override.
- `foreman-pi-hook` bridges Pi extension lifecycle events into that same
  per-pane native status model, with its own default state path and optional
  config-level override.
- Normal startup now enters the interactive dashboard loop with live tmux
  polling, direct input, popup focus-close behavior, and binary-level tmux E2E
  coverage.
- The dashboard now supports built-in TUI palettes (`catppuccin`, `gruvbox`,
  `tokyo-night`, `nord`, `dracula`, `terminal`) plus a `no-color` fallback,
  with config-based default selection and runtime cycling via `t`.
- The dashboard now uses compact harness marks plus an in-product legend, and
  supports cycling a harness-only view with `h` without losing actionable
  session/window behavior or landing on empty harness views by default.

## Harness Support Matrix

| Harness | Compatibility mode | Native mode |
|---------|--------------------|-------------|
| Claude Code | yes | yes |
| Codex CLI | yes | yes |
| Pi | yes | yes |
| Gemini CLI | yes | no |
| OpenCode | yes | no |

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

## Codex Native Hooks

Foreman also ships `foreman-codex-hook` for Codex native status integration.
The bridge reads official Codex hook JSON on stdin, resolves the pane from
`TMUX_PANE` or `--pane-id`, and writes the per-pane signal file that Foreman
overlays in native mode.

By default the bridge writes next to Foreman's run logs under `codex-native`.
You can override that path with `[integrations.codex_cli].native_dir` in
`config.toml`.

Example repo-local `.codex/hooks.json`:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-codex-hook"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "foreman-codex-hook"
          }
        ]
      }
    ]
  }
}
```

## Pi Native Extension Bridge

Foreman also ships `foreman-pi-hook` for Pi native status integration. Pi does
not use the same external hook JSON surface as Claude Code or Codex CLI here;
instead, a small Pi extension calls the bridge on Pi lifecycle events and the
bridge writes the per-pane signal file that Foreman overlays in native mode.

By default the bridge writes next to Foreman's run logs under `pi-native`. You
can override that path with `[integrations.pi].native_dir` in `config.toml`.

Example project-local `.pi/extensions/foreman.ts`:

```typescript
import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { spawnSync } from "node:child_process";

export default function (pi: ExtensionAPI) {
  const runHook = (event: string) => {
    const args = ["--event", event];
    if (process.env.TMUX_PANE) {
      args.push("--pane-id", process.env.TMUX_PANE);
    }
    spawnSync("foreman-pi-hook", args, { stdio: "inherit" });
  };

  pi.on("agent_start", async () => runHook("agent-start"));
  pi.on("agent_end", async () => runHook("agent-end"));
  pi.on("session_shutdown", async () => runHook("session-shutdown"));
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
| `mise run build` | Build release binaries |
| `mise run install-local` | Install `foreman` and the hook binaries locally |
| `mise run check` | Fast quality gate (fmt + lint + typecheck + test) |
| `mise run dev` | Run `foreman` from source |
| `mise run ci` | CI entrypoint (= check) |
| `mise run plan -- <slug>` | Create a plan directory for a unit of work |
| `mise run verify` | Heavy validation (integration, docker, security) |
| `mise run verify-release` | Release-confidence compiled-binary tmux gauntlet plus checklist/report artifact |
| `mise run verify-native` | Opt-in real Claude, Codex, and Pi E2E drill when the corresponding env flags are set |
| `mise run verify-ux` | Focused TUI/runtime smoke, navigation perf smoke, plus VHS artifact refresh |

## GitHub Actions

- `.github/workflows/ci.yml` runs the fast gate on pushes to `main` and on pull
  requests targeting `main`, and runs `mise run verify` on pull requests.
- `.github/workflows/release.yml` runs on `v*` tags, verifies the repo,
  builds release bundles for Linux x86_64, macOS x86_64, and macOS aarch64,
  and publishes a GitHub release.

## Release Process

1. Bump `Cargo.toml` to the intended release version.
2. Merge the release branch to `main`.
3. Run `mise run verify-release` if you want the release-confidence report locally before tagging.
4. If you have local auth for the external harnesses, run `mise run verify-native`
   with the relevant env flags set for a real-binary drill.
5. Push a matching annotated tag such as `v1.0.0`.
6. GitHub Actions verifies the repo and publishes release bundles.

Each release archive contains:
- `foreman`
- `foreman-claude-hook`
- `foreman-codex-hook`
- `foreman-pi-hook`
- `README.md`

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
