---
id: foreman-architecture
title: foreman Architecture
description: >
  Durable system record for foreman: invariants, principles, cross-cutting
  workflows, and architectural decisions. Read before implementing.
index:
  - id: system-overview
    keywords: [components, data-flows, trust-boundaries, modules]
  - id: goals-non-goals
    keywords: [goals, non-goals, scope]
  - id: invariants-boundaries
    keywords: [invariants, state, worktree-safety, observability, failure-modes]
  - id: principles-patterns
    keywords: [commands, reducer, effects, patterns, principles]
  - id: cross-cutting-workflows
    keywords: [validation-loop, check, verify, artifact-debugging, ci-failure]
  - id: decisions
    keywords: [adrs, decisions, truth-hierarchy, decisions-dir]
  - id: module-map
    keywords: [modules, boundaries, package-map]
  - id: where-human-thought-goes
    keywords: [human-ownership, agent-ownership, promotion-rules]
---

# foreman - Architecture

> This document is the durable system of record for invariants, principles, and
> decisions. Keep it updated as the system evolves. Agents: read this before
> implementing.

## System Overview

**Stack**: rust

Foreman is a keyboard-first Ratatui dashboard for monitoring and controlling AI
coding agents running inside tmux. The product contract lives in `SPEC.md`. This
document records the architecture and the seams that let that contract stay
testable as the app grows.

### Components

| Component | Purpose |
|---|---|
| CLI + config | Parse startup flags, config path/init flows, runtime overrides, popup mode, and notification toggles |
| App state core | Own `Command`, `Action`, `Mode`, `Focus`, selection state, filters, sort mode, modal state, and reducer logic |
| Ratatui renderer | Render header, sidebar, preview, input, footer, help, and overlays from pure state |
| tmux adapter | Discover sessions/windows/panes, capture pane output, focus panes, send input, rename windows, create windows, and kill panes |
| Harness integrations | Detect supported harness families, translate compatibility signals, and overlay native signals such as Claude when available |
| Pull request service | Resolve pull request metadata for the selected workspace with caching and graceful degradation |
| Notification service | Apply suppression, cooldown, and profile rules and dispatch best-effort notifications |
| Logging + telemetry | Persist structured run logs, latest-run pointer, retention cleanup, and header-level system stats |

### Primary Data Flows

1. **Startup and boot**
   `foreman` parses config and overrides, starts logging, initializes adapters,
   builds initial state, and renders the empty or populated shell.
2. **Refresh and status loop**
   Poll tick triggers tmux discovery and pane capture, harness interpreters
   derive status signals, reducer updates state, and renderer redraws.
3. **Operator action loop**
   Key or mouse input maps to `Command`, then `Action`, then reducer state
   transition, then optional Effects through adapters, then redraw. Draft text
   and modal targets stay in reducer-owned state rather than widget-local state.
4. **Attention loop**
   Status transitions flow through notification policy, which decides whether to
   emit, suppress, or debounce a notification and logs the decision.

### Trust Boundaries

- **User input boundary**: CLI flags, config file contents, search text, rename
  text, and direct pane input are untrusted until parsed and validated.
- **tmux boundary**: session inventory, pane metadata, and captured terminal
  content come from an external process and may be stale, partial, or missing.
- **Harness boundary**: native integration signals are more structured but still
  external to Foreman and may disconnect or downgrade to compatibility mode.
- **Local tooling boundary**: Git, GitHub-aware commands, browser openers,
  clipboard tools, and notification backends are optional dependencies and must
  fail soft.

---

## Goals / Non-Goals

**Goals** - what this system optimizes for:

- Give the operator one keyboard-first control surface for multi-agent tmux work.
- Keep working, needs-attention, idle, error, and unknown states easy to
  distinguish at a glance.
- Prefer durable architecture seams over ad hoc shell logic in the UI layer.
- Make most behavior testable without requiring a live tmux server.
- Keep native integrations first-class while preserving compatibility-mode
  usefulness.

**Non-Goals** - explicit exclusions:

- Foreman is not a tmux replacement.
- Foreman is not a general-purpose plugin host.
- Foreman does not need to expose every low-level harness event in the main
  status model.
- Foreman does not include pull request mutation features such as review,
  merge, or comment.

---

## Invariants & Boundaries

> These rules prevent architectural drift. Violating them requires an ADR.

### State and rendering invariants

- `render = pure(state)`. Rendering does not mutate state, block on I/O, or
  derive hidden side effects.
- Exactly one focus target is active at a time.
- Only one high-priority mode consumes input at a time.
- Direct-input drafts and rename/spawn/kill modal targets live in `AppState`;
  widgets render them but do not own them.
- Search queries, flash prefixes, and their restore-selection targets also live
  in `AppState`; cancel behavior is reducer-owned rather than widget-local.
- Refreshes, filtering, and sorting must not leave selection pointing at an
  invalid target.
- Collapsed session state persists across refreshes.

### Integration and adapter invariants

- Native integration state wins over compatibility heuristics when both are
  available for the same agent.
- Compatibility heuristics are allowed to be lower confidence, but they must
  fail soft rather than hide the pane entirely.
- All tmux, GitHub, browser, clipboard, and notification effects flow through
  adapters or services, never the renderer or reducer.
- tmux action adapters return structured success/failure results rather than
  leaking subprocess text into reducer-facing code.
- Destructive pane actions require explicit confirmation.

### Persistence and runtime boundaries

