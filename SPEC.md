---
id: foreman-spec
title: foreman Specification
description: >
  Correctness envelope for foreman — the requirements, contracts, invariants,
  and acceptance criteria that any valid implementation must satisfy.
index:
  - id: summary
    keywords: [overview, product, foreman, tmux, agents]
  - id: requirements
    keywords: [must, should, may, native-mode, compatibility-mode, notifications]
  - id: interfaces
    keywords: [cli, config, tmux, integrations, pull-requests, notifications]
  - id: invariants
    keywords: [state, selection, precedence, failure-modes, observability]
  - id: acceptance
    keywords: [validation, scenarios, check, verify, ci]
---

# foreman — Specification

> This document defines the correctness envelope for foreman. For how the
> system is organized, see `docs/architecture.md`. For how to work in this repo,
> see `AGENTS.md`.

## Summary

Foreman is a TUI for managing AI agents across tmux. It monitors AI coding
agents running inside tmux panes and gives the operator a single control surface
for navigation, direct input, pull request awareness, and notifications.

Foreman is intentionally designed around two integration paths:
- `native mode` for harnesses that expose structured hooks, events, or machine-readable state
- `compatibility mode` for harnesses that can only be observed through tmux-visible process data and terminal capture

Native integrations are the preferred long-term architecture. Compatibility
integrations exist so the dashboard remains useful even when a harness does not
yet expose a stable contract.

## Glossary

- **tmux session**: A top-level tmux workspace that can contain multiple windows.
- **tmux window**: A container inside a session that can contain one or more panes.
- **tmux pane**: The basic terminal surface monitored and focused by the dashboard.
- **agent pane**: A pane recognized as running a supported AI coding agent.
- **non-agent pane**: A pane not recognized as running a supported AI coding agent.
- **operator**: The human user driving the dashboard.
- **focus**: Moving tmux view and selection to a target pane.
- **popup mode**: A mode intended for use inside a tmux popup, where successful focus-oriented actions close the dashboard automatically.
- **summary panel**: A compact view of current work, TODOs, or recent derived activity.
- **subagent**: A child task or delegated unit of work surfaced by an agent that supports this concept.
- **flash navigation**: Short-label jump navigation that lets the operator quickly target any visible item.
- **pull request panel**: The product surface that shows pull request status and related actions for the selected workspace.
- **notification profile**: A named notification-behavior preset that determines which status transitions emit alerts.
- **native mode**: An integration path that consumes structured events, hooks, or machine-readable output from a supported agent harness.
- **compatibility mode**: A fallback integration path that infers state from tmux-visible process and terminal behavior when no structured integration is available.

## Goals / Non-Goals

**Goals:**

- Provide a terminal-native operator console for active multi-agent work inside tmux.
- Let an operator tell at a glance which agents are working, done, blocked, or broken.
- Prioritize reliable completion and attention detection over deep parsing of every tool or approval detail.
- Provide enough product and validation detail that another team could rebuild the product without the original implementation.
- Preserve a keyboard-first workflow.

**Non-Goals:**

- Foreman is not a general-purpose tmux replacement.
- Foreman does not define a general plugin platform for arbitrary integrations.
- Foreman does not require deep inspection of tool-by-tool activity, approval transcripts, or every intermediate agent decision.
- Foreman does not include pull request mutation features such as merging, reviewing, or commenting.
- Foreman does not promise identical behavior on every operating system or every notification backend.
- This specification does not define internal module names, source file names, or implementation-specific APIs.

## Requirements

### MUST

**R1. Product identity and command surface**

- The product name is `Foreman`.
- The primary binary and command name is `foreman`.
- The repository description is `TUI for managing AI agents across tmux`.

**R2. Startup and configuration**

