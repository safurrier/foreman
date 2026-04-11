# Foreman Retro Against `agent-scaffold`

Generated: 2026-04-11

## Scope

Reviewed:

- root product docs and workflow docs in this repo
- every plan under `.ai/plans/`, especially `LEARNING_LOG.md`
- the full local git history from `e70165c` through `feefce1`
- the scaffold templates and task contract in `../agent-scaffold`
- the initial generated Foreman state from commit `e70165c`

## Executive Summary

- The scaffold gave Foreman a useful outer loop on day 0: `mise run plan`, `check`, `verify`, and CI parity were already there. That reduced setup thrash and let the work start immediately. See `../agent-scaffold/templates/AGENTS.md.tmpl:24-34`, `../agent-scaffold/.mise/tasks/plan:96-144`, and `../agent-scaffold/.mise/tasks/verify:84-101`.
- The part that really made Foreman succeed was the product-specific layer added almost immediately after init: a real spec, a real architecture record, a chunked TDD plan, and a validation ladder built around tmux, compiled-binary smoke, release gauntlets, and opt-in real harness drills. See `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:12-33`, `.ai/plans/2026-04-08-160754-spec-driven-tdd/LEARNING_LOG.md:39-76`, `docs/workflows.md:31-47`, and `.mise/tasks/verify-release:1-91`.
- The biggest avoidable gap was that the scaffold started Foreman from very generic docs and a very generic Rust starter. The generated repo still looked like a tiny library-plus-binary template even though the target was a Ratatui + tmux + multi-harness TUI. See `../agent-scaffold/stacks/rust/src/main.rs.tmpl:1-3`, `../agent-scaffold/stacks/rust/src/lib.rs:1-35`, `../agent-scaffold/templates/SPEC.md.tmpl:22-85`, and commit `e70165c`.
- The next biggest avoidable gap was planning and validation shape. The stock plan contract only required `META.yaml`, `TODO.md`, `LEARNING_LOG.md`, and `VALIDATION.md`, and its optional templates are nearly empty. Foreman's first successful plan had to invent `EXPLORATION.md`, `TRACEABILITY.md`, and `CHUNK_SPECS.md` to get enough structure. See `../agent-scaffold/templates/.ai/plans/AGENTS.md:6-20`, `../agent-scaffold/templates/.ai/plans/_templates/SPEC.md:15-27`, `../agent-scaffold/templates/.ai/plans/_templates/IMPLEMENTATION.md:11-17`, and `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:30-33`.
- Not everything could have been one-shot. Several key discoveries required real empirical validation: blank detached alternate-screen capture, Codex hook behavior differing between non-interactive mode and tmux TTY, tmux startup races in CI, and shell portability issues. See `.ai/plans/2026-04-08-160754-spec-driven-tdd/LEARNING_LOG.md:183-193`, `.ai/plans/2026-04-09-221745-codex-hook-native/LEARNING_LOG.md:25-30`, and `.ai/plans/2026-04-10-091010-release-automation/LEARNING_LOG.md:51-98`.

## What Worked Well

### 1. The scaffold's outer contract was solid

Quote:

> `mise run check` ... `mise run verify` ... CI calls `mise run ci` (= `check`).  
> `../agent-scaffold/templates/AGENTS.md.tmpl:26-34`

Analysis:

- This gave the repo a deterministic command surface from the start.
- `mise run plan` already enforced branch discipline and stable slugs. See `../agent-scaffold/.mise/tasks/plan:110-144`.
- That matters because the git history is cleanly incremental instead of spending the first day inventing task runners or branch hygiene. The history moves from `e70165c` initialization to spec, app state, CLI, render shell, tmux inventory, runtime, native integrations, then validation/UX hardening.

### 2. The first major adaptation was correct: spec-first, then architecture, then vertical slices

Quote:

> "the safest path is spec-first and test-first rather than trying to wire tmux, Ratatui, and harness integrations all at once."  
> `.ai/plans/2026-04-08-160754-spec-driven-tdd/LEARNING_LOG.md:13-15`

Quote:

> "one user-facing behavior / one or more spec invariants / one dominant test layer"  
> `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:24-28`

Analysis:

- This was the decisive process win.
- The delivery order in the first real plan matched the final codebase shape well: state core, render shell, tmux inventory, status engine, direct actions, secondary services, then runtime closeout. See `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:51-66`.
- The current architecture record reflects the same structure cleanly in `src/app/`, `src/ui/`, `src/adapters/`, `src/integrations/`, and `src/services/`. See `AGENTS.md:17-24` and `docs/architecture.md:41-54`.

### 3. The repo converged on the right validation model

