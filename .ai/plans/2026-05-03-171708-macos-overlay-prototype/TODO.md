---
id: macos-overlay-prototype-todo
title: Task List
description: Checkable task list for Swift macOS overlay prototype + validation/UX hardening.
---

# TODO — macos-overlay-prototype

## Prototype

- [x] Add Foreman control API DTOs for `agents`, `focus`, and `send`.
- [x] Add CLI commands: `foreman agents --json`, `foreman focus --pane`, `foreman send --stdin`.
- [x] Add SwiftPM macOS overlay prototype under `apps/macos-overlay/`.
- [x] Launch overlay against local `target/debug/foreman`.
- [x] Fix first spinner issue by bounding control API preview text.

## Vendored Design Skills

- [x] Vendor `ehmo/platform-design-skills/skills/macos` into `.agent/skills/macos-design-guidelines`.
- [x] Vendor `petekp/agent-skills/skills/macos-app-design` into `.agent/skills/macos-app-design`.
- [x] Add Foreman-specific overlay UX skill wrapper.
- [x] Add Foreman-specific overlay validation skill wrapper.
- [x] Update `.agent/skills/README.md` and `AGENTS.md` with the new overlay guidance.

## Validation / UX Hardening

- [x] Add deterministic overlay fixture: `apps/macos-overlay/Fixtures/agents-attention.json`.
- [x] Add fake-foreman screenshot capture script: `scripts/capture-macos-overlay.sh`.
- [x] Capture first visual artifact under `.ai/validation/macos-overlay/attention.png`.
- [x] Add `docs/macos-overlay/architecture.md`.
- [x] Add `docs/macos-overlay/ux-checklist.md`.
- [x] Add `docs/macos-overlay/validation.md`.
- [x] Add Swift unit tests for model/filtering/client decoding.
- [x] Add a hardened `ProcessRunner` that drains stdout/stderr asynchronously.
- [x] Add XCUITest or an app-level fake-foreman call-log assertion.
- [x] Add real tmux smoke script/task for control API focus/send.
- [x] Add initial mise task: `capture-macos-overlay`.
- [x] Add full mise tasks: `swift-build`, `swift-test`, and `verify-macos-overlay`.

## Slice 1 — Interaction Parity Foundation

- [x] Make normal typing route to search while compose is inactive.
- [x] Make `↑` / `↓` navigate the selected agent.
- [x] Make `Enter` focus the selected agent.
- [x] Make double-click focus the clicked agent.
- [x] Make `Cmd+R` refresh and `Esc` close.
- [x] Extract keyboard handling into testable overlay state/command reducer.
- [x] Make list scrolling follow selection when keyboard navigation moves past the visible rows.
- [x] Add visible focus regions: list, detail/preview, compose.
- [x] Add `Tab` / `Shift+Tab` cycling through focus regions.
- [x] Add footer hints that change by active region/mode.

## Slice 2 — Help and Scrollable Details

- [x] Add `?` help surface.
- [x] Add keyboard-scrollable help content for constrained window heights.
- [x] Add preview/detail keyboard scrolling when detail region is active.
- [x] Add help visual fixtures/captures for normal and scrolled help states.
- [x] Add tests for `?`, help scroll, and `Esc` close behavior.

## Slice 3 — App-Level Fake-Foreman Action Gauntlet

- [x] Add app-level fake-Foreman gauntlet harness.
- [x] Prove launch calls `agents --json`.
- [x] Prove synthesized double-click/row event path calls `focus --pane <id> --json` through gauntlet focus log.
- [x] Prove `Enter` calls `focus --pane <id> --json` through posted app key event.
- [x] Prove compose typing plus `Cmd+Enter` calls `send --pane <id> --stdin --json` with expected stdin.
- [x] Prove refresh calls `agents --json` again.
- [x] Add `mise run macos-overlay-gauntlet` and fold UI XCTest gauntlet into `verify-macos-overlay`.
- [x] Add `ForemanOverlayUITests` SwiftPM XCTest target for local UI event gauntlet.

## Slice 4 — Visual State Fixture Gauntlet

- [x] Split reusable SwiftUI overlay views into `ForemanOverlayUI`.
- [x] Add headless snapshot executable `foreman-overlay-snapshot`.
- [x] Add `mise run render-macos-overlay-snapshots`.
- [x] Fold headless snapshots into `verify-macos-overlay`.
- [x] Add `agents-empty.json` fixture.
- [x] Add fake command failure/error fixture mode.
- [x] Add `agents-long-path.json` fixture.
- [x] Add `agents-long-preview.json` fixture.
- [x] Add `agents-many.json` fixture.
- [x] Add `agents-mixed-status.json` fixture.
- [x] Extend capture script to capture one named state or all states.
- [x] Make `verify-macos-overlay` assert expected screenshot artifacts exist.

## Slice 4.5 — PR Link and Themes

- [x] Add optional pull-request metadata to control API entries via `foreman agents --json --pull-requests`.
- [x] Make Swift client request pull-request metadata.
- [x] Add PR detail panel with clickable `Open PR` link.
- [x] Add deterministic PR fixture data and OCR assertion for `Pull Request` / `Open PR`.
- [x] Change default hotkey to `Ctrl+Option+Shift+A`.
- [x] Add lightweight overlay themes and `Cmd+T` cycling.
- [x] Add theme snapshots for Indigo and Terminal.

## Slice 5 — Foreman Feature-Parity Matrix

- [x] Draft gauntlet/parity plan in `.ai/plans/2026-05-03-171708-macos-overlay-prototype/GAUNTLET.md`.
- [x] Promote stable parity matrix into `docs/macos-overlay/validation.md`.
- [ ] Decide whether overlay should expose all-panes/non-agent visibility.
- [ ] Decide whether overlay should expose harness/provider filter.
- [x] Explicitly defer or scope PR panel, spawn, rename, kill, notifications.

## Slice 6 — Configurable Hotkey and Menu-Bar Polish

- [x] Add environment-configurable hotkey parsing via `FOREMAN_OVERLAY_HOTKEY`.
- [x] Replace local parser with `KeyboardShortcuts` package recorder for normal persisted hotkey configuration.
- [x] Set default hotkey to `Ctrl+Option+Shift+A`.
- [x] Add settings surface for hotkey.
- [x] Improve menu bar item label/icon to show the configured hotkey label.
- [x] Add stable menu item names and shortcuts.

## Slice 7 — Terminal Activation Adapter

- [x] Decide initial terminal foreground activation behavior after focus: default `none`, optional configured app activation.
- [x] Add `TerminalActivating` adapter boundary.
- [x] Support initial modes: none and configured terminal activation.
- [x] Keep AppleScript/AppKit activation out of SwiftUI views.
- [x] Add gauntlet proof that missing configured terminal activation is non-fatal.
- [x] Add manual Mac smoke notes for supported terminals.