- The `foreman` command starts the interactive dashboard.
- The CLI can print the expected config file location and exit.
- The CLI can initialize a default config file and exit.
- The CLI can diagnose local install, hook wiring, and runtime fallback state and exit.
- The CLI can emit machine-readable diagnosis output.
- The CLI can perform safe, additive setup fixes or scaffolds for supported integrations.
- The CLI can target setup and diagnosis at the current repo or an explicit repo path.
- The CLI supports user-scoped and project-scoped setup for supported integrations.
- The CLI can limit setup writes to selected supported integrations.
- The CLI can load a user config file.
- The CLI can override runtime polling and capture settings from the command line.
- The CLI can enable popup behavior.
- The CLI can enable debug logging mode.
- The CLI can disable notifications for the current run.

**R3. File logging**

- Normal interactive runs write session logs to a user-scoped log directory.
- The latest active run is easy to locate.
- Older log files are cleaned up automatically.

**R4. Multi-session monitoring**

- Foreman monitors panes across all tmux sessions, not just the currently attached session.
- Monitoring refreshes on a configurable interval.
- Foreman captures recent pane output for analysis and preview.
- Foreman keeps tmux pane metadata fresh across the full inventory, but it does
  not need to recapture full preview text for every pane on every refresh.
- Preview freshness is prioritized for the selected row, the current sidebar
  viewport, and panes that have changed identity or need attention.
- Off-screen panes may temporarily reuse cached preview text and are refreshed
  gradually across later polls.

**R5. Integration architecture**

- Foreman supports two integration modes: native mode and compatibility mode.
- Native mode is treated as the preferred path whenever a harness supports it.
- Compatibility mode exists as a fallback rather than the primary long-term architecture.

**R6. Supported harnesses and rollout order**

- Foreman recognizes these supported harness families:
- Claude Code
- Codex CLI
- Pi
- Gemini CLI
- OpenCode
- The current v1 support matrix is:
- Claude Code: compatibility mode and native mode
- Codex CLI: compatibility mode and native mode
- Pi: compatibility mode and native mode
- Gemini CLI: compatibility mode
- OpenCode: compatibility mode
- Foreman distinguishes recognized agent panes from unrecognized panes.
- Claude Code, Codex CLI, and Pi are first-class native integrations in the
  current v1 line.
- Future supported harnesses should use native integrations where practical and
  compatibility integrations otherwise.

**R7. Visibility controls**

- The dashboard hides non-agent sessions and non-agent panes by default.
- The operator can toggle both visibility rules at runtime.

**R8. Status model**

- Each recognized agent exposes a coarse operator-facing status model:
- `working`
- `needs attention`
- `idle`
- `error`
- `unknown`
- The product optimizes for answering these operator questions:
- Is the agent currently working?
- Is the agent done or otherwise waiting for me?
- Is the agent in a broken or unclear state?
- Short-lived state transitions are debounced so active work does not flicker noisily.
- Fine-grained details such as individual tool calls, explicit approval prompts, or low-level acceptance state may be omitted from the primary model.

**R9. Main UI surfaces**

- The dashboard includes a header with system and workload overview.
- The dashboard includes a sidebar organized as a semantic session/agent tree in the default agents-only view.
- The default sidebar elides singleton tmux window rows so the operator sees the actionable agent directly under its session.
- Topology-oriented views preserve explicit tmux window rows so the raw session/window/pane tree remains available when needed.
- Singleton counts such as `1w/1p` or `1p` are not repeated when the visible hierarchy already communicates that structure.
- The dashboard includes a detailed pane preview.
- The dashboard includes an input area for sending text to the selected agent.
- The dashboard includes a footer with contextual actions or hints.
- The footer uses labeled action groups so movement, selection, search, view,
  and panel controls are easy to scan.
- The dashboard includes a help surface with a legend for compact badges, status indicators, and status-source hints.
- The help surface is keyboard-scrollable in layouts where its full contents do not fit at once.
- The dashboard may include summary, subagent, and pull request detail panels.
- Popup mode preserves the same side-by-side sidebar/details mental model as
  the regular dashboard when the popup has enough width and height.
- The preview identifies whether the selected or actionable pane status comes from native mode or compatibility heuristics.
- The preview can surface setup or diagnosis hints when compatibility fallback is likely caused by missing hook wiring or failing hook commands.

**R9b. Details information hierarchy**