Quote:

> "add at least one real e2e-style path whenever the feature crosses a process or adapter boundary."  
> `.ai/plans/2026-04-08-160754-spec-driven-tdd/LEARNING_LOG.md:71-76`

Quote:

> `Release gauntlet | Compiled-binary operator walkthroughs with temporary tmux worlds plus a durable checklist/report artifact`  
> `docs/workflows.md:37-47`

Analysis:

- This is the strongest part of the final process.
- The repo did not stop at unit tests plus one smoke test. It ended with:
  - fake-backed contract tests
  - real tmux runtime smoke
  - compiled-binary release gauntlet
  - focused UX capture lane
  - opt-in real harness E2E
- Those lanes are encoded directly in the repo now. See `.mise/tasks/verify:109-125`, `.mise/tasks/verify-release:19-69`, `.mise/tasks/verify-native:11-41`, and `docs/workflows.md:48-112`.

### 4. Durable lessons did get promoted out of plan logs

Quote:

> "The plan logs were the right place to mine for repeated failures ... But those artifacts should stay historical. The stable repo guidance belongs in `AGENTS.md` and `docs/`."  
> `.ai/plans/2026-04-10-100544-context-improve-sync/LEARNING_LOG.md:17-23`

Analysis:

- That promotion loop worked.
- The final repo now carries tmux shell portability, atomic native-signal writes, Docker-context guidance, and validation-lane structure in durable docs. See `AGENTS.md:63-75`, `docs/tour.md:53-60`, and `docs/workflows.md:114-148`.

## Where The Start Was Lacking

### 1. The generated repo context was too generic for the product

Quote:

> `<!-- Describe the problem this project solves in 2-3 sentences -->`  
> `../agent-scaffold/templates/AGENTS.md.tmpl:5-11`

Quote:

> `<!-- What this project is, in one paragraph. -->`  
> `../agent-scaffold/templates/SPEC.md.tmpl:22-25`

Quote:

> `| <!-- name --> | <!-- purpose --> |`  
> `../agent-scaffold/templates/docs/architecture.md.tmpl:31-49`

Analysis:

- Foreman had to replace generic placeholders before implementation could safely proceed.
- The initial generated Foreman commit still carried these placeholders almost verbatim. See commit `e70165c`, especially `AGENTS.md:7`, `SPEC.md:24`, and `docs/architecture.md:33-49` from that commit.
- This created avoidable early churn. The first plan explicitly notes that the repo was "still a scaffold" and that `docs/architecture.md` was still mostly template content. See `.ai/plans/2026-04-08-160754-spec-driven-tdd/LEARNING_LOG.md:13-15` and `.ai/plans/2026-04-08-160754-spec-driven-tdd/SPEC.md:31-38`.

### 2. The architecture template was biased toward service backends, not a local TUI/harness app

Quote:

> `### Security & Auth`  
> `### Data & Migrations`  
> `correlation ID`  
> `Background jobs are idempotent and safe to retry.`  
> `../agent-scaffold/templates/docs/architecture.md.tmpl:69-91`

Analysis:

- Those sections are not wrong generically, but they do not pressure the author toward the seams Foreman actually needed:
  - pure render state
  - reducer vs effect boundary
  - tmux adapter boundary
  - native vs compatibility precedence
  - compiled-binary validation
- Foreman's final architecture doc had to replace that bias with TUI-specific boundaries and harness-specific trust boundaries. See `docs/architecture.md:45-54`, `docs/architecture.md:74-89`, and `docs/architecture.md:145-170`.

### 3. The stock plan templates were too thin to drive a complex product

Quote:

> Required files: `META.yaml`, `TODO.md`, `LEARNING_LOG.md`, `VALIDATION.md`  
> `../agent-scaffold/templates/.ai/plans/AGENTS.md:6-20`

Quote:

> `### MUST` / `-`  
> `../agent-scaffold/templates/.ai/plans/_templates/SPEC.md:15-27`

Quote:

> `## Steps` / `1.`  
> `../agent-scaffold/templates/.ai/plans/_templates/IMPLEMENTATION.md:11-17`

Analysis:

- That is enough for small, obvious slices.
- It is not enough for a spec-heavy multi-stage rebuild.
- Foreman's first serious plan immediately outgrew the template and added:
  - `EXPLORATION.md`
  - `TRACEABILITY.md`
  - `CHUNK_SPECS.md`
  - an explicit test ladder
  - explicit exit criteria per chunk
- See `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:30-33` and `.ai/plans/2026-04-08-160754-spec-driven-tdd/LEARNING_LOG.md:39-69`.

### 4. The Rust stack starter was far too small for the target class of app

