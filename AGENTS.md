# foreman

TUI for managing AI agents across tmux

## WHY

<!-- Describe the problem this project solves in 2-3 sentences -->

**Done means**: all `mise run check` gates pass; intended behavior is covered by tests.

Correctness invariants live in [`SPEC.md`](SPEC.md). System design and decisions in [`docs/architecture.md`](docs/architecture.md).

## WHAT

```
foreman/
├── src/                    # Source code
│   ├── main.rs             # Entry point
│   └── lib.rs              # Library with examples
├── .mise.toml              # Task runner config
├── Cargo.toml              # Package manifest
├── rustfmt.toml            # Formatter config
├── Dockerfile              # Multi-stage build
└── README.md
```

Key steering files:
- `AGENTS.md` — this file (steering index)
- `SPEC.md` — correctness envelope (requirements, contracts, invariants)
- `docs/architecture.md` — system description, principles, decisions
- `docs/decisions/` — architectural decision records (ADRs)
- `.agent/skills/` — optional opinionated workflow plugins

## HOW

```bash
mise run setup      # install tools and dependencies (one-time)
mise run check      # fast quality gate: fmt + lint + typecheck + test  ← before committing
mise run verify     # heavier validation: integration, security, docker  ← before merging
mise run dev        # start local development```

CI calls `mise run ci` (= `check`). Pre-commit hooks mirror CI exactly.
If `check` passes locally, CI passes.

## Stack: rust

- Formatter: cargo fmt (rustfmt)
- Linter: cargo clippy (deny warnings)
- Type checker: cargo check (compiler type + borrow analysis)
- Tests: cargo test

## Starting Work

```bash
git checkout -b feat/<slug>
mise run plan -- <slug>    # creates .ai/plans/YYYY-MM-DD-HHmmSS-<slug>/
```

`mise run plan` refuses to run on the default branch. Slugs must be lowercase
kebab-case and unique within `.ai/plans/`.

The task scaffolds META.yaml, TODO.md, LEARNING_LOG.md, and VALIDATION.md.
Add SPEC.md or IMPLEMENTATION.md if the work is complex.

See `.ai/plans/AGENTS.md` for the full plan structure and `_example/` for a reference.

## Before Pushing

1. `mise run check` — must pass
2. `/plan-sync` — verify plan META, TODOs, and logs are current
3. `/spec-sync` — review changes against SPEC.md, capture any decisions as ADRs
4. `/context-engineering update` — update AGENTS.md if workflows or structure changed
5. `/docs-workflow update` — update docs/ if architecture or module docs affected

Steps 2-5 are fast when nothing changed. The agent reviews its own diff and skips
if no updates are needed.

## Skills

Optional workflow plugins live in `.agent/skills/`. Each `SKILL.md` describes when to
load it and what workflow it encodes. Skills are preference-based — canonical truth
stays in `docs/`.

`.claude/skills` is symlinked to `.agent/skills` for Claude Code discovery. Other
harnesses can create `<harness-config>/skills → ../.agent/skills` similarly.

## Further Reading

| Document | Purpose |
|---|---|
| `SPEC.md` | Correctness envelope — requirements, contracts, invariants |
| `.ai/plans/` | Plan directories for units of work (see `.ai/plans/AGENTS.md`) |
| `docs/architecture.md` | System description, principles, decisions |
| `docs/decisions/` | Architectural decision records |
| `README.md` | Human-oriented quick start |
