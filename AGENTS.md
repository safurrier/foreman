# foreman

**When a user corrects you or gives repo-specific tribal knowledge, document it
in the closest `AGENTS.md` before continuing.**

Foreman is a Rust/Ratatui operator console for AI agents running in tmux. Treat
`SPEC.md` as the product contract, `docs/architecture.md` as the architecture
record, and `docs/workflows.md` as the validation/process guide.

## How to Work Here

For meaningful work, create a feature branch, run `mise run plan -- <slug>`, and
keep the active plan current as scope changes. Use the smallest validation layer
that proves the slice, then run `mise run check` before push and `mise run
verify` before merge or runtime/release-sensitive changes.

## Commands

**Setup**: `mise run setup`.

**Fast gate**: `mise run check`.

**Heavy gate**: `mise run verify`.

**Release confidence**: `mise run verify-release`.

**Native harness proof**: run `mise run native-preflight`, then require the real
Claude, Codex, and Pi E2Es with `mise run verify-native`.

**Local app**: `mise run dev`.

## Gotchas

- **DO** use portable `sh` in tmux smoke tests. **NOT** `zsh`. **BECAUSE**
  GitHub Linux runners do not guarantee `zsh`, and panes can exit immediately.

- **DO** write native-signal fixtures atomically. **NOT** overwrite them
  in-place. **BECAUSE** partial reads create false compatibility fallback and
  flaky runtime tests.

- **DO** promote recurring workflow lessons out of `.ai/plans/*` into `docs/`
  or `AGENTS.md`. **NOT** treat historical plan logs as canonical truth.
  **BECAUSE** plan artifacts are evidence for a slice, not long-term onboarding.

- **DO** commit structured `.ai/plans/` and `.ai/validation/` paths that the
  workflow depends on. **NOT** commit `.ai/handoffs/`, `.ai/research/`, or
  plan-local artifact scratch. **BECAUSE** only the structured plan and
  validation roots are durable repo context.

- **DO** check the active Docker context when `mise run verify` fails in the
  Docker phase. **NOT** assume the Rust app regressed first. **BECAUSE** the
  common failure mode here has been local Colima or Docker runtime state.

- **DO** treat strict native verification as part of done when touching real
  harness or hook behavior. **NOT** count skip-only `mise run verify-native`
  runs as done. **BECAUSE** the real-provider E2Es are the only proof that
  native Claude, Codex, and Pi wiring still works end to end.

- **DO** keep native integration status pure to provider hook/file signals.
  **NOT** promote native panes with terminal text heuristics. **BECAUSE**
  heuristics are intentionally compatibility behavior; mixing them into native
  provenance makes Foreman look precise while it is guessing.

## Related Context

| Path | What's there |
|---|---|
| `docs/tour.md` | First read, repo map, and daily loop |
| `docs/workflows.md` | Plan artifacts, validation ladder, and environment notes |
| `docs/architecture.md` | System boundaries, invariants, and module map |
| `.ai/plans/AGENTS.md` | Plan artifact contract and lifecycle |
| `README.md` | Human quickstart, install, dashboard keys, and status matrix |

<!-- generated-by: context-engineering@2.2.0 | last-updated: 2026-04-30 -->