Quote:

> `fn main() { println!("Hello from {{ project_name }}!"); }`  
> `../agent-scaffold/stacks/rust/src/main.rs.tmpl:1-3`

Quote:

> `Greeter`  
> `../agent-scaffold/stacks/rust/src/lib.rs:1-35`

Analysis:

- This is a reasonable minimal Rust starter.
- It is not a good base for a Ratatui + tmux + harness-integration product.
- Because the generated shape was so generic, the first real architecture move had to invent the module map, adapter seams, and runtime ownership model from scratch. See `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:113-144`.

### 5. The generic `verify` lane was too weak for harness engineering

Quote:

> `Phase 2: stack-specific verification`  
> Rust verify = `cargo test --all-features` + optional Docker build  
> `../agent-scaffold/.mise/tasks/verify:64-101`

Analysis:

- That is enough for a normal library or service repo.
- It was not enough for Foreman.
- Foreman needed three extra lanes before confidence felt real:
  - `verify-release` for compiled-binary operator walkthroughs and a report
  - `verify-ux` for focused runtime/visual evidence
  - `verify-native` for opt-in real external harness drills
- Those were added later and became core to the final validation story. See `.mise/tasks/verify:116-123`, `.mise/tasks/verify-release:30-69`, `.mise/tasks/verify-native:11-41`, and `docs/workflows.md:54-112`.

### 6. The scaffold did not give the team a durable place for workflow rough edges

Quote:

> `docs/architecture.md` is generated, but there is no generated `docs/workflows.md` or `docs/tour.md`.  
> Verified from `../agent-scaffold/templates/docs/`.

Analysis:

- Foreman eventually added both because plan logs kept surfacing repeatable workflow knowledge that did not belong in the top-level architecture doc.
- See `docs/tour.md:23-29` and `docs/workflows.md:17-29`.

## What Would Likely Have Enabled Near One-Shot Execution

This section is inference from the evidence above, not a claim that all later discoveries were avoidable.

### 1. Start with a filled product-specific spec and architecture, not placeholder docs

If Foreman had started with:

- the native-vs-compatibility model
- the supported harness rollout order
- the reducer/render/effect boundary
- the intended validation ladder

then the early rewrite cycle would have been much smaller. Compare `../agent-scaffold/templates/SPEC.md.tmpl:22-85` with the finished `SPEC.md:28-38`, `SPEC.md:110-135`, and `SPEC.md:243-250`.

### 2. Start with a stronger plan contract for complex slices

The winning plan shape was:

- problem statement
- requirements and constraints
- explicit delivery order
- dominant test layer per slice
- exit criteria
- traceability artifacts

That structure appears in the first real Foreman plan, not in the scaffold template. See `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:24-33`, `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:51-111`, and `.ai/plans/2026-04-08-160754-spec-driven-tdd/LEARNING_LOG.md:64-69`.

### 3. Start with harness-aware validation lanes already present

If `verify-release`, `verify-ux`, and `verify-native` had existed from the start, much of the late-stage validation invention would have become normal implementation work instead of follow-up process work. See `.ai/plans/2026-04-10-151735-release-validation-gauntlet/LEARNING_LOG.md:11-18` and `.ai/plans/2026-04-10-194614-post-gauntlet-followups/LEARNING_LOG.md:11-18`.

### 4. Start with tmux/harness fixture helpers and artifact conventions

The stable patterns were:

- shell portability via `sh`
- retrying transient tmux startup failures
- alternate-screen fallback
- atomic native-signal writes
- durable artifact directories and reports

Those ended up as first-class helpers and docs. See `tests/support/tmux.rs:47-72`, `tests/support/tmux.rs:214-242`, `tests/support/tmux.rs:275-320`, `tests/support/release.rs:84-92`, and `.mise/tasks/verify-release:14-18`.

### 5. Start with a repo tour and workflow guide

Foreman only stabilized once the repo had a durable place to store "how to work here" that was separate from product spec and architecture. See `docs/tour.md:23-29`, `docs/tour.md:53-60`, and `docs/workflows.md:114-148`.

## What Still Would Not Have Been One-Shot

These are the parts I do not think a better scaffold could have removed:

