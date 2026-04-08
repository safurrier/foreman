# Existing Docs Migration Workflow

When a repo already has documentation — in any directory — treat it as a **migration
source**, not a cleanup target. This workflow governs how to read, classify, migrate,
and report on pre-existing content before touching any files.

## When This Applies

- Any non-standard doc directory: `ai_agent_docs/`, `agent-docs/`, `context/`, `llm-docs/`, etc.
- Existing `docs/` files that pre-date the skill (no skill-generated frontmatter)
- Existing AGENTS.md content that may contain module-specific detail worth preserving

## Step 1: Discovery

Find all pre-existing documentation:

```
1. Glob for non-standard doc dirs: ai_agent_docs/, agent-docs/, context/, llm-docs/
2. Glob for docs/*.md files without YAML frontmatter (not skill-generated)
3. Read every file found — don't skip based on filename
4. Build an inventory table before doing anything else
```

Inventory format:
```
| File | Lines | Content type (initial guess) |
|------|-------|------------------------------|
| ai_agent_docs/editor-setup.md | 60 | Mixed: keybindings (human) + file map (agent) |
| ai_agent_docs/architecture.md | 80 | Architectural: system design |
```

## Step 2: Classify Each File

For each file, classify its content into one or more of these categories:

### Category A: Architectural / Cross-Cutting
*Belongs in `docs/{topic}.md` with frontmatter.*

Signals:
- System design, data flows, decision records
- Cross-module relationships
- Why something is built the way it is
- Migration history or upgrade paths

### Category B: Module Operational — Agent-Facing
*Belongs in `{module}/AGENTS.md`.*

Signals:
- File map of a config directory (agent needs to navigate)
- Commands specific to one module (agent needs to invoke)
- Scripts and when/why to run them (agent needs to operate)

**IMPORTANT**: Apply the audience test — does an AI agent need this to *work in the codebase*? Script purposes and file maps are agent-useful. Keybindings and shell aliases are typed by humans; agents don't need them in AGENTS.md.

### Category B2: Module Operational — Human-Facing
*Belongs in `docs/{topic}-reference.md` or a `README.md` alongside the config.*

Apply the **actor test**: *who does this?*

Signals:
- Interactive keyboard shortcuts for editors, terminals, or IDEs — humans press keys; agents don't
- Shell aliases and shorthand commands typed at a prompt — humans type them; agents invoke full commands
- UI or display preferences: color themes, pager config, prompt style — humans configure these
- Daily-use cheat sheets or personal workflow references

Preserve this content — it's stable and accurate. Route it to `docs/` where both humans and agents can find it on demand. Don't put it in AGENTS.md where it inflates agent context without benefit.

### Category C: Fast-Changing Specifics
*Discard (do not migrate).*

Signals:
- Test counts ("runs 159 tests")
- Timing data ("completes in 8.2s")
- Line counts, file counts, module counts
- Version-specific details that will immediately go stale

### Category D: Unverifiable
*Keep in place; do not delete; flag in report.*

Content is unverifiable when:
- Referenced paths or commands don't exist in the repo
- Claims can't be confirmed without running E2E/networked operations
- Content describes external services or environments not present locally

Unverifiable content should be **preserved at its current path** and reported. Do not
migrate it to a new location — it hasn't been validated yet. A human should decide
whether it's still accurate.

## Step 3: Verify Accuracy

For each Category A and B file:

1. **Path verification**: Every path mentioned in backticks — does it exist?
   ```
   file/path/mentioned.py → does it exist? ✅ / ❌
   ```
2. **Command spot-check**: Key commands — do the executables exist?
   - `which <command>` or check package.json scripts / Makefile targets / .mise.toml tasks
   - Do NOT run the commands themselves, just verify they're declared
3. **Key claim check**: For keybindings, confirm the config file exists and is non-empty
   (can't verify individual bindings without running the app, so accept on faith if config exists)

Mark each file: `✅ verified` / `⚠️ partially verified` / `❌ unverifiable`

If a file is `❌ unverifiable`: keep it at its current path, do not migrate or delete.

## Step 4: Build Migration Manifest

Before touching any files, produce a manifest:

```
Migration Manifest
==================
MIGRATE to docs/system-architecture.md:
  Source: ai_agent_docs/architecture.md (✅ verified)
  Content: system design, data flow, CLI commands

MIGRATE to src/payments/AGENTS.md:
  Source: ai_agent_docs/payments-module.md (✅ verified: dir exists)
  Content: file map, commands (agent-facing)

MIGRATE to docs/editor-reference.md:
  Source: ai_agent_docs/editor-setup.md (keybindings — human-facing, not AGENTS.md)

KEEP IN PLACE (unverifiable):
  ai_agent_docs/external-api-docs.md — references external service, can't verify

DISCARD:
  ai_agent_docs/old-setup.md — all referenced paths deleted, superseded by current toolchain
```

**Cross-module backlinks**: Before moving a cross-cutting doc (e.g., from `agent_docs/`
to `docs/`), grep for references to its current path across all AGENTS.md files in the
repo. Include files with inbound references in the manifest so their paths get updated
during migration.

**Do not proceed to file operations without completing this manifest.**

## Step 5: Execute Migration

In order:
1. Write all destination files (new AGENTS.md, docs/ files) from migrated content
2. Run validation (`validate_frontmatter.py`, `verify_references.py`) on new files
3. Only after validation passes: remove source files that were fully migrated
4. Never remove source files that had `❌ unverifiable` or `⚠️ partially verified` content
   unless every unverifiable piece was explicitly noted in the manifest

## Step 6: Report

The report must include the migration manifest with outcomes:

```
Pre-Existing Docs
-----------------
Read: 5 files in ai_agent_docs/

Migrated (3):
  ai_agent_docs/architecture.md → docs/system-architecture.md ✅
  ai_agent_docs/payments-module.md → src/payments/AGENTS.md ✅ (file map + commands, agent-facing)
  ai_agent_docs/editor-setup.md → docs/editor-reference.md ✅ (keybindings, human-facing)

Kept in place (1):
  ai_agent_docs/external-api-docs.md — unverifiable (references external service)

Discarded (1):
  ai_agent_docs/old-setup.md — all referenced paths deleted; superseded by current toolchain
  Discarded content: bootstrap steps for a removed Docker Compose workflow

Removed source dir: ai_agent_docs/ (after all migratable content confirmed)
```

## Content Merging Rules

When migrating into a file that already exists:

1. Read the destination file first
2. Identify content in the source that is NOT already present
3. Merge: add missing content, keep existing content, resolve conflicts by preferring
   what's verifiable
4. Do NOT overwrite destination content that was already accurate

When migrating agent-facing operational tables into a nested AGENTS.md:
- Preserve the full table — do not summarize or truncate
- Apply the same validation markers (✅/⚠️/⏸️) to any commands listed

When migrating human-facing content (interactive shortcuts, aliases, display preferences):
- Route to `docs/{topic}-reference.md`, not AGENTS.md
- Preserve the full table — do not summarize or truncate
