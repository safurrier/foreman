---
id: tour
title: Repo Tour
description: >
  First read for working on foreman: what to read first, how the repo is laid
  out, and where durable workflow knowledge lives.
index:
  - id: purpose
  - id: read-order
  - id: repo-map
  - id: daily-loop
  - id: where-lessons-live
---

# Repo Tour

## Purpose

Foreman is a Rust/Ratatui dashboard for monitoring and controlling AI harnesses
running in tmux. The product contract lives in [`SPEC.md`](../SPEC.md), and the
durable implementation boundaries live in [`architecture.md`](architecture.md).

## Read Order

1. [`../SPEC.md`](../SPEC.md) for the correctness envelope
2. [`architecture.md`](architecture.md) for architectural boundaries
3. [`workflows.md`](workflows.md) for plan, validation, and environment rules
4. [`.ai/plans/AGENTS.md`](../.ai/plans/AGENTS.md) for the plan artifact contract

## Repo Map

| Path | Role |
|---|---|
| `src/app/` | Reducer-owned state, commands, actions, and selection/mode invariants |
| `src/ui/` | Pure Ratatui rendering |
| `src/adapters/` | tmux subprocess seams |
| `src/integrations/` | Harness detection plus native overlays |
| `src/services/` | Notifications, PRs, logging, and system stats |
| `tests/` | Unit, contract, tmux smoke, runtime smoke, and opt-in real harness E2E |
| `.ai/plans/` | Task-local specs, logs, validation history, and retrospectives |
| `.ai/validation/` | Stable UX and release evidence consumed by CI and release workflows |

## Daily Loop

1. Create or stay on the feature branch for the slice.
2. Run `mise run plan -- <slug>` for any meaningful unit of work.
3. Implement the smallest vertical slice that closes a spec or workflow gap.
4. Keep the active plan current: `META.yaml`, `TODO.md`, `LEARNING_LOG.md`, and
   `VALIDATION.md`.
5. Run `mise run check`.
6. Before pushing, sync plan/spec/context/docs.
7. Run `mise run verify` before merge.

## Where Lessons Live

- `.ai/plans/*` is working memory and evidence for a slice.
- `.ai/validation/*` is stable review and release proof.
- `.ai/handoffs/*` and `.ai/research/*` are scratch by default and should not
  become canonical repo context.
- `docs/*.md` is durable onboarding and reference material.
- `AGENTS.md` is the short steering file future sessions should read first.

When a lesson keeps recurring in plan logs or validation notes, promote it into
`docs/` or `AGENTS.md` rather than expecting future sessions to rediscover it.