- Config is user-scoped, typed, and safe to load with defaults.
- Logs are user-scoped, keep a latest-run pointer, and clean up old runs
  automatically.
- No reliance on absolute paths, mutable global state, or undeclared local
  artifacts.
- The app must remain usable as an empty shell when tmux or optional local
  tooling is unavailable.

### Worktree safety

- The crate builds and tests from a clean checkout or Git worktree.
- Generated artifacts are deterministic or ignored.
- Plan artifacts under `.ai/plans/` are documentation inputs, not runtime
  dependencies.

### Traceability and observability

- Run logs are structured and machine-parseable.
- Action failures that matter to the operator are visible in the UI and in logs.
- Pull request lookups, notification suppression decisions, and integration-mode
  selection are observable in logs.
- Header-level system stats give a quick local CPU and memory read without
  becoming a heavy telemetry subsystem.

---

## Principles & Patterns

> Durable preferences. If a rule is critical and objective, enforce it in CI.

- **Prefer commands over raw keys** - input maps to `Command`, then `Action`;
  business logic never matches raw keycodes.
- **Prefer a pure reducer with explicit Effects** - reducer owns state
  transitions; adapters own I/O.
- **Prefer fake-backed contract tests** - most tmux, pull request, and
  notification behavior should be validated behind seams before live smoke tests.
- **Prefer stable logical identity over index-based selection** - selection
  should survive refreshes and reordering whenever possible.
- **Prefer keyboard-first, instant interaction** - navigation should feel
  immediate, with no delay inserted between keypress and state change.
- **Prefer progressive disclosure** - header, sidebar, preview, and input stay
  primary; search, flash, help, rename, spawn, and PR detail remain secondary
  surfaces.
- **Prefer monochrome-safe status cues** - color may help, but labels and
  symbols must carry meaning on their own.
- **Prefer deterministic jump labels** - flash navigation labels should be
  stable for a given visible target set and avoid prefix ambiguity within one
  invocation.

---

## Cross-Cutting Workflows

### Validation Loop

CI defines "passing." Local commands mirror CI exactly:

```bash
mise run check    # fast: fmt + lint + typecheck + unit tests  (on push)
mise run verify   # heavy: integration, security, docker        (on PR)
```

### Test strategy by layer

| Layer | Primary purpose |
|---|---|
| Unit tests | Reducer logic, command/action mapping, config parsing, status debouncing, precedence rules |
| Ratatui buffer tests | Surface visibility, focus treatment, overlays, empty-state shell, monochrome-safe status presentation |
| Adapter contract tests | tmux, harness, pull request, and notification seams with fakes or spies |
| Real-environment smoke / E2E tests | End-to-end operator journeys that require a real CLI process, live tmux coordination, or other external tooling |

### Pre-push documentation and contract sync

Before pushing feature work:

1. `mise run check`
2. `/plan-sync`
3. `/spec-sync`
4. `/context-engineering update` when AGENTS routing changed
5. `/docs-workflow update` when docs changed

### Artifact-first debugging

When CI fails, inspect the uploaded `test-results/` artifact first and use it
to choose the narrowest reproducible local command.

---

## Decisions

Global architectural decisions live in `docs/decisions/`. Include only
decisions that materially constrain work.

**Truth hierarchy** (highest to lowest authority):

1. CI/tooling enforcement
2. ADRs in `docs/decisions/`
3. This document (`docs/architecture.md`)
4. Module-level docs and `AGENTS.md`

**Index**:

| ADR | Decision |
|---|---|
| [0001-stack-choice](decisions/0001-stack-choice.md) | Stack selection for foreman |

---

## Module Map

The codebase now has the core runtime seams below. Keep new work inside these
boundaries unless an ADR changes them.

| Module | Purpose | Docs |
|---|---|---|
| src/cli.rs | CLI flags, config path/init flows, runtime override parsing, and bootstrap wiring | `SPEC.md` |
| src/app/ | Core state, commands, actions, reducer, selectors, drafts, modal targets, and UI-facing invariants | This document |
| src/ui/ | Ratatui layout, widgets, rendering, and buffer-test helpers | This document |
| src/adapters/tmux.rs | tmux discovery, capture, focus, send-input, rename, spawn, and kill seam; transport only, no status heuristics | `SPEC.md` |
| src/integrations/ | Harness recognition, compatibility status derivation, debounce logic, and native-over-compatibility precedence overlays | `SPEC.md` |
| src/services/notifications.rs (planned) | Notification policy, cooldowns, backend dispatch | `SPEC.md` |
| src/services/pull_requests.rs (planned) | Pull request lookup, caching, degradation behavior | `SPEC.md` |
| src/services/logging.rs | Run logs, latest-run pointer, retention cleanup, and bootstrap/inventory summaries | `SPEC.md` |

---

## Where Human Thought Goes

**Humans define**:

- Invariants and boundaries in `SPEC.md` and this document
- Status semantics for supported harnesses
- Architectural decisions and ADRs
- What belongs in the fast quality gate versus heavy verification
- Which operator-facing trade-offs are acceptable when native and compatibility
  signals disagree

**Agents own**:

- Implementation that follows these constraints
- Adding the highest-leverage tests before wiring I/O
- Keeping plan, spec, and architecture docs in sync with meaningful behavior
  changes
- Completing the validation loop before committing

**Promotion rules**:

- Repeated ambiguity -> document here or in module docs
- Repeated objective failure -> encode in CI/tooling
- Repeated preference debate -> optional Skill plugin (`.agent/skills/`)
