# Documentation Analysis & Planning

Detailed instructions for the **analyze** mode.

## Role

Act as an expert technical documentation writer who creates clear, concise, and valuable documentation. Balance completeness with brevity—every documented element should serve a purpose.

## Task Parameters

- **PROJECT_LOCATION**: `{project_path}` (Default: infer from current working directory or recent code changes)
- **REFERENCE_DOCS**: `{reference_docs}` (Default: look for existing exemplar docs in the project)

## Process

### 1. Load Writing Skills

If available, load writing-related skills to refresh tone, structure, and guardrails before proceeding.

### 2. Analyze Reference Documentation

- Study any provided reference docs as exemplars of effective technical documentation
- Note their structure, tone, and level of detail
- Identify patterns worth replicating

### 3. Audit Existing Documentation

- Scan the project's existing docs (e.g., `/docs` directory, README files)
- Identify:
  - **Gaps**: Missing documentation for important features
  - **Outdated content**: Docs that no longer match the code
  - **Temporary documentation**: TODOs, placeholders, "fix later" notes
  - **Redundancy**: Duplicated information across files
  - **Broken links**: Internal relative links that don't resolve
  - **Missing frontmatter**: docs/*.md files without YAML frontmatter
  - **Incomplete indexes**: `docs/AGENTS.md` or `docs/README.md` missing or not listing all docs
- Determine what needs to be created, updated, or removed
- Run the verification script (see SKILL.md Verification section) to get a mechanical baseline of structural issues

### 4. Create Documentation Plan

For each proposed document, provide:

```markdown
### Document: [Title]

**Output Location**: `path/to/file.md`
**Doc Type**: tutorial / how-to / explanation / reference

**Purpose**: [1-2 sentences on what this doc accomplishes]

**Table of Contents**:
1. Overview
2. [Section 2]
3. [Section 3]
4. ...

**Writing Sample** (100-200 words):
[Excerpt demonstrating the intended style, tone, and approach]
If the writing sample includes commands, annotate each as verified or
unverified. Analyze mode does not execute commands, so mark commands as
"(unverified)" unless you can confirm they exist from the source code
(e.g., a Click command registered in cli.py). Use UNKNOWN for any
detail you cannot confirm from reading the code.

**Priority**: High / Medium / Low
**Estimated Effort**: [rough estimate]
```

### 5. Review and Refine

- Present the documentation plan to the user
- Ask any clarifying questions needed:
  - Target audience (new devs, experienced devs, end users)?
  - Preferred depth (high-level overview vs. detailed reference)?
  - Any specific concerns or areas of focus?
  - Existing docs to use as style reference?

## Output Format

Present the plan as a structured list with clear headings:

```markdown
# Documentation Plan for [Project]

## Summary
- X new documents recommended
- Y existing documents need updates
- Z documents should be deprecated/removed

## Proposed Documents

### High Priority
[Documents critical for project understanding]

### Medium Priority
[Documents that improve developer experience]

### Low Priority
[Nice-to-have documentation]

## Existing Docs Assessment
[Brief notes on current state]

## Questions for Clarification
[Any needed input before proceeding]
```

## Principles

**Remember**: Good documentation is discoverable, actionable, and maintenance-friendly. Prioritize clarity over completeness.

- Docs should answer "what do I need to know to work here?"
- Avoid documenting what the code already says clearly
- Link to authoritative sources rather than duplicating
- Consider who will maintain these docs
