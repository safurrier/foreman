# foreman

TUI for managing AI agents across tmux

Current crate version: `1.1.0`

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
foreman --setup --user --project
foreman --doctor
foreman
```

Setup model:
- `foreman --setup` is meant to be safe to rerun.
- `--user` writes home-directory wiring under `~/.claude`, `~/.codex`, and `~/.pi`.
- `--project` writes repo-local wiring for the current checkout, or the repo passed with `--repo`.
- Without explicit scope flags, `foreman --setup` defaults to `project` when the current directory looks like a repo, and `user` otherwise.
- Setup only targets one repo at a time. Run it again for other checkouts:
  `foreman --setup --project --repo /path/to/repo`
- Provider flags let you limit writes to one integration:
  `foreman --setup --project --codex --repo /path/to/repo`
- Setup fixes files and directories. It does not retroactively repair already-running panes; restart those agents after wiring changes.

If the dashboard is falling back to `compatibility heuristic` more than you
expect, run doctor before debugging tmux by hand:

```bash
foreman --doctor
foreman --setup --user --project --dry-run
foreman --setup --user --project
```

Doctor checks:
- local config and log paths
- hook binaries and provider CLIs on `PATH`
- user-level and project-level hook wiring for Claude, Codex, and Pi
- live tmux/runtime fallback signals when a tmux server is available

Safe fixes today:
- initialize default `config.toml`
- create the default native signal directories under `~/.local/state/foreman/`
- merge Claude hook wiring into `.claude/settings.local.json` or `~/.claude/settings.local.json`
- scaffold or merge `.codex/hooks.json` or `~/.codex/hooks.json`
- scaffold `.pi/extensions/foreman.ts` or `~/.pi/extensions/foreman.ts`

Use `--repo` when you want to target another checkout:

```bash
foreman --setup --project --repo /path/to/repo
foreman --setup --project --codex --repo /path/to/repo
foreman --setup --user --claude
foreman --doctor --repo /path/to/repo
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
- `o` cycles `stable` and `attention->recent`
- `s` starts flash jump with inline labels; `S` also focuses tmux after the jump
- `h` cycles only the visible harness families, then returns to `all`
- `H` / `P` reveal non-agent sessions or panes
- `t` cycles the active theme
- `?` opens the scrollable help/legend surface; use `j` / `k` or `PageUp` / `PageDown` inside it

Preview and help also surface where a status came from:
- `native hook` means a structured harness signal and is treated as higher confidence
- `compatibility heuristic` means tmux-observed behavior and is treated as lower confidence
- the help overlay starts with a legend for tree, status, and harness glyphs so the sidebar icons are decodable without guesswork
- selected panes can now surface compact setup hints and point back to
  `foreman --setup --user --project` or `foreman --doctor` when compatibility fallback looks suspicious

If `j` / `k` still feels slow locally, run with debug logging and inspect the latest run log:

```bash
foreman --debug
tail -f ~/.local/state/foreman/logs/latest.log
```

Useful timing lines:
- `timing operation=action action=move-selection ...`
- `timing operation=render_frame ...`
- `timing operation=inventory_tmux ...`
- `timing operation=inventory_native ...`
- `timing operation=startup_cache_load ...`
- `timing operation=startup_cache_write ...`

`inventory_tmux` now also shows how many panes were freshly captured vs reused
from cached previews on that refresh, which is the first thing to check if
local lag still feels tied to tmux polling. Startup cache lines include the cache
path and age so popup startup behavior is easier to debug. `foreman --doctor` also
reports the popup cache path, freshness window, age, and last inventory counts.

UI preferences:

```toml
[monitoring]
startup_cache_max_age_ms = 300000

[ui]
theme = "catppuccin" # catppuccin | gruvbox | tokyo-night | nord | dracula | terminal | no-color
default_sort = "stable" # stable | attention-recent
```

