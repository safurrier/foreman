# Foreman

<p align="center">
  <img src="foreman-logo.png" alt="Foreman logo" width="180">
</p>

Foreman is a terminal console and native macOS control app for supervising AI
coding agents that run in tmux.

It gives one operator view over Claude Code, Codex CLI, Pi, Gemini CLI, and
OpenCode panes. The dashboard groups tmux sessions, shows which panes are stable
or need attention, lets you jump directly to the right pane, and can wire native
status hooks for the harnesses that support them. The optional native macOS app
packages that same control plane as `Foreman.app` for a global hotkey,
Spotlight/Raycast launch, quick search, preview, compose/send, and pane focus
from outside the terminal.

Current crate version: `1.4.0`

Use Foreman when you have more than one agent session running and need a control
surface. If you only need to inspect an ordinary tmux tree, tmux itself is the
better tool.

## Pick your path

| Path | Use it when | First useful command |
|---|---|---|
| Terminal dashboard | You live in tmux and want the full Ratatui console | `foreman` |
| macOS app | You want a global hotkey, Spotlight/Raycast launch, search, preview, and pane focus outside the terminal | `open -a Foreman` |
| Control API | You want scripts or native clients to read Foreman's tmux inventory | `foreman agents --json` |

Foreman is built from this source checkout today. Release artifacts are published
from version tags; do not assume a package registry install unless a release note
or local workflow verifies it.

## Why operators use it

- One dashboard for agent panes instead of spelunking through tmux windows.
- Native hook signals for Claude Code, Codex, and Pi when they are wired;
  lower-confidence compatibility detection when they are not.
- Fast operator actions: focus a pane, compose input, search/filter, inspect
  status provenance, and get desktop notifications when work finishes or needs
  attention.
- Optional PR cards, linked repositories, and extension-provider cards for
  adjacent context without opening each repo by hand.

## Quick start: terminal dashboard

Requirements:

- `tmux`
- Rust toolchain with `cargo`
- `mise` for repo tasks

Install the local CLI binaries:

```bash
mise run setup
mise run install-local
```

That installs:

- `foreman`
- `foreman-claude-hook`
- `foreman-codex-hook`
- `foreman-pi-hook`

Wire user-level and current-repo harness integration files, then inspect the
result:

```bash
foreman --setup --user --project
foreman --doctor
foreman
```

Success signal: `foreman --setup` ends with `Next` steps, `foreman --doctor`
prints Machine/Config/Repo/Runtime findings, and a ready setup has no `ERROR`
lines. `WARN` lines are still useful: they tell you which panes are running in
fallback mode or which hook wiring needs a restart. `foreman` should open the
operator dashboard; press `?` there for the key map and status legend.

`foreman --setup` is safe to rerun. It writes hook/config files, but it does not
repair already-running agent panes. Restart affected panes after changing hook
wiring.

To try the dashboard from the checkout without installing:

```bash
mise run dev
```

## Native macOS app

The macOS app is a Swift/AppKit/SwiftUI client for Foreman's Rust control API.
It does not reimplement tmux discovery; it calls `foreman agents --json`,
`foreman focus --pane ... --json`, and `foreman send --pane ... --json`.

Use the app for:

- global hotkey access to Foreman from anywhere
- type-to-search over agent panes
- attention/recent sorting and filters
- detail preview and pull request cards
- compose/send to a selected pane
- double-click or Enter/Focus to jump the terminal to a pane

Local install/reset:

```bash
mise run install-macos-overlay-app
open -a Foreman
```

The install task builds `apps/macos-overlay`, installs
`~/Applications/Foreman.app`, and removes stale local development bundles so
macOS launchers do not open an old app.

Run this after changing overlay code and before manual Spotlight/Raycast testing:

```bash
mise run validate-macos-overlay-change
mise run install-macos-overlay-app
open -a Foreman
```

See [macOS App Bundle](docs/macos-overlay/app-bundle.md),
[macOS Overlay Architecture](docs/macos-overlay/architecture.md), and
[macOS Overlay Validation](docs/macos-overlay/validation.md) for the deeper app
workflow.

