---
id: operator-guide
title: Operator Guide
description: >
  User-facing install, setup, dashboard, configuration, native hook, and
  notification reference for Foreman operators.
index:
  - id: install-and-setup
  - id: doctor
  - id: dashboard
  - id: configuration
  - id: native-hooks
  - id: notifications
  - id: troubleshooting
---

# Operator Guide

This guide covers the day-to-day Foreman surface: installing local binaries,
wiring harness hooks, reading dashboard state, configuring notifications, and
debugging fallback-heavy panes.

## Install And Setup

Requirements:

- `tmux`
- Rust toolchain with `cargo`

Install Foreman and hook bridges from this checkout:

```bash
mise run setup
mise run install-local
```

First-run setup:

```bash
foreman --setup --user --project
foreman --doctor
foreman
```

Setup is safe to rerun.

| Scope | What it writes |
|---|---|
| `--user` | Home-directory wiring under `~/.claude`, `~/.codex`, and Pi's `~/.pi/agent/extensions` |
| `--project` | Repo-local wiring for the current checkout, or the repo passed with `--repo` |

Without explicit scope flags, `foreman --setup` defaults to project setup when
the current directory looks like a repo, and user setup otherwise.

Setup targets one repo at a time:

```bash
foreman --setup --project --repo /path/to/repo
```

Provider flags limit setup to one integration:

```bash
foreman --setup --project --codex --repo /path/to/repo
foreman --setup --user --claude
```

Setup fixes files and directories. It does not repair already-running panes.
Restart affected agents after wiring changes.

To install into a temporary root for testing:

```bash
mise run install-local -- --root /tmp/foreman-install
```

Minimal tmux smoke run:

```bash
tmux new-session -d -s alpha "sh -lc \"printf '%s\n' 'Claude Code ready'; exec sleep 600\""
foreman
```

## Doctor

Run doctor when the dashboard falls back to compatibility detection more than
expected:

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

Safe doctor fixes:

- initialize default `config.toml`
- create native signal directories under `~/.local/state/foreman/`
- merge Claude hook wiring into `.claude/settings.local.json` or `~/.claude/settings.local.json`
- scaffold or merge `.codex/hooks.json` or `~/.codex/hooks.json`
- scaffold `.pi/extensions/foreman.ts` or `~/.pi/agent/extensions/foreman.ts`

Preview safe repairs:

```bash
foreman --doctor --doctor-fix --doctor-dry-run
```

Apply safe repairs:

```bash
foreman --doctor --doctor-fix
```

Target another checkout:

```bash
foreman --doctor --repo /path/to/repo
```

## Dashboard

Common keys:

| Key | Action |
|---|---|
| `j` / `k` | Move through the tree |
| `Tab` or `1` / `2` / `3` | Switch panel focus |
| `i` | Compose input for the actionable agent row |
| `f` | Focus tmux on the actionable pane |
| `Enter` | Send in compose mode, or act on the selected row in normal mode |
| `/` | Search |
| `o` | Cycle `stable` and `attention->recent` sort modes |
| `s` | Start flash jump with inline labels |
| `S` | Flash jump and focus tmux after the jump |
| `h` | Cycle visible harness families, then return to `all` |
| `H` / `P` | Reveal non-agent sessions or panes |
| `t` | Cycle the active theme |
| `?` | Open the help and legend surface |

Preview and help surfaces show status provenance:

- `native hook` means a structured harness signal.
- `compatibility heuristic` means tmux-observed behavior.

The help overlay includes tree, status, and harness glyph legends. Selected
panes can also show setup hints that point back to
`foreman --setup --user --project` or `foreman --doctor` when compatibility
fallback looks suspicious.

### tmux Popup

Foreman can run as a floating tmux popup so it does not consume a pane or
window. Bind it in tmux with `display-popup` and pass `--popup` so Foreman
knows jump-to-pane actions should close the popup after a successful focus.

Example `~/.tmux.conf` binding:

```tmux
# <prefix>a opens Foreman as a floating operator console.
bind a display-popup -h 80% -w 80% -E -- "$HOME/.cargo/bin/foreman" --popup
```

If popup startup feels slow, use direct argv form as shown above. Avoid wrapping
the command in a shell string unless you need shell expansion, because
`display-popup -E "foreman --popup"` starts the user's shell first.

On macOS, unsigned or quarantined binaries can flash and close when launched
inside `display-popup`. Reinstalling from a trusted build path or clearing the
quarantine attribute for the installed binary fixes that environment issue.

## Configuration

Show resolved config, UI state, popup cache, and runtime values:

```bash
foreman --config-show
```

Reset persisted UI choices for the resolved config:

```bash
foreman --reset-ui-state
```

UI preferences:

```toml
[monitoring]
startup_cache_max_age_ms = 300000

[ui]
theme = "catppuccin" # catppuccin | gruvbox | tokyo-night | nord | dracula | terminal | no-color
default_sort = "stable" # stable | attention-recent
```

Foreman persists runtime UI choices such as theme, sort mode, filters,
collapsed sessions, and the last selected target in a UI state file next to the
resolved config file. Explicit theme and default sort config values in the
`[ui]` table win over persisted values. Persisted values fill in only when those
keys are omitted.

## Native Hooks

Harness support:

| Harness | Compatibility mode | Native mode |
|---|---:|---:|
| Claude Code | yes | yes |
| Codex CLI | yes | yes |
| Pi | yes | yes |
| Gemini CLI | yes | no |
| OpenCode | yes | no |