- The details pane uses stable, labeled sections so alerts, pull request state, current selection, diagnostics, dashboard overview, and pane output are visually distinguishable.
- The details pane uses aligned scan rows for recurring facts such as status, source, target, workspace, PR state, diagnostics, and actions.
- The selected target summary avoids duplicating the same status, source, target, and command facts in multiple adjacent blocks.
- Event summaries and pane output use distinct names so recent notifications or refreshes are not confused with terminal output.

**R10. Keyboard-first interaction**

- The operator can move through visible items with keyboard navigation.
- The operator can change focus between the sidebar, preview, and input regions.
- The operator can collapse and expand session groups.
- The operator can trigger focus on the selected pane.
- The operator can cycle a harness/provider filter that narrows the tree to one
  supported agent family at a time, with empty harness views skipped by default.
- The operator can cycle the active theme at runtime.
- The operator can scroll the help surface with the keyboard when help is open.
- Escape cancels or dismisses non-normal modes; in normal mode it quits like `q`.

**R11. Pane focus behavior**

- Foreman can focus the selected pane by switching tmux context as needed.
- Session and window selections can resolve to an actionable visible pane for focus-oriented actions.
- The UI identifies the resolved actionable pane for focus-oriented and direct-input actions.
- In popup mode, successful focus-oriented actions close the dashboard automatically.

**R12. Direct input**

- The operator can compose and send text to the actionable selected agent pane.
- Multiline composition is supported.
- Text editing behaves safely for non-ASCII input.

**R13. Operational pane actions**

- The operator can kill the selected pane with explicit confirmation.
- The operator can rename the selected pane's window.
- The operator can spawn a new agent window in the selected session.
- Lightweight canned responses for supported harnesses are optional and are not core to the rebuild.

**R14. Search**

- The dashboard provides interactive search that filters visible items by query.
- Search supports moving between matches.
- Search renders inline in the footer as `/<query>` with match count and does
  not obscure the dashboard with a floating overlay.
- Search can confirm into a focus action or an expansion action depending on the selected target.
- Search can cancel and restore the previous cursor position.

**R15. Flash navigation**

- The dashboard provides a fast label-based jump mode.
- Flash navigation renders short labels inline on visible targets without obscuring the target rows.
- It supports both jump-only and jump-and-focus variants.
- It supports overflow beyond single-character labels.
- It can be canceled cleanly.

**R16. Sorting**

- The dashboard provides at least two sidebar sort modes:
- `stable`
- `attention -> recent`
- `stable` keeps the common monitoring view from reordering rows due to volatile status changes.
- `attention -> recent` sorts by status rank first, then by most recent activity.
- Changing sort mode preserves the current logical selection whenever possible.

**R17. Pull request awareness**

- When pull request monitoring is enabled and relevant local tooling is available, Foreman detects whether the selected workspace has an open pull request.
- Foreman shows compact pull request state for the selected workspace.
- Foreman provides an expandable pull request detail view.
- Foreman supports opening the pull request in the browser.
- Foreman supports copying the pull request URL.
- Foreman avoids repeated unnecessary lookups for the same visible paths.
- Foreman degrades gracefully when pull request data is unavailable.

**R18. Notifications**

- When notifications are enabled, Foreman notifies when a working agent becomes complete or returns to a ready state.
- When notifications are enabled, Foreman notifies when an agent enters a `needs attention` state.
- Notifications are suppressed when the operator is already looking at the relevant pane or already focused on that agent in the dashboard, except popup mode may still notify for the selected pane because the dashboard is transient.
- Per-agent cooldowns reduce noise.
- Runtime muting and unmuting are supported.
- Runtime notification-profile switching is supported.
- Notification backend fallback is supported when a preferred path is unavailable.

**R19. Validation framework**

- The project has unit tests.
- The project has local build, format, lint, and test workflows.
- The project has shell-based end-to-end tests for core user journeys.
- Continuous integration checks build, test, formatting, and linting.
- `mise run check` is the fast quality gate.
- `mise run ci` delegates to the same quality gate as local checks.

### SHOULD