## What Foreman shows

Foreman starts from live tmux inventory and layers higher-confidence signals on
top of it.

- **Inventory**: tmux sessions, windows, panes, titles, working directories, and
  captured preview lines.
- **Harness identity**: Claude Code, Codex CLI, Pi, Gemini CLI, OpenCode, or a
  non-agent pane when you ask for all panes.
- **Status**: working, idle, needs attention, error, or unknown.
- **Provenance**: whether status came from a native hook or from compatibility
  heuristics.
- **Actions**: focus tmux, send input, search/filter, inspect details, and open
  related PR/provider context.

`native hook` means Foreman read a structured harness signal. `compatibility
heuristic` means Foreman inferred state from tmux-visible behavior and treats it
as lower confidence.

## Dashboard basics

Run:

```bash
foreman
```

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

`Attention → Recent` is not pure recency sort. It keeps urgent panes above idle
panes, then uses real pane/native-signal activity as the recency tiebreaker.

## Native harness support

| Harness | Compatibility mode | Native mode |
|---|---:|---:|
| Claude Code | yes | yes |
| Codex CLI | yes | yes |
| Pi | yes | yes |
| Gemini CLI | yes | no |
| OpenCode | yes | no |

Wire native hooks after installing Foreman:

```bash
foreman --setup --user --project
foreman --doctor
```

Use `--repo` when the repo to diagnose or wire is not your current directory:

```bash
foreman --setup --project --repo /path/to/repo
foreman --doctor --repo /path/to/repo
```

If an existing pane was started before hook wiring changed, restart that agent
pane. Setup updates files; it does not repair already-running processes.

See [Operator Guide](docs/operator-guide.md) for setup scopes, doctor fixes,
native hook examples, notification config, UI preferences, and troubleshooting.

## Pull requests, provider cards, and linked repositories

Foreman's JSON control API can attach PR metadata and read-only extension cards:

```bash
foreman agents --json --pull-requests
foreman agents --json --extensions
```

The macOS app renders PR/inventory first, then asks for extension cards only for
the selected pane:

```bash
foreman extensions --pane %42 --json
```

The included Harness Kit provider example maps `hk brief --json` and
`hk status --json` into lifecycle cards such as `NEEDS VALIDATION`,
`NEEDS REVIEW`, `NEEDS SYNC`, `READY`, and `NO WORK`. It is read-only: Foreman
copies or opens commands and evidence, but it does not run mutating HK commands
such as `hk sync`, `hk export`, or `hk ready`.

Install and operate the provider from
[Harness Kit Provider](docs/providers/harness-kit.md).

If an agent pane runs from notes, Obsidian, scratch space, or a launcher
directory while the relevant code lives elsewhere, link the pane explicitly:

```bash
foreman links add --pane %82 --repo ~/git_repositories/foreman --json
foreman links list --json
foreman links remove --pane %82 --json
```

Foreman still displays the pane's real working directory as `Workspace`, but PR
lookups and extension providers use the linked repository. Links are guarded by
the pane working-directory fingerprint so stale tmux pane IDs do not silently
point at the wrong repo.

## Notifications

Foreman can send desktop notifications for completion and attention states. On
macOS, `alerter` is the preferred backend because notification clicks can focus
the related tmux pane. Custom notification sounds can use
`notification-sounds:<prefix>` so playback stays on the notification path instead
of direct `afplay` audio.

