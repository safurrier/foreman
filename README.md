# foreman

Foreman is a terminal console for supervising AI agents that are running in
tmux.

Current crate version: `1.2.0`

It gives one operator view over Claude Code, Codex CLI, Pi, Gemini CLI, and
OpenCode panes. The dashboard groups tmux sessions, shows which panes are
stable or need attention, lets you jump directly to the right pane, and can
wire native status hooks for the harnesses that support them.

## Why Use It

- See all agent sessions and panes in one TUI.
- Prefer native hook signals for Claude Code, Codex, and Pi when available.
- Fall back to lower-confidence tmux compatibility detection for unsupported
  or unwired panes.
- Jump tmux to the selected pane, compose input, search, filter, and inspect
  status provenance without leaving the dashboard.
- Use setup and doctor commands to install hooks, check wiring, and diagnose
  fallback-heavy sessions.
- Send desktop notifications when work completes or an agent needs attention.

## Quick Start

Requirements:

- `tmux`
- Rust toolchain with `cargo`
- `mise` for repo tasks

Install the local binaries:

```bash
mise run setup
mise run install-local
```

Wire the current repo and user-level harness config, then check the result:

```bash
foreman --setup --user --project
foreman --doctor
foreman
```

To try Foreman from the checkout without installing:

```bash
mise run dev
```

The install task provides:

- `foreman`
- `foreman-claude-hook`
- `foreman-codex-hook`
- `foreman-pi-hook`

## Dashboard Basics

Common keys:

| Key | Action |
|---|---|
| `j` / `k` | Move through the tree |
| `Tab` or `1` / `2` / `3` | Switch panel focus |
| `i` | Compose input for the actionable agent row |
| `f` | Focus tmux on the selected actionable pane |
| `Enter` | Send in compose mode, or act on the selected row |
| `/` | Search |
| `o` | Cycle `stable` and `attention->recent` sort modes |
| `s` / `S` | Start flash jump, optionally focusing tmux |
| `h` | Cycle visible harness families |
| `H` / `P` | Reveal non-agent sessions or panes |
| `t` | Cycle themes |
| `?` | Open help and status legend |

Status labels include their source. `native hook` means Foreman read a
structured harness signal. `compatibility heuristic` means Foreman inferred
state from tmux-visible behavior and treats it as lower confidence.

## Native Harness Support

| Harness | Compatibility mode | Native mode |
|---|---:|---:|
| Claude Code | yes | yes |
| Codex CLI | yes | yes |
| Pi | yes | yes |
| Gemini CLI | yes | no |
| OpenCode | yes | no |

Run setup after installing Foreman to wire native hooks:

```bash
foreman --setup --user --project
```

If an existing pane was started before hook wiring changed, restart that agent
pane. Setup updates files; it does not repair already-running processes.

See [Operator Guide](docs/operator-guide.md) for setup scopes, doctor fixes,
native hook examples, notification config, UI preferences, and troubleshooting.

## Notifications

Foreman can notify on completion and attention states through configured desktop
backends. On macOS, `alerter` is the preferred backend because click actions can
focus the related tmux pane. Custom macOS notification sounds can use
`notification-sounds:<prefix>` so playback stays on the notification path
instead of direct `afplay` audio.

See [Operator Guide](docs/operator-guide.md#notifications) for the full config.

## Demo Source

![Foreman quickstart demo](demos/readme-quickstart.gif)

The README demo source lives at:

- [`demos/readme-quickstart.tape`](demos/readme-quickstart.tape)

Render it with VHS when refreshing product media:

```bash
vhs demos/readme-quickstart.tape
```

Generated GIFs should be refreshed from the tape and checked before commit.

## Docs

- [Operator Guide](docs/operator-guide.md) - install, setup, dashboard, config,
  hooks, notifications, and troubleshooting
- [Repo Tour](docs/tour.md) - first read for contributors
- [Workflow Guide](docs/workflows.md) - plan workflow, validation ladder, and
  release process
- [Architecture](docs/architecture.md) - system boundaries and module map
- [Changelog](CHANGELOG.md) - release history

## Development

```bash
git checkout -b feat/<slug>
mise run plan -- <slug>
mise run check
```

Useful tasks:

| Command | Purpose |
|---|---|
| `mise run setup` | Install dependencies and prepare the environment |
| `mise run fmt` | Auto-format code |
| `mise run lint` | Run lint checks |
| `mise run typecheck` | Run static type analysis |
| `mise run test` | Run Rust tests |
| `mise run build` | Build release binaries |
| `mise run check` | Fast quality gate |
| `mise run verify` | Heavy validation |
| `mise run verify-release` | Release-confidence operator gauntlet |
| `mise run native-preflight` | Check local real-harness readiness |
| `mise run verify-native` | Real Claude, Codex, and Pi E2E drill |
| `mise run verify-ux` | TUI runtime smoke and UX artifact refresh |

CI calls `mise run ci`, which maps to the fast check gate.