- Foreman should keep the common path fully usable without a mouse.
- Foreman should keep selections stable across refreshes, filtering changes, and re-sorting.
- Foreman should avoid noisy pull request polling and unnecessary repeated notifications.
- Foreman should make active work, waiting states, and errors visually distinct at a glance.
- When both are available, native-mode status should be treated as more authoritative than compatibility-mode status.

### MAY

- Foreman may show derived summary information such as TODOs, recent activity, or child-task activity.
- Foreman may surface a remaining-context indicator when a supported harness exposes it.

## Interfaces & Contracts

### CLI contract

- The command-line interface is exposed through `foreman`.
- It supports normal interactive startup.
- Normal interactive startup renders immediately with a loading state before the first tmux inventory refresh completes.
- Popup startup may seed that first render from a fresh persisted inventory snapshot, but cached state must be marked and replaced by live tmux refresh.
- It supports showing the config path.
- It supports showing resolved config, UI state, popup cache, and runtime values.
- It supports resetting persisted runtime UI state.
- It supports initializing a config file.
- It supports popup execution mode.
- It supports debug logging mode.
- It supports notification suppression for a single run.
- It supports runtime overrides for monitoring cadence and capture depth.
- Hook-based native integrations may ship companion helper commands when the
  main dashboard process is not the right place to consume hook stdin directly.

### Config contract

- Foreman uses a user-scoped config file.
- Config supports pane polling interval.
- Config supports pane capture depth.
- Config supports popup startup cache freshness.
- Config supports per-harness integration mode selection or preference where multiple modes are possible.
- Config supports pull request monitoring enablement and cadence.
- Config supports notification enablement.
- Config supports notification cooldowns.
- Config supports notification backend preference.
- Config supports a small set of built-in named notification profiles.
- Config supports selection of the active notification profile.
- Config supports selection of the default UI theme.
- Config supports selection of the startup sort mode with `stable` and `attention-recent` values.
- Runtime UI choices may persist across launches, including sort mode, theme, filters, collapsed sessions, and last selected target.
- Built-in UI themes include `catppuccin`, `gruvbox`, `tokyo-night`, `nord`,
  `dracula`, `terminal`, and `no-color`.
- Config may override the local signal directory or equivalent bridge path for
  harnesses that use file-backed native integrations.
- Backward-compatible migration from an older flatter notification-sound format may be supported.

### Agent integration contract

- For each supported harness, Foreman defines how an agent process is identified.
- For each supported harness, Foreman defines whether it is operating in native mode or compatibility mode.
- For each supported harness, Foreman defines which signals are authoritative for `working`, `needs attention`, `idle`, `error`, and `unknown`.
- For each supported harness, Foreman defines fallback behavior when native integration becomes unavailable.
- For each supported harness, Foreman defines how state transitions are debounced to avoid flicker and notification noise.
- Native integrations should consume structured hooks, events, or machine-readable streams where available.
- File-backed hook bridges use the tmux pane identity from `TMUX_PANE` so signal files match the pane IDs Foreman discovers from tmux inventory.
- Compatibility integrations may use tmux-visible process metadata and captured terminal content, but these heuristics are treated as lower-confidence signals.
- Compatibility integrations must not keep reviving an agent identity from stale title or preview text once the pane foreground command has returned to a shell.

### tmux contract

- Foreman depends on tmux for discovery and control.
- Foreman can discover sessions, windows, and panes.
- Foreman can capture recent pane content.
- Foreman can send text input to panes.
- Foreman can focus sessions, windows, and panes.
- Foreman can create new windows.
- Foreman can rename windows.
- Foreman can kill panes.
- If tmux is unavailable or not running, the dashboard does not crash and remains usable as an empty shell with a visible error state.

### Pull request contract

- Foreman may use local Git and GitHub-aware tooling to determine whether the selected workspace has an associated open pull request.
- Browser-opening and clipboard-copy behavior are part of the user-facing contract when platform support exists.

### Notification contract

- Foreman supports automatic fallback across available notification backends.
- Sound playback and richer notification backends are best-effort and do not destabilize the dashboard when unavailable.

### Task contract

