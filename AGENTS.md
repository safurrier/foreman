# foreman

TUI for managing AI agents across tmux

## WHY

Foreman gives the operator one keyboard-first surface for watching and
controlling AI harnesses running in tmux. Work here is done when the behavior
matches `SPEC.md`, the architecture stays honest, and both `mise run check` and
the intended test layer for the slice are green.

Correctness invariants live in [`SPEC.md`](SPEC.md). System design and durable
boundaries live in [`docs/architecture.md`](docs/architecture.md).

## WHAT

Key code paths:
- `src/app/` — commands, actions, reducer, state, and selection invariants
- `src/ui/` — pure Ratatui rendering
- `src/adapters/` — tmux transport and subprocess seams
- `src/integrations/` — harness recognition plus native-over-compatibility overlays
- `src/services/` — notifications, pull requests, logging, and system stats
- `tests/` — unit, contract, tmux smoke, runtime smoke, and opt-in real harness E2E
- `.ai/plans/` — task-local working memory, validation evidence, and retrospectives

Key steering files:
- `AGENTS.md` — this file
- `SPEC.md` — correctness envelope
- `docs/tour.md` — repo quickstart and reading order
- `docs/workflows.md` — plan, validation, and non-code rough edges
- `docs/architecture.md` — architecture record
- `.ai/plans/AGENTS.md` — plan artifact contract

## HOW

```bash
mise run setup      # install tools and dependencies
mise run check      # fast gate: fmt + lint + typecheck + test
mise run verify     # heavy gate: integration + release gauntlet + docker
mise run verify-release  # compiled-binary release-confidence walkthroughs
mise run native-preflight  # local readiness check for real Claude/Codex/Pi E2Es
mise run dev        # run the app locally
```

Normal workflow:
1. `git checkout -b feat/<slug>`
2. `mise run plan -- <slug>`
3. Keep the slice narrow and update the active plan as you work
4. Run `mise run check`
5. Before pushing: `/plan-sync`, `/spec-sync`, `/context-engineering update`, `/docs-workflow update`
6. Run `mise run verify` before merge
7. If the slice changes native hooks or real harness flows, run `mise run native-preflight`
   and then strict native verification before calling it done
8. Use `mise run verify-release` when you want the standalone release report artifact

CI mirrors `mise run ci` for the fast gate. Pull requests also run
`mise run verify`.

## Stack: rust

- Formatter: `cargo fmt`
- Linter: `cargo clippy -- -D warnings`
- Type checker: `cargo check`
- Tests: `cargo test`

## Gotchas

- **DO** use portable `sh` in tmux smoke tests. **NOT** `zsh`. **BECAUSE** GitHub
  Linux runners do not guarantee `zsh`, and panes can exit immediately.
- **DO** write native-signal fixtures atomically. **NOT** overwrite them
  in-place. **BECAUSE** partial reads create false compatibility fallback and
  flaky runtime tests.
- **DO** promote durable workflow lessons out of `.ai/plans/*` into `docs/` or
  `AGENTS.md`. **NOT** treat historical plan logs as canonical truth.
  **BECAUSE** plan artifacts are evidence for a slice, not long-term onboarding.
- **DO** check the active Docker context when `mise run verify` fails in the
  Docker phase. **NOT** assume the Rust app regressed first. **BECAUSE** the
  common failure mode here has been local Colima or Docker runtime state.
- **DO** treat strict native verification as part of done when you touch real
  harness or hook behavior. **NOT** count skip-only `mise run verify-native`
  runs as done. **BECAUSE** the real-provider E2Es are the only proof that
  native Claude/Codex/Pi wiring still works end to end.

## Further Reading

| Document | Purpose |
|---|---|
| [`docs/tour.md`](docs/tour.md) | First read for repo onboarding and code map |
| [`docs/workflows.md`](docs/workflows.md) | Durable workflow, validation ladder, and rough edges |
| [`docs/architecture.md`](docs/architecture.md) | System boundaries, invariants, and module map |
| [`.ai/plans/AGENTS.md`](.ai/plans/AGENTS.md) | Plan file contract and lifecycle |
| [`README.md`](README.md) | Human-oriented product quick start |