Foreman also persists runtime UI choices such as theme, sort mode, filters, collapsed sessions, and the last selected target in `ui-state.json` next to the resolved config file. Explicit `[ui].theme` and `[ui].default_sort` config values win over persisted values; persisted values fill in only when those keys are omitted. You can cycle themes live with `t` and sort modes live with `o` in normal mode. Use `foreman --reset-ui-state` to remove persisted UI choices.

Contributor docs:
- [`docs/tour.md`](docs/tour.md) — repo quickstart and reading order
- [`docs/workflows.md`](docs/workflows.md) — durable workflow and validation notes

## Current Status

- `foreman --config-path` and `foreman --init-config` are live.
- `foreman --debug` enables debug-level run logging without changing the normal
  interactive startup path.
- Normal startup paints immediately with a loading state, then fills tmux
  inventory asynchronously and prioritizes visible/selected preview capture on
  the next refresh.
- Popup startup can also render from a fresh tmux-socket-scoped startup cache,
  clearly marked as `cached ... ago`, before live tmux refresh replaces it.
- Interactive startup still resolves native Claude, Codex, and Pi overlays,
  Gemini CLI and OpenCode compatibility detection, and header-level system
  stats once the first inventory refresh lands.
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
| `foreman --setup --user --project` | Initialize config and wire user-level plus project-level Claude, Codex, and Pi integration |
| `foreman --config-show` | Print resolved config, UI state, popup cache, and runtime values |
| `foreman --doctor` | Diagnose local install, hook wiring, popup cache, UI state, and live runtime fallback from the current repo |
| `foreman --doctor --doctor-fix --doctor-dry-run` | Preview safe doctor repairs before writing setup/config files |
| `foreman --doctor --doctor-fix` | Apply safe doctor repairs such as missing config, native dirs, and provider wiring |
| `foreman --reset-ui-state` | Remove persisted sort/theme/filter/collapse/selection choices for the resolved config |
| `mise run ci` | CI entrypoint (= check) |
| `mise run plan -- <slug>` | Create a plan directory for a unit of work |
| `mise run verify` | Heavy validation (integration, docker, security) |
| `mise run verify-release` | Release-confidence compiled-binary tmux gauntlet plus checklist/report artifact under `.ai/validation/release/` |
| `mise run native-preflight` | Check tmux plus local Claude/Codex/Pi readiness before the real native E2E lane |
| `mise run verify-native` | Real Claude, Codex, and Pi E2E drill; strict mode requires all env flags and actual execution |
| `mise run verify-ux` | Focused TUI/runtime smoke, deterministic native-dashboard/focus checks, navigation perf smoke, plus VHS artifact refresh under `.ai/validation/ux/` |

## GitHub Actions

- `.github/workflows/ci.yml` runs the fast gate on pushes to `main` and on pull
  requests targeting `main`, and runs `mise run verify` on pull requests.
- `.github/workflows/release.yml` runs on `v*` tags, verifies the repo,
  builds release bundles for Linux x86_64, macOS x86_64, and macOS aarch64,
  and publishes a GitHub release.

## Release Process

1. Bump `Cargo.toml` to the intended release version and add a matching
   `CHANGELOG.md` entry.
2. Merge the release branch to `main`.
3. Run `mise run verify-release` if you want the release-confidence report locally before tagging.
4. If the release touches native harness behavior, run `mise run native-preflight`
   first, then strict `mise run verify-native`.
5. Push a matching annotated tag such as `v1.1.0`.
6. GitHub Actions verifies the repo and publishes release bundles.

Validation evidence now lands in a stable root:

- `.ai/validation/ux/`
- `.ai/validation/release/`

Strict native verification:

```bash
mise run native-preflight
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

If `codex --version` and `npm list -g @openai/codex --depth=0` disagree, fix the
PATH-resolved install first. Foreman's real Codex E2E depends on Codex hook
support, which requires a current Codex CLI on PATH.

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
