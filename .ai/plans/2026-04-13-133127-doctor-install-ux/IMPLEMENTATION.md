---
id: plan-implementation
title: Implementation Plan
description: >
  Step-by-step approach for this unit of work.
  Optional — create only when the approach isn't obvious.
---

# Implementation — doctor-install-ux

## Approach

- Build a reusable diagnosis layer first, then hang both CLI and TUI UX off the
  same findings. That avoids a one-off `doctor` command that drifts from the
  runtime view.
- Keep the first version conservative:
  - diagnose broadly
  - auto-fix only where writes are additive and easy to reason about
  - print snippets for risky configs instead of guessing
- Treat setup diagnosis as a separate concern from lifecycle classification.
  Pane status should remain about the agent. Setup drift should appear as a
  diagnostic reason and operator alert.

## Steps

1. Define the doctor model
   - Add a new module, likely `src/doctor.rs`, with:
     - `DoctorReport`
     - `DoctorFinding`
     - `DoctorSeverity`
     - `DoctorScope`
     - `DoctorFix`
   - Findings should be serializable so CLI JSON and tests share one contract.
   - Keep check functions pure where possible so tempdir-based tests stay cheap.

2. Add CLI plumbing
   - Decide between:
     - flat flags added to the current `Cli`, or
     - a narrow subcommand migration
   - Recommendation for v1: flat utility flags to minimize CLI churn.
   - Add:
     - `--doctor`
     - `--doctor-json`
     - `--doctor-repo <path>`
     - `--doctor-strict`
     - `--doctor-fix`
   - Extend utility-command handling in `src/cli.rs`.

3. Implement machine and provider checks
   - Machine checks:
     - resolved config path exists
     - log dir exists or is creatable
     - tmux exists
     - hook binaries are on `PATH`
   - Provider checks:
     - Claude: search known settings files and detect Foreman hook entries
     - Codex: check `.codex/hooks.json`, hook entries, version floor, feature flag
     - Pi: check `.pi/extensions/foreman.ts`
   - Runtime-adjacent checks:
     - latest log summaries show fallback with zero native applied
     - native dirs exist / contain recent files when expected
     - preview/log signatures like `hook error`

4. Add safe fix/scaffold flows
   - Codex:
     - create or merge `.codex/hooks.json`
   - Pi:
     - create `.pi/extensions/foreman.ts`
   - Claude:
     - phase 1: print exact snippet and target file suggestion
     - follow-up only after merge behavior is proven:
       - safe additive merge into supported settings file
       - backup or dry-run output before write
   - Every fix path needs:
     - `--dry-run`
     - idempotent re-run behavior
     - explicit reporting of written files

5. Reuse diagnosis in runtime/TUI
   - During bootstrap or refresh, synthesize a smaller runtime diagnosis view
     from:
     - overlay summaries
     - warnings
     - selected pane/provider
   - Add:
     - operator alert when native is broadly degraded
     - compact context explanation for compatibility mode
     - explicit “run doctor” hint
   - Avoid turning the TUI into a config editor. Link diagnosis to the CLI fix
     path instead.

6. Tighten install and first-run UX
   - After `mise run install-local`, print next steps:
     - `foreman --init-config`
     - `foreman --doctor --doctor-repo .`
   - Update README and workflow docs so binary install and native setup are
     clearly separate steps.
   - Add a troubleshooting section keyed by finding IDs, not just prose.

7. Validate in layers
   - Unit:
     - finding synthesis
     - repo detection
     - JSON output
   - Integration:
     - missing hook files
     - empty native dirs
     - overlay fallback summaries
     - hook-error preview signatures
   - End-to-end:
     - bootstrap/TUI rendering of doctor-derived hints
     - strict real native E2E rerun if hook behavior changes

## Phase breakdown

### Phase 1 - Doctor core

- New doctor model and machine/repo checks
- CLI human output and `--json`
- Unit and CLI tests

### Phase 2 - Fix scaffolds

- Safe Codex/Pi `--fix`
- Claude snippet/report path
- Tempdir mutation tests

### Phase 3 - Runtime diagnosis

- Bootstrap/runtime synthesis from doctor + overlay summaries
- Operator alert and context-panel hints
- Render and reducer tests

### Phase 4 - Install UX

- `install-local` next-step guidance
- README/workflow troubleshooting refresh
- Example screenshots or text fixtures

### Phase 5 - Hardening

- log/finding IDs
- stronger tests around hook-command failure signatures
- strict native rerun if any provider bridge behavior changed

## Open decisions

1. Whether to keep utility flags or introduce subcommands now
2. Which Claude settings file Foreman should consider canonical for scaffold
   output
3. Whether compatibility heuristics should explicitly suppress generic `error`
   for known hook-command failure strings once a setup diagnosis exists