Foreman native mode is hook-only. Compatibility mode can still classify panes
by tmux-visible behavior for harnesses or states that do not have native
signals.

### Claude Code

`foreman-claude-hook` reads official Claude hook JSON on stdin, uses
`TMUX_PANE` to identify the pane, and writes the per-pane signal file that
Foreman overlays in native mode.

Override the native signal path with:

```toml
[integrations.claude_code]
native_dir = "/path/to/claude-native"
```

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

### Codex CLI

`foreman-codex-hook` reads official Codex hook JSON on stdin, resolves the pane
from `TMUX_PANE` or `--pane-id`, and writes the per-pane signal file that
Foreman overlays in native mode.

Override the native signal path with:

```toml
[integrations.codex_cli]
native_dir = "/path/to/codex-native"
```

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

### Pi

`foreman-pi-hook` is called by a Pi extension on Pi lifecycle events. The
bridge writes the per-pane signal file that Foreman overlays in native mode.

Override the native signal path with:

```toml
[integrations.pi]
native_dir = "/path/to/pi-native"
```

Example project-local `.pi/extensions/foreman.ts`:

```typescript
import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { spawnSync } from "node:child_process";

export default function (pi: ExtensionAPI) {
  const runHook = (event: string) => {
    const paneId = process.env.TMUX_PANE;
    if (!paneId) {
      return;
    }
    const args = ["--event", event, "--pane-id", paneId];
    spawnSync("foreman-pi-hook", args, { stdio: "inherit" });
  };

  pi.on("agent_start", async () => runHook("agent-start"));
  pi.on("agent_end", async () => runHook("agent-end"));
  pi.on("session_shutdown", async () => runHook("session-shutdown"));
}
```

## Notifications

Notification config:

```toml
[notifications]
enabled = true
cooldown_ticks = 3
backends = ["alerter", "notify-send", "osascript"]
active_profile = "all" # all | completion-only | attention-only
sound_profile = "default"

[notifications.sound_profiles.default]
completion = "Tink"
needs_attention = "Ping"
cycle = "random" # random | sequential

[notifications.sound_profiles.custom_macos]
completion = "notification-sounds:foreman-completed-"
needs_attention = "notification-sounds:foreman-needs-input-"
cycle = "random"
```

On macOS, install `alerter` for clickable notifications. When `alerter` reports
a content or action click, Foreman asks tmux to select the notification's
window and pane. Sound entries can be system sound names, audio files, audio
directories, notification sound prefixes, or `none`.

For Do Not Disturb / Focus-friendly custom sounds on macOS, copy AIFF/CAF/WAV
files into `~/Library/Sounds` and reference them with
`notification-sounds:<prefix>`. Foreman randomly or sequentially selects a file
whose basename starts with the prefix and passes that basename to
`alerter --sound`, so playback stays on the macOS notification path. Plain file
and directory paths still use `afplay` and may bypass Focus.

### macOS Custom Sound Profiles

There are two useful ways to configure custom sounds on macOS.

The easy route is a direct file or directory path. Foreman plays these with
`afplay`.

```toml
[notifications]
enabled = true
backends = ["alerter", "osascript"]
sound_profile = "custom-files"

[notifications.sound_profiles.custom-files]
completion = "~/Sounds/foreman/completed"
needs_attention = "~/Sounds/foreman/needs-input"
cycle = "random"
```

This is simple and works with any local audio files Foreman can read. The trade
off is that direct `afplay` playback may ignore macOS Focus / Do Not Disturb.

The Focus-aware route is to install sounds into macOS notification sounds and
let `alerter --sound` play them:

```bash
mkdir -p ~/Library/Sounds
cp ~/Sounds/foreman/completed/*.aiff ~/Library/Sounds/
cp ~/Sounds/foreman/needs-input/*.aiff ~/Library/Sounds/
```

Name files with stable prefixes:

```text
foreman-completed-soft.aiff
foreman-completed-bright.aiff
foreman-needs-input-soft.aiff
foreman-needs-input-urgent.aiff
```

Then reference those prefixes:

```toml
[notifications]
enabled = true
backends = ["alerter", "osascript"]
sound_profile = "macos-notification-sounds"

[notifications.sound_profiles.macos-notification-sounds]
completion = "notification-sounds:foreman-completed-"
needs_attention = "notification-sounds:foreman-needs-input-"
cycle = "random"
```

With this profile, Foreman chooses a matching basename from `~/Library/Sounds`
and passes it to `alerter --sound`. macOS owns playback, so Focus / Do Not
Disturb behavior matches normal notification sounds as closely as `alerter`
allows.

Use `cycle = "sequential"` when you want predictable rotation instead of random
selection.

With VoiceOver, use the VoiceOver Notifications menu (`VO-N`) to navigate to a
Foreman notification, then open the notification actions menu
(`VO-Command-Space`) and choose `Open tmux pane`.

## Troubleshooting

If local navigation feels slow, run with debug logging and inspect the latest
run log:

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

`inventory_tmux` shows how many panes were freshly captured versus reused from
cached previews on that refresh. Startup cache lines include the cache path and
age. `foreman --doctor` also reports the popup cache path, freshness window,
age, and last inventory counts.

If `codex --version` and `npm list -g @openai/codex --depth=0` disagree, fix
the PATH-resolved install first. Foreman's real Codex E2E depends on Codex hook
support, which requires a current Codex CLI on `PATH`.
