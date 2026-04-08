# Documentation Update Analyzer

Detailed instructions for the **update** mode.

Maintain technical documentation synchronized with evolving codebases. Identify what documentation needs updating based on code changes and ensure accuracy.

---

## Task Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| `PROJECT_LOCATION` | Current working directory | Where to analyze |
| `CHANGE_SCOPE` | Auto-detect from git | What changes to consider |
| `UPDATE_THRESHOLD` | Significant changes only | Filter minor changes |
| `MODE` | PLAN | PLAN (report only) or AUTO (execute updates) |

---

## Analysis Process

### 1. Auto-Detect Code Changes

Automatically determine the scope by checking in order:

1. **Uncommitted changes**: If working directory has changes, analyze those
2. **Current branch**: Compare current branch against main/master
3. **Recent commits**: Last 10 commits on current branch if no clear base
4. **Time-based**: Last 7 days of commits as fallback

**Git commands for detection**:

```bash
# Check for uncommitted changes
git status --porcelain

# If on feature branch, compare with main
git diff main...HEAD --name-status

# Get recent commits on current branch
git log --oneline -10

# Show actual diffs for analysis
git diff main...HEAD
```

Use these to identify:
- Modified files and their diffs
- New files added
- Deleted files
- Moved/renamed files

### 2. Categorize Changes

Review detected changes and classify by type:

| Category | Icon | Description | Doc Impact |
|----------|------|-------------|------------|
| **Breaking** | 🔴 | API changes, removed features, changed behaviors | Critical - must update |
| **Feature** | 🟡 | New functionality, endpoints, options | Important - should update |
| **Enhancement** | 🟢 | Performance, UI/UX improvements | Minor - optional update |
| **Fix** | - | Bug fixes affecting documented behavior | Case-by-case |

### 2.5. Map Change Categories to Doc Impact

For each change category, determine which doc type is affected:

| Change Category | Likely Doc Type Impact |
|-----------------|----------------------|
| API change (breaking) | reference docs (interfaces, types) |
| New feature | how-to or tutorial (usage guide) |
| Architecture change | explanation docs (architecture, design) |
| New tool/dependency | reference (setup, configuration) |
| Workflow change | how-to (task guides, runbooks) |

From this mapping, identify which specific docs need updating, creating, or archiving.

### 3. Map Changes to Documentation

For each significant change, identify:

- **Affected Files**: Which code files were modified
- **Documentation Impact**: Which docs reference this code
- **Update Type**: Required changes (update, add, remove)
- **Specific Locations**: Line numbers or sections needing attention

### 4. Detect Documentation Gaps

Identify:
- New features lacking documentation
- Outdated examples or commands
- Missing configuration options
- Deprecated functionality still documented
- Commands that no longer work

### 5. Generate Update Plan

**High Priority Updates (Breaking/Security)**:
```markdown
File: README.md
Section: Installation
Change: New required dependency added
Action: Add `pip install newpackage` to installation steps
```

**Medium Priority Updates (New Features)**:
```markdown
File: ARCHITECTURE.md
Section: API Endpoints
Change: New /api/v2/users endpoint added
Action: Document endpoint, parameters, and response format
```

**Low Priority Updates (Enhancements)**:
```markdown
File: CONTRIBUTING.md
Section: Testing
Change: New test utilities added
Action: Add examples of using new test helpers
```

### 6. Create New Documentation

For undocumented additions:
- **Document Name**: Where it should be created
- **Content Outline**: What it should cover
- **Priority**: When it should be written

---

## Output Format

### MODE: PLAN (Default)

```markdown
## Documentation Update Report

### Summary
- X files need updates
- Y new documents recommended
- Z deprecated sections to remove

### Priority Actions

#### 🔴 Critical Updates
[List breaking changes requiring immediate documentation updates]

#### 🟡 Important Updates
[List new features needing documentation]

#### 🟢 Minor Updates
[List enhancements and fixes to document]

### Detailed Changes

#### File: [path/to/doc.md]
**Section**: [Section name]
**Change**: [What code changed]
**Action**: [Specific update needed]
**Lines**: [Line numbers if applicable]

[Repeat for each file...]

### New Documentation Needed
[List any new docs that should be created]

### Questions
[Any clarifications needed before proceeding]
```

### MODE: AUTO

```markdown
## Documentation Update Report

### Summary
- X files updated automatically
- Y new documents created
- Z updates skipped (minor priority)

### Executed Updates

#### ✅ Completed Updates
[List of files modified with brief description of changes]

#### ⚠️ Manual Review Needed
[Any updates that couldn't be automated]

#### 📋 Skipped Minor Updates
[List of minor updates for future consideration]

### Change Log
[Show diffs for each modified file]
```

---

## Execution Options

### MODE: PLAN (Default)

After analysis:
1. Present the complete update report
2. Offer to show detailed instructions for each file
3. Ask which updates to proceed with based on priority
4. Generate updated content for selected sections upon request

### MODE: AUTO

After analysis:
1. Present the update report summary
2. Automatically execute all 🔴 Critical and 🟡 Important updates
3. Show a diff of changes made to each file
4. List any 🟢 Minor updates that were skipped
5. Report any updates that failed or need manual intervention

**AUTO Mode Behavior**:
- Updates existing files in place
- Creates new files when needed
- Preserves file formatting and structure
- Shows before/after comparisons
- **Stops on any errors to prevent corruption**

---

## Best Practices

**When mapping changes to docs**:
- Search for function/class names in doc files
- Check for example code that uses changed APIs
- Look for configuration examples that may be affected
- Consider both user-facing and developer-facing docs

**When writing updates**:
- Match the existing doc's tone and style
- Keep changes minimal and focused
- Preserve surrounding context
- Add migration notes for breaking changes

**When uncertain**:
- Mark as "needs review" rather than guessing
- Ask the user for clarification
- Err on the side of flagging potential issues

**After executing updates**:
- Run the verification script (see SKILL.md Verification section for invocation) to check internal links, frontmatter presence, and index completeness
- Fix any verification failures before reporting results
