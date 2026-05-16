---
id: docs-index
title: foreman Docs
description: >
  Human-facing index for foreman's durable docs, ordered from onboarding to
  deeper design references.
index:
  - id: start-here
  - id: operate-foreman
  - id: understand-the-system
  - id: decisions
---

# foreman Docs

## Start Here

- [Repo Tour](tour.md) — first read for onboarding, reading order, and the code map
- [Workflow Guide](workflows.md) — HK lifecycle, validation layers, and common rough edges

## Operate Foreman

- [Operator Guide](operator-guide.md) — install, setup, dashboard, config, hooks, notifications, and troubleshooting
- [Harness Kit Provider](providers/harness-kit.md) — install and operate the read-only HK extension card provider
- [macOS App Bundle](macos-overlay/app-bundle.md) — build, install, launch, and smoke-test `Foreman.app`
- [macOS Overlay Validation](macos-overlay/validation.md) — required overlay change lane, snapshots, gauntlet, and manual smoke checks

## Understand the System

- [Architecture](architecture.md) — invariants, boundaries, and cross-cutting design
- [macOS Overlay Architecture](macos-overlay/architecture.md) — Swift modules, app-shell seams, hotkey routing, and control API boundaries
- [macOS Overlay UX Checklist](macos-overlay/ux-checklist.md) — command-palette and good-Mac-citizen UX expectations

## Decisions

- [ADR 0001 — Stack Choice](decisions/0001-stack-choice.md) — why the repo is Rust-first
