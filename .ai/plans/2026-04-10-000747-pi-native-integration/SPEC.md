---
id: plan-spec
title: Task Specification
description: >
  Requirements and constraints for this unit of work.
  Optional — create only for complex or scoped work.
---

# Specification — pi-native-integration

## Problem

Foreman already supports Claude Code and Codex CLI as first-class native
integrations, but Pi is still absent even though the top-level product
direction is to add more harnesses with native status where practical. Pi needs
to join the supported harness set without weakening the existing architecture:
native status must stay outside the reducer, compatibility fallback must keep
working, and the slice must prove itself with a real Pi-binary E2E rather than
only synthetic signal files.

## Requirements

### MUST

- Add Pi as a recognized harness family in compatibility mode and native mode.
- Add typed config and runtime wiring for Pi integration preference and native
  signal directory selection.
- Ship a supported Pi companion bridge command that writes Foreman's shared
  per-pane native signal files.
- Use Pi lifecycle events to drive native `working` and `idle` states.
- Keep compatibility mode available when Pi native signals are missing or
  disabled.
- Add real tests for bridge behavior, bootstrap/native precedence, and an
  opt-in live Pi-binary E2E that proves dashboard input can produce a
  completion notification.
- Sync `SPEC.md`, `README.md`, and `docs/architecture.md` to the shipped Pi
  surface.

### SHOULD

- Keep the Pi-specific glue thin. The Rust side should own native signal
  semantics, while the Pi extension stays a small transport wrapper.
- Avoid overly broad Pi recognition tokens that would misclassify unrelated
  panes such as generic shells or panes that merely contain `api`.

## Constraints

- Do not move Pi transport details into the reducer or renderer.
- Do not require Foreman to launch or embed Pi directly; monitoring still begins
  from tmux discovery.
- Do not replace Claude/Codex behavior while adding Pi.