| Command | Purpose |
|---------|---------|
| `mise run setup` | Install tools and project dependencies |
| `mise run check` | Fast quality gate for format, lint, typecheck, and test |
| `mise run verify` | Heavier validation beyond the fast gate |
| `mise run verify-release` | Release-confidence compiled-binary tmux gauntlet with checklist/report artifact |
| `mise run ci` | CI entrypoint, aligned with local quality checks |

## Invariants

### Primary state

- The dashboard maintains a periodically refreshed inventory of agent panes and non-agent panes.
- The dashboard maintains integration-mode state per recognized agent.
- The dashboard maintains a current selection.
- The dashboard maintains a focused panel.
- The dashboard maintains filter and optional-panel toggle state.
- The dashboard maintains pull request lookup state.
- The dashboard maintains notification state.
- The dashboard maintains short-lived modal state for search, flash navigation, renaming, spawning, confirmation, and help.

### Mode precedence

- Only one high-priority mode consumes input at a time.
- Help, rename, flash navigation, search, spawn, input editing, preview scrolling, and ordinary sidebar navigation have a deterministic precedence order.

### Selection invariants

- Refreshes do not leave the dashboard pointing at an invalid target.
- Selection remains logically stable across refresh and sorting changes.
- Collapsed and expanded session state persists across refreshes.
- Preview refresh prioritization does not change the logical selection model or
  tmux targeting semantics.

### Integration precedence invariant

- If a recognized agent has both native-mode and compatibility-mode signals available, Foreman prefers native-mode state as the source of truth for operator-visible status and notifications.

### Pull request panel invariant

- If a pull request detail panel auto-opens on first discovery for a workspace, it does not repeatedly re-open for that same workspace after the operator closes it.

### Failure modes and edge cases

- tmux may be missing or not running.
- A pane may fail to capture during polling.
- Off-screen panes may temporarily show cached preview text while a later poll
  refreshes them.
- Native hooks, events, or machine-readable streams may be unavailable, misconfigured, or temporarily disconnected.
- Compatibility heuristics may produce ambiguous or stale results.
- Pull request tooling may be missing, unauthenticated, or rate-limited.
- The selected workspace may not be a repository with a pull request.
- Browser or clipboard actions may be unavailable on the current machine.
- A notification backend or richer sound path may be unavailable.
- Mouse hit-testing may be approximate rather than exact in some layouts.
- Some panel combinations may force a summary surface to remain visible even when a toggle suggests otherwise.
- In all of these cases, Foreman fails soft rather than fail closed.

### Observability

- Foreman writes structured run logs to disk.
- The latest active run is easy to locate.
- User-facing action failures are surfaced in the UI.
- Pull request polling lifecycle and notification backend selection are observable through logs.
- Debug logging surfaces timing for `move-selection`, `render_frame`, `inventory_tmux`, and `inventory_native`.
- tmux timing logs expose how many previews were freshly captured vs reused from
  cache on a given refresh.
- Header-level system stats give the operator a quick read on local CPU and memory pressure.

## Acceptance

### Validation entrypoints

```bash
mise run check
mise run verify
mise run verify-ux
mise run ci
```

### Acceptance scenarios

**A1. Config utility flow**

- Given a writable user config directory, when the operator asks `foreman` to reveal the config path, the CLI prints the location and exits successfully.
- Given a writable user config directory, when the operator asks `foreman` to create a default config, the CLI writes the file and exits successfully.

**A2. Normal startup logging**

- Given a normal interactive run, when the dashboard starts, it creates a fresh run log, updates the latest-log pointer, and retains only recent logs.
- Given debug logging mode, startup writes debug lines in addition to the normal structured run log.

**A3. Multi-session discovery**

- Given at least two tmux sessions with visible panes, when the dashboard starts, it renders a loading state immediately and then both sessions appear in the sidebar after the first background inventory refresh.

**A4. Agent detection**

- Given panes running supported agent families, including Claude Code, Codex CLI, Pi, Gemini CLI, and OpenCode signatures, when the monitor refreshes, those panes appear as recognized agents rather than generic panes.
- Given a pane that previously showed supported agent output but whose foreground command is now a shell prompt, compatibility mode no longer treats that pane as an active agent based on stale preview or title text alone.

