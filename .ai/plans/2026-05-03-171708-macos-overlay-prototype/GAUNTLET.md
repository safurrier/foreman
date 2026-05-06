---
id: macos-overlay-gauntlet
title: macOS Overlay Gauntlet
description: Foreman-style feature parity and validation gauntlet for the Swift macOS overlay.
---

# macOS Overlay Gauntlet

## Purpose

Foreman's Rust TUI has release gauntlets that prove workflows, not just isolated
functions. The macOS overlay needs the same style of confidence, scoped to what
the overlay is supposed to be: a fast global control surface.

This gauntlet is **not** a promise to clone every TUI feature. It proves parity
for core operator workflows and documents intentional deferrals.

## Gauntlet Layers

| Layer | Environment | Proves |
|---|---|---|
| Core state tests | Swift XCTest | filtering, selection, modes, command reducer |
| Client/process tests | Swift XCTest + fake runner | decode, command failures, large stdout/stderr safety |
| Fake app action gauntlet | Swift app + fake `foreman` | real UI actions call `agents`, `focus`, `send` correctly |
| Visual fixture gauntlet | Swift app + deterministic fixtures | important UI states render clearly |
| Real control API smoke | isolated tmux + real `foreman` | Foreman/tmux `agents/focus/send` contract works |
| Manual Mac smoke | real app session | hotkey, Spaces, focus/activation, feel |

## Feature Parity Matrix

| Capability | TUI behavior | Overlay target | Proof target |
|---|---|---|---|
| Agent discovery | multi-session tmux tree | searchable agent list | fake fixture + real tmux smoke |
| Search | `/` search mode | type-to-search by default | core test + app gauntlet |
| Keyboard navigation | `j/k`, arrows, tree movement | arrows initially; consider `j/k` | core test + app gauntlet |
| Selection visibility | sidebar scroll follows selection | list scroll follows selection | app gauntlet / visual many fixture |
| Focus pane | `f` / Enter target focus | Enter, double-click, Focus button | fake app action + real tmux smoke |
| Direct input | compose mode sends to pane | compose + `Cmd+Enter` send | fake app action + real tmux smoke |
| Focus regions | sidebar/details/input | list/detail/compose via Tab | core test + app gauntlet |
| Help | `?`, keyboard-scrollable | `?`, overlay-specific, keyboard-scrollable | app gauntlet + constrained screenshot |
| Preview scroll | details panel scroll | detail region scrolls preview | app gauntlet + screenshot |
| Status/source | badges + provenance | status badge + source metadata | fixture decode + screenshots |
| All panes/non-agents | toggles | optional all-panes toggle | control API + UI test if added |
| Harness filter | provider cycle | optional filter once list grows | core test if added |
| Theme cycle | terminal themes | defer; use Mac appearance first | none |
| PR panel | PR detail/actions | defer until control API expands | none |
| Spawn/rename/kill | operational tmux actions | defer | none |
| Notifications | runtime notification system | defer to Foreman TUI/service | none |

## Proposed Gauntlet Scenario

A deterministic fake app gauntlet should create a fixture with at least four
agents:

1. Claude, needs attention, long preview.
2. Codex, working, long path.
3. Pi, idle.
4. Shell/non-agent if all-panes mode is enabled.

Then drive this workflow:

1. Launch overlay with fake Foreman.
2. Wait for `agents --json` call.
3. Type a search term and assert filtered selection.
4. Press `Down` enough times to move below the initially visible list area.
5. Assert selected row remains visible or screenshot contains the selected row.
6. Press `Enter`; assert fake Foreman saw `focus --pane <selected> --json`.
7. Reopen/refresh overlay.
8. Press `?`; assert help is visible.
9. Scroll help down; assert below-fold help text becomes visible.
10. Press `Esc`; assert help closes and selection/search remain sane.
11. Press `Tab` through list/detail/compose; assert footer or focus indicator updates.
12. Compose text and press `Cmd+Enter`; assert fake Foreman saw
    `send --pane <selected> --stdin --json` and stdin contains the text.

## Artifacts

Expected local artifacts:

```text
.ai/validation/macos-overlay/gauntlet/
├── fake-foreman-calls.log
├── fake-foreman-stdin.log
├── overlay.log
├── screenshots/
│   ├── startup.png
│   ├── many-selection-scrolled.png
│   ├── help-top.png
│   ├── help-scrolled.png
│   └── compose.png
└── summary.md
```

## Exit Criteria

`mise run verify-macos-overlay` should eventually fail on any regression in:

- app launch against fake Foreman
- initial inventory load
- type-to-search
- keyboard navigation
- row selection visibility
- focus action wiring
- send action wiring
- help visibility and scrollability
- visual fixture capture
- real tmux control API smoke
