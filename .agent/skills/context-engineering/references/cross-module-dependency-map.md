# Cross-Module Dependency Mapping

For monorepos and multi-module projects, trace dependency chains to help agents
understand how modules interact. Knowing "if I change X, Y also breaks" prevents
cascading failures.

## When to Generate

- Repos with 3+ distinct modules/services/packages
- When explicit workspace manifests exist (workspace.toml, pnpm-workspace.yaml, etc.)
- When a user asks about cross-module concerns
- When the architecture doc identifies significant inter-module dependencies

## Discovery Strategy

### 1. Package Manifest Analysis

Find all package manifests and extract internal dependencies:

| Language | Manifest | What to look for |
|----------|----------|-----------------|
| Python | pyproject.toml, setup.py | `[project.dependencies]`, internal path deps |
| JS/TS | package.json | `dependencies`, `devDependencies`, workspace refs |
| Go | go.mod | `require` blocks with internal module paths |
| Rust | Cargo.toml | `[dependencies]`, workspace member refs |
| Java | pom.xml, build.gradle | Module dependencies |

For each module extract:
- Internal dependencies (other modules in the same repo)
- Key external dependencies that create coupling
- What it exports/provides to other modules

### 2. Import Analysis (supplement)

If manifests don't tell the full story:
- Grep for import patterns referencing other modules
- Check for shared config files or templates
- Look for shared test utilities or fixtures

### 3. Build the Dependency Graph

Map: which modules depend on which, with direction.

## Output Format

### Inline in AGENTS.md (fewer than 5 modules)

Add a "Module Dependencies" section:

```markdown
## Module Dependencies

`api` → `shared-lib` → `worker`
`frontend` → `shared-lib`

**Change impact:**
- Changes to `shared-lib/` affect `api`, `worker`, and `frontend`
- `frontend` and `worker` are independent of each other
```

### Separate doc (5+ modules)

Create `docs/cross-module-deps.md`:

```yaml
---
id: cross-module-deps
title: Cross-Module Dependencies
description: >
  Dependency graph and change impact matrix. Consult before making
  changes that span module boundaries.
index:
  - id: dependency-graph
    keywords: [deps, dependencies, graph, who-depends-on-whom]
  - id: change-impact
    keywords: [impact, cross-cutting, ripple, affected-modules]
---
```

### Change Impact Matrix

The most actionable output — a table agents can reference before making changes:

```markdown
## Change Impact

| If you change... | Also test... | Why |
|-----------------|-------------|-----|
| `shared/types/` | All modules | Shared type definitions |
| `api/routes/` | `frontend/`, `tests/e2e/` | API contract changes |
| `db/migrations/` | `api/`, `worker/` | Schema dependency |
| `shared/config/` | All modules | Shared configuration |
```

## Guidelines

- Focus on internal dependencies, not external libraries
- Trace the full chain (A → B → C), not just direct deps
- Include shared resources: configs, types, utilities used by multiple modules
- Keep it stable — dependency graphs change less often than code
- No fast-changing specifics (don't count number of shared functions, etc.)