**A5. Integration mode selection**

- Given a supported harness with native integration available, when the dashboard discovers that agent, it uses native mode as the authoritative source of status.
- Given a Claude, Codex, or Pi hook bridge runs inside a tmux pane, when it writes a native signal, Foreman matches that signal to the same pane ID and promotes the pane from compatibility to native mode.
- Given native integration is not available and the harness is still visible in tmux, the dashboard falls back to compatibility mode instead of dropping the pane entirely.
- Given config forces compatibility mode for Claude, Codex, or Pi, native signal data does not override compatibility status for that run.

**A6. Default hiding and toggle behavior**

- Given a mix of agent and non-agent panes, when the dashboard starts, non-agent-only sessions and non-agent panes are hidden by default.
- When the operator toggles the relevant filters, those hidden items become visible.
- Given one visible agent pane inside a one-pane tmux window, the default
  sidebar shows the agent pane directly below its session, with harness glyph
  and status styling on the pane row.
- Given topology-oriented filters are enabled, the same one-pane tmux window
  remains visible as an explicit window row.
- Given visible supported harness families, pressing the harness-view key cycles
  the sidebar through those harnesses plus the unfiltered view and reconciles the
  selection to a visible target without stopping on empty harness views by
  default.

**A7. Status stability**

- Given an agent shows signs of active work, repeated refreshes continue to show that agent as working.
- If the activity signal disappears only briefly, the UI does not flicker immediately back to idle.

**A7b. Details pane hierarchy**

- Given a selected agent pane with pull request and diagnostic state, the details pane separates current alerts, pull request state, selected target facts, diagnostics, overview, and recent terminal output under distinct labels.
- Given the details pane includes recurring metadata, labels and values align into predictable scan rows instead of loose prose.
- Given recent notification or pull request activity exists, it is labeled as activity rather than recent terminal output.
- Given a pane is selected, the details pane shows status/source/target metadata once before the recent terminal output section.

**A8. Keyboard navigation**

- Given visible sessions and panes, keyboard navigation advances through visible targets.
- Activating a session header expands or collapses that session.
- Given the dashboard is running, pressing `t` cycles the active theme without
  changing selection or mode.
- The heavy validation lane includes a deterministic burst-navigation perf smoke that enforces a bounded `move-selection` latency budget with a crowded tmux fixture.
- Given the dashboard is running, pressing `?` opens help with a legend for
  status and harness marks.
- Given the help surface is taller than the visible popup, `j` / `k`,
  arrow keys, and page navigation keys scroll it without changing the
  underlying selection.
- Given focus changes between sidebar, preview, and compose, the footer updates
  its primary hints to match the active panel instead of repeating a single
  static control summary.
- Given a window row is selected, focus-oriented actions resolve to the top visible
  actionable pane instead of silently no-oping.
- Given a session or window row is selected, the preview identifies the resolved
  target pane and help explains that `f` jumps tmux there.
- Given a selected or actionable pane is using compatibility mode, the preview
  and help identify that status as compatibility-derived and lower confidence
  than native hook signals.

**A9. Popup auto-exit**

- Given the dashboard is running in popup mode, when the operator focuses a target pane successfully, tmux switches to that pane and the dashboard closes.

**A10. Direct input**

- Given an actionable agent row is selected, when the operator composes text,
  including multiple lines, and sends it, that content is delivered to the resolved
  agent pane.

**A11. Claude Code first-class support**

- Given the product is being rebuilt from scratch, when the first native integration is delivered, Claude Code is the first harness supported in native mode.
- Completion and needs-attention detection for Claude Code are reliable enough to drive notifications without terminal-only parsing as the primary signal.
- Foreman ships a supported Claude Code hook bridge so native status does not
  depend on ad hoc user scripts.

**A11b. Codex CLI native hook support**

- Given Codex CLI native integration is configured, real Codex hook events can
  drive native `working` and `idle` status without terminal parsing as the
  primary source.
