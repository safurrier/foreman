---
id: docs-index
title: Foreman docs
description: >
  Human-facing index for foreman's durable docs, ordered from onboarding to
  deeper design references.
index:
  - id: start-here
  - id: operations
  - id: understand-the-system
  - id: decisions
---

# Foreman docs

## Start here

- [Repo Tour](tour.md)—first read for onboarding, reading order, and the code map
- [Workflow Guide](workflows.md)—HK lifecycle, validation layers, unattended setup/resume receipts, and common rough edges

## Operations

- [Operator Guide](operator-guide.md)—install, setup, dashboard, config, hooks, notifications, and troubleshooting
- [Harness Kit Provider](providers/harness-kit.md)—install and operate the read-only HK extension card provider
- [macOS App Bundle](macos-overlay/app-bundle.md)—build, install, launch, and smoke-test `Foreman.app`
- [macOS Overlay Validation](macos-overlay/validation.md)—required overlay change lane, snapshots, gauntlet, and manual smoke checks

## Understand the system

- [Architecture](architecture.md)—invariants, boundaries, and cross-cutting design
- [Project Evolution](project-evolution.md)—evidence-backed phases and retained boundaries
- [macOS Overlay Architecture](macos-overlay/architecture.md)—Swift modules, app-shell seams, hotkey routing, and control API boundaries
- [macOS Overlay UX Checklist](macos-overlay/ux-checklist.md)—command-palette and good-Mac-citizen UX expectations

## Decisions

- [Decision 0001—Stack choice](decisions/0001-stack-choice.md)—why the repo is Rust-first
- [Decision 0002—Source aggregation](decisions/0002-source-aggregation-and-remote-ssh.md)—source-scoped local and SSH inventory
- [Decision 0003—Remote focus](decisions/0003-remote-jump-terminal-activation.md)—tmux focus and machine-local display activation
- [Decision 0004—Source companion](decisions/0004-source-companion-relay.md)—companion, snapshot, tunnel, and display architecture
- [Decision 0005—Native provenance](decisions/0005-native-provenance-authority.md)—hook-native authority and compatibility fallback