- Codex native behavior had a real environment-dependent nuance. `codex exec` emitted hooks reliably in non-interactive mode but not from a tmux TTY in local testing. See `.ai/plans/2026-04-09-221745-codex-hook-native/LEARNING_LOG.md:25-30`.
- Detached tmux alternate-screen capture was empirically flaky and required a fallback in the fixture. See `.ai/plans/2026-04-08-160754-spec-driven-tdd/LEARNING_LOG.md:183-190` and `tests/support/tmux.rs:219-242`.
- CI exposed real portability and race issues that were hard to know in advance: shell assumptions, window-index assumptions, and transient tmux startup failure. See `.ai/plans/2026-04-10-091010-release-automation/LEARNING_LOG.md:51-98`.
- The UX layer still needed real operator feedback to reveal trust gaps that isolated tests did not surface. See `.ai/plans/2026-04-10-151735-release-validation-gauntlet/LEARNING_LOG.md:11-24`.

So: a better scaffold could have removed a lot of avoidable setup and process churn, but it could not have replaced empirical harness and tmux validation.

## Recommended `agent-scaffold` Improvements

### 1. Add a repo archetype for harnessed TUIs

Instead of only stack + shape, add an optional archetype such as:

- `archetype = "tui-harness"`

That archetype should generate:

- a filled AGENTS/SPEC/architecture package biased toward local TUI products
- `src/app`, `src/ui`, `src/adapters`, `src/integrations`, `src/services`
- a binary-first Rust layout instead of `Greeter`
- a `docs/workflows.md` and `docs/tour.md`

Why: Foreman's final stable architecture is archetype-specific, not just Rust-specific. See `AGENTS.md:17-32` and `docs/architecture.md:41-54`.

### 2. Upgrade the plan template for "complex" mode

Keep the minimal template for small work, but add a scaffold-supported complex variant that requires:

- `SPEC.md`
- `IMPLEMENTATION.md`
- `TRACEABILITY.md`
- declared dominant validation layer
- "what would make this one-shot next time?" prompt in `LEARNING_LOG.md`
- "what got promoted to durable docs?" prompt in `VALIDATION.md` or `LEARNING_LOG.md`

Why: Foreman's first real plan had to invent this anyway. See `.ai/plans/2026-04-08-160754-spec-driven-tdd/IMPLEMENTATION.md:30-33`.

### 3. Add first-class validation-lane extensions

Keep generic `verify`, but let archetypes register optional named lanes such as:

- `verify-runtime`
- `verify-ux`
- `verify-release`
- `verify-real`

Also allow these lanes to publish report/artifact paths in one standard place.

Why: the final Foreman process depends on named heavy lanes, not a monolithic `verify`. See `.mise/tasks/verify:116-123` and `docs/workflows.md:48-112`.

### 4. Add harness-fixture support libraries for the archetype

For `tui-harness`, scaffold:

- tmux fixture wrapper with retry and capture fallback
- release harness helper with temp repos, fake CLIs, and atomic native-signal writes
- artifact directory conventions under `.ai/`

Why: these helpers were not accidental polish; they were required to make the validation story stable. See `tests/support/tmux.rs:275-320` and `tests/support/release.rs:84-92`.

### 5. Add native-bridge companion-binary stubs and packaging hooks

For harness-aware repos, scaffold optional companion binaries and release packaging slots from the start.

Why:

- Foreman eventually needed `foreman-claude-hook`, `foreman-codex-hook`, and `foreman-pi-hook`.
- Release automation later had to learn that shipping only the main binary was incomplete.

See `.ai/plans/2026-04-10-091010-release-automation/LEARNING_LOG.md:18-23` and `docs/workflows.md:146-147`.

### 6. Add workflow-doc templates, not just architecture

Generate:

- `docs/tour.md`
- `docs/workflows.md`

with sections for:

- read order
- validation ladder
- environment rough edges
- promotion rule for plan lessons

Why: Foreman needed both to keep plan logs from becoming the only source of truth. See `docs/tour.md:53-60` and `AGENTS.md:70-75`.

### 7. Add environment-hardening defaults where possible

Two generic improvements look worth upstreaming:

- build Docker with a temporary clean `DOCKER_CONFIG` when the goal is public-image validation
- ship a stack-appropriate `.dockerignore` when a Dockerfile is scaffolded

Why: these were avoidable heavy-lane friction points. See `.ai/plans/2026-04-09-152615-full-spec-gap-closure/LEARNING_LOG.md:11-28` and `.mise/tasks/verify:61-89`.

## Bottom Line

The scaffold did not fail Foreman. It gave a good outer loop.

But Foreman only became efficient once the repo stopped behaving like "a Rust project generated from a generic template" and started behaving like "a harnessed tmux TUI with explicit architecture and explicit validation lanes."

The shortest path to better one-shot outcomes is not "make the generic scaffold smarter in the abstract." It is:

1. add an archetype for harnessed TUIs
2. strengthen the complex-plan template
3. scaffold named validation lanes and artifact/report conventions
4. scaffold workflow docs and harness fixtures, not just task runners

