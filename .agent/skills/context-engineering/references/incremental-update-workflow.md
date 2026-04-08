# Incremental Update Workflow

The update mode defaults to incremental updates rather than full bottom-up
reconstruction. This significantly reduces update time for typical repos.

## Algorithm

### Step 1: Assess Staleness

Run `scripts/check_staleness.py <path-to-AGENTS.md>` to get a JSON report with:
- Last update commit and date
- Number of commits since
- Changed files categorized by impact (HIGH / MEDIUM / LOW)
- Recommendation: `skip` | `partial-update` | `full-update` | `full-rebuild`

### Step 2: Act on Recommendation

| Recommendation | Action |
|---------------|--------|
| `skip` | Report "docs are current" and exit |
| `partial-update` | Update only affected sections (Step 3) |
| `full-update` | Review all sections, update stale ones (Step 3) |
| `full-rebuild` | Full scan like quick-start, preserving accurate existing content |

### Step 3: Targeted Section Updates

For each section of the existing AGENTS.md, check what changed:

| Section | Update when... |
|---------|---------------|
| WHY | README or project purpose changed |
| Repo Map | Directory structure changed (new/removed top-level dirs) |
| Commands | Package manifests, Makefiles, or CI workflows changed |
| Invariants | CI workflows, security configs, or build constraints changed |
| Gotchas | Common failure patterns changed (hard to detect — review if HIGH impact files changed) |
| Progressive disclosure | New AGENTS.md files created/removed, new docs/ files added |
| Frontmatter | Validate with `scripts/validate_frontmatter.py` — update if headings changed |

### Step 4: Diff-Based Editing

For each section that needs updating:
1. Read the current content
2. Discover the new reality (same discovery as quick-start, but only for that section)
3. Merge: keep accurate existing content, update stale parts, add missing parts
4. Do NOT rewrite sections that are still accurate

### Step 5: Validate

After updates:
1. Run `scripts/validate_frontmatter.py` — check all frontmatter is valid
2. Run `scripts/verify_references.py` — check all referenced paths exist

## Signals That Force Full Rebuild

Even in incremental mode, fall back to full scan if:
- The existing AGENTS.md has no recognizable structure (malformed or missing frontmatter)
- More than 50% of referenced paths no longer exist
- The repo has been significantly reorganized (many top-level dirs changed)
- User explicitly requests full rebuild
- `check_staleness.py` returns `full-rebuild`

## Report Format (Incremental)

In addition to the standard report, include:
- **Commits analyzed**: N commits since last doc update
- **Sections updated**: list of which sections changed and why
- **Sections unchanged**: list of sections that were still accurate
- **Validation results**: output of frontmatter and reference checks
