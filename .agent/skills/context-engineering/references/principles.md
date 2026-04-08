# Documentation Principles

Conceptual invariants that govern all documentation generation and maintenance.
These principles are the evaluation rubric — every skill decision should trace
back to one or more of these.

## The Principles

### P1. Intent-First Taxonomy

Classify every doc by what it helps the reader *do*, not by its topic. Use the
Intent classification compass (see `references/docs-structure.md`):

| If the content... | ...and serves the reader's... | ...then it's a... |
|---|---|---|
| informs action | acquisition of skill | **tutorial** |
| informs action | application of skill | **how-to** |
| informs cognition | application of skill | **reference** |
| informs cognition | acquisition of skill | **explanation** |

A doc that mixes types should be split. A doc that doesn't fit any type probably
shouldn't exist.

### P2. Canonical Truth, No Duplication

For any fact, task, or concept there is **one canonical doc**. Everything else
links to it. When you find duplicate content, consolidate — don't add a third copy.

Corollary: prefer `file:line` references over pasting code that will go stale.

### P3. Deterministic Verification Over Subjective Quality

If it can be machine-checked, check it mechanically:
- Internal links resolve
- Referenced paths exist in the repo
- Frontmatter is well-formed
- Docs indexes list all docs/ files
- Commands are validated (✅/⚠️/⏸️)

Subjective quality (tone, clarity, completeness) matters but is secondary to
structural correctness. Fix the mechanical failures first.

### P4. Generate Volatile Truth; Write Durable Explanation

Reference data that changes (test counts, timing, file counts, version numbers)
should be generated from source or omitted entirely. Human prose focuses on
invariants — things that are true across versions.

| Write this | Not this |
|-----------|---------|
| `uv run pytest tests/unit/ -v` — run unit tests | `runs 159 tests in 8.2s` |
| `mise run check` — fast quality gate | `completes in ~15s` |

### P5. Navigation Is a Product Surface

Discovery must be predictable. Every docs/ directory gets two indexes:
- `README.md` — human-oriented, ordered by learning progression
- `AGENTS.md` — agent-oriented, organized by doc type with frontmatter

These indexes are not optional decoration. They are the entry points that make
the rest of the documentation findable.

### P6. Sustainability Beats Completeness

Only create doc surface area you can keep green. A missing doc is better than a
stale doc — stale docs actively mislead. If you can't verify it, keep it small
and clearly scoped.

Sizing targets:
- Root AGENTS.md: < 150 lines, hard stop at ~300
- Nested AGENTS.md: < 100 lines
- docs/ files: < 100 lines, hard stop at ~150
- docs/ indexes: curated, not exhaustive

### P7. Audience Filter — The Actor Test

Before routing content, ask: *who does this?*

- **Agent does it** (navigates file maps, runs commands, follows invariants) → AGENTS.md
- **Human does it** (presses keyboard shortcuts, types aliases, configures UI) → `docs/`

The test is not about content type or durability. Stable, accurate content is
still human-facing if a human is the one who uses it.

### P8. Progressive Disclosure

AGENTS.md is a routing index, not an encyclopedia. Structure:

```
AGENTS.md (root)          → routing tables, key commands, invariants
  ├── nested AGENTS.md    → module-specific commands, file maps
  └── docs/               → depth on demand (tutorials, references, explanations)
       ├── AGENTS.md      → agent routing index for docs/
       └── README.md      → human progression index for docs/
```

An agent reads AGENTS.md first, decides what else to load, and loads only that.

### P9. Validate What You Claim

Every command, path, and reference in generated docs must be confirmed:
- **Commands**: validate existence (package.json scripts, Makefile targets, CI steps).
  Run non-destructive checks (`--help`, dry-run). Mark status ✅/⚠️/⏸️.
- **Paths**: confirm they exist in the repo. Run `verify_references.py`.
- **Frontmatter**: run `validate_frontmatter.py` after generation.

Never include a command you haven't at least confirmed is declared somewhere.

### P10. Stateless Onboarding

Assume zero prior knowledge. The files you write onboard future agent sessions
that know nothing about this repo. Every AGENTS.md must be self-contained enough
that a fresh agent can orient and start working from it alone.

---

## Applying the Principles

When making a documentation decision, trace it to a principle:

- "Should I include this keybinding table?" → P7 (actor test: human does it → docs/)
- "Should I list test counts?" → P4 (volatile truth → omit)
- "Should I add this to AGENTS.md or create a separate doc?" → P8 (progressive disclosure) + P6 (sustainability)
- "Should I copy this code snippet?" → P2 (canonical truth → link instead)
- "Is this docs/ directory organized well?" → P1 (intent-first taxonomy) + P5 (navigation surface)