- Foreman ships a supported Codex hook bridge so Codex native status does not
  depend on ad hoc user scripts.

**A11c. Pi native extension support**

- Given Pi native integration is configured, Pi lifecycle events can drive
  native `working` and `idle` status without terminal parsing as the primary
  source.
- Foreman ships a supported Pi bridge and documented extension pattern so Pi
  native status does not depend on ad hoc user scripts.
- Real Pi-binary E2E proves a dashboard-sent prompt can produce native Pi
  signals and a completion notification.

**A12. Pane operations**

- Given a pane is selected, a confirmed kill action terminates the pane.
- Renaming the selected window applies the new name.
- Spawning a new agent in the selected session creates a new window running that agent.

**A13. Search**

- Given several visible sessions and panes, entering search text filters the sidebar to matching targets and moves selection to a match.
- Canceling search restores the previous selection.

**A14. Flash navigation**

- Given flash navigation is active, typing a visible label jumps directly to that target while the labels remain visible inline in the sidebar.
- Using the focus variant focuses the chosen pane in tmux.

**A15. Sorting**

- Given the dashboard is running, cycling sort mode reorders the sidebar by the selected preset (`stable` or `attention -> recent`) and preserves the current logical selection where possible.
- Given `[ui].default_sort = "attention-recent"`, startup uses the attention/recent order before the operator presses `o`, even when persisted UI state contains a different sort.
- Given the operator changes sort/filter/theme/collapsed state or selection, a later launch restores those choices when the target still exists; explicit `[ui]` config keys override persisted theme/sort.
- Given persisted UI state is corrupt or unwanted, `foreman --doctor` reports it and `foreman --reset-ui-state` removes it.

**A16. Pull request awareness**

- Given pull request monitoring is enabled and local tooling is available, when the selected workspace has an open pull request, compact pull request state appears, detail can be expanded, manual refresh reports in-progress feedback, and browser or copy actions work.

**A17. Pull request graceful degradation**

- Given pull request monitoring is enabled but the local environment cannot provide pull request data, monitoring continues and the dashboard remains usable without that pull request surface.

**A18. Notification behavior**

- Given notifications are enabled, agent completion emits a completion notification unless suppression rules apply.
- Given notifications are enabled, entry into a `needs attention` state emits an attention notification unless suppression rules apply.
- Given multiple notification backends are configured, Foreman uses configured backend order and falls back when an earlier backend fails.
- Given the startup notification profile excludes a transition kind, that transition is suppressed until the operator changes profile at runtime.
- Muting and profile switching are reflected in the UI.

**A19. Validation stack**

- Given a clean checkout, when local validation commands run, unit tests pass and the expected local quality gate succeeds.
- Given `vhs` is available locally, when `mise run verify-ux` runs, focused TUI
  smoke tests pass and fresh visual artifacts can be generated from the live
  binary.
- Given `mise run verify-release` runs, the compiled binary completes the
  startup/discovery, action, and integration gauntlets inside temporary tmux
  worlds and emits a durable checklist/report artifact under `.ai/validation/release/`.
- Given the focused runtime smoke runs, help/legend display, harness-view
  cycling, deterministic native-hook provenance plus attention surfacing, live
  `f` focus, and acting on the filtered selection all work in the compiled
  binary inside real tmux.
- Given the navigation performance smoke runs in the heavy UX lane, selection
  bursts do not trigger pull-request lookups for every intermediate workspace.
- Given the popup startup cache path is enabled, the heavy UX lane proves
  cached-first render and verifies startup-cache writes stay cheap and bounded.
- When CI runs, build, test, formatting, and lint checks succeed.
- When CI runs the heavy validation lane, the UX artifact bundle and
  release-gauntlet report are uploaded as reviewable artifacts.
- Those review artifacts come from a stable validation root under `.ai/validation/`
  and missing evidence is treated as a failure.

### Definition of done

- The required behaviors above are implemented.
- The local validation workflow is green.
- End-to-end smoke coverage exists for core operator flows.
- Continuous integration is green.
- User-facing documentation matches the actual CLI and runtime behavior.