See [Operator Guide — Notifications](docs/operator-guide.md#notifications) for
configuration, custom sound routes, and troubleshooting. That guide includes both
macOS custom sound routes: direct file playback and the `alerter --sound`
notification-sound prefix path that better respects Focus / Do Not Disturb.

## Control API for scripts and clients

The Rust CLI is also the control API used by `Foreman.app` and scripts:

```bash
foreman agents --json
foreman agents --json --all-panes
foreman agents --json --pull-requests
foreman agents --json --extensions
foreman extensions --pane %42 --json
foreman focus --pane %42 --json
foreman send --pane %42 --text "continue" --json
```

Use `foreman <command> --help` for the exact contract. These commands are the
stable seam for clients; tmux scraping and native signal details stay behind the
CLI.

## Demo

The macOS overlay demo is generated from the real Swift renderer with fixture
agent data, so it is deterministic rather than a live desktop recording.

![Foreman macOS overlay demo](demos/macos-overlay-demo.gif)

The terminal dashboard demo is a VHS recording of the TUI path:

![Foreman dashboard demo](demos/readme-quickstart.gif)

## Docs

- [Docs index](docs/README.md) — start here for the durable docs map
- [Operator Guide](docs/operator-guide.md) — setup, dashboard, config, hooks,
  notifications, extension providers, and troubleshooting
- [Repo Tour](docs/tour.md) — contributor-oriented code map and reading order
- [Workflow Guide](docs/workflows.md) — HK lifecycle, validation ladder, release
  process, and `.ai/` policy
- [Architecture](docs/architecture.md) — system boundaries and module map
- [macOS App Bundle](docs/macos-overlay/app-bundle.md) — build, install, launch,
  and validate `Foreman.app`
- [macOS Overlay Architecture](docs/macos-overlay/architecture.md) — Swift app
  boundaries and control API seams
- [Harness Kit Provider](docs/providers/harness-kit.md) — install and operate
  the read-only HK provider
- [Changelog](CHANGELOG.md) — release history

## Development

Start meaningful work on a branch and track it with Harness Kit:

```bash
git checkout -b feat/<slug>
hk start <slug> --plan "Describe the intended change" --target .
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
| `mise run check` | Fast quality gate; this is what CI calls |
| `mise run verify` | Heavy validation, including release/UX evidence |
| `mise run verify-release` | Release-confidence operator gauntlet |
| `mise run pr-preflight` | Large-PR checklist and cheap merge-prep guardrails |
| `mise run validate-macos-overlay-change` | Required lane for macOS app, overlay, keyboard/focus, screenshot, or control-API changes |
| `mise run capture-macos-overlay-demo` | Regenerate the deterministic macOS overlay demo GIF/MP4; requires `ffmpeg` |
| `mise run install-macos-overlay-app` | Build, install, and reset `~/Applications/Foreman.app` |
| `mise run verify-macos-overlay-app` | Non-activating app-bundle smoke test |
| `mise run native-preflight` | Check local real-harness readiness |
| `mise run verify-native` | Opt-in real Claude, Codex, and Pi E2E drill |
| `mise run verify-ux` | TUI runtime smoke and UX artifact refresh |

Validation rule of thumb:

- TUI/reducer/config change: start with focused Rust tests, then `mise run check`.
- macOS overlay/control API change: run `mise run validate-macos-overlay-change`.
- Native harness/hook behavior: run `mise run native-preflight`, then opt into
  `mise run verify-native` when done.
- Release-sensitive change: run `mise run verify` before tagging.

CI calls `mise run ci`, which maps to the fast check gate. See
[Workflow Guide](docs/workflows.md) for HK lifecycle, validation layers, `.ai/`
policy, and release evidence.

## Release plan for `1.4.0`

The extension-provider platform, linked repositories, selected-pane macOS
loading, deterministic macOS demo, and HK provider example are a minor release.
The planned release version is `1.4.0`.

Before tagging:

```bash
python3 - <<'PY'
import tomllib
from pathlib import Path
assert tomllib.loads(Path("Cargo.toml").read_text())["package"]["version"] == "1.4.0"
PY
rg -n 'Current crate version: `1.4.0`|## 1.4.0' README.md CHANGELOG.md
mise run check
mise run verify
```

After the version-bump PR is squash-merged:

```bash
git checkout main
git pull --ff-only origin main
git tag -a v1.4.0 -m "Release 1.4.0"
git push origin v1.4.0
gh run list --workflow Release --limit 3
gh run watch <run-id> --exit-status
gh release view v1.4.0
```

The release workflow rejects tags that do not match `Cargo.toml`, rebuilds the
release binaries, uploads validation evidence, and publishes GitHub release
archives for Linux and macOS.
