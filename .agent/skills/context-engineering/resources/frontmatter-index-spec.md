# Frontmatter Index Specification

Content docs (`docs/*.md`) get YAML frontmatter with a machine-readable index for
progressive disclosure. An agent reads frontmatter to decide whether to load the
full document, and can target specific sections by matching `index[].id` to `##`
headings.

## Contents
- Format (YAML frontmatter schema)
- Field definitions (id, title, description, index[])
- Examples (root AGENTS.md, nested module, docs/ file)
- How agents use frontmatter
- Validation

## Exceptions -- Files That Do NOT Get Frontmatter

- **AGENTS.md** (at any level): Routing indexes, not content docs. Agents auto-discover these by convention -- frontmatter adds no routing value.
- **README.md used as an index**: Navigation surfaces like `docs/README.md` may optionally have frontmatter, but it's not required.

Only content docs that agents need to decide whether to load require frontmatter.

## Format

```yaml
---
id: {kebab-case-unique-identifier}
title: {Human-readable title}
description: >
  1-2 sentence summary. Agents read this to decide whether to load the full doc.
index:
  - id: {section-id}
  - id: {section-id-2}
---
```

## Field Definitions

### id (required)

Globally unique within the repo. Format: `{scope}-{purpose}`.

| Doc type | Convention | Example |
|----------|-----------|---------|
| docs/ file | `{topic}` | `architecture`, `testing-guide` |

### title (required)

Human-readable document title. Usually matches the `# heading`.

### description (required)

1-2 sentences answering: "What will I learn from this document?"

**Good**: "Setup commands, file map, and deployment workflow for the API service."
**Bad**: "Various information about the API." (too vague to route on)

### index[] (required)

Array of section descriptors. Each maps to a `## heading` in the document body.

- **index[].id** (required): Kebab-case identifier matching a `## heading` in the doc.

**Heading depth**: `index[].id` entries correspond to `##` (h2) headings only.
Subsections at `###` or deeper are not indexed — they're discovered by reading
the parent `##` section. If a subsection is important enough to index, promote
it to `##` level.

## Examples

### docs/ Content File

```yaml
---
id: testing-guide
title: Testing Guide
description: >
  Test infrastructure, pytest configuration, marker conventions,
  and CI integration for the full test suite.
index:
  - id: test-structure
  - id: commands
  - id: markers
---
```

## How Agents Use Frontmatter

1. Agent encounters a task (e.g., "fix the API endpoint")
2. Agent reads frontmatter of available docs (~100 words each -- cheap)
3. Matches task against `id`, `title`, and `description` to determine relevance
4. Loads only the docs that are relevant
5. Within a doc, jumps to specific `##` sections by matching `index[].id`

This makes references like "See `docs/testing.md`" actionable -- the agent reads
frontmatter first and decides whether loading the full doc is worth the context cost.

## Validation

Run `scripts/validate_frontmatter.py` to verify:
- All docs/*.md content files have frontmatter (AGENTS.md files are excluded)
- Required fields present (id, title, description, index)
- `id` values unique across the repo
- `index[].id` values correspond to actual `##` headings
