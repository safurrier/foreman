# Specification — release-hygiene-final-pass

## Problem

Foreman is close to a 1.0 release, but three operator-trust gaps and one release-hygiene gap remain:

- session and window rows do not consistently use the resolved actionable pane for notification suppression and repo-scoped behavior
- validation evidence still depends on dated plan paths instead of a stable output root
- degraded tmux preview capture is counted globally but not surfaced per pane
- the repo still needs an explicit final scrub for stray benchmark-name references

## Requirements

### MUST

- Make notifications use the resolved actionable pane, not just literal pane-row selection.
- Make PR lookup and runtime diagnostics use actionable pane workspace identity for session and window rows.
- Preserve aggregate row metadata for the sidebar without overloading it for repo-scoped behavior.
- Move validation evidence outputs to a stable location that CI and release workflows consume.
- Fail validation when expected evidence is missing instead of ignoring missing files.
- Attribute capture failures to pane ids in logs.
- Surface selected-pane preview provenance in the preview panel.
- Keep degraded preview behavior soft-fail: panes remain visible and usable even if capture fails.
- Remove any stray textual benchmark-name references from the repo.
- Run the full validation ladder, including strict real native E2E.

### SHOULD

- Keep the preview provenance model small and semantic.
- Reuse the new stable evidence root for local workflow docs as well as CI/release.
- Add direct regression tests for session-row and window-row actionable-pane semantics.

## Constraints

- Do not regress the denser sidebar or virtualized navigation work.
- Do not revert existing unrelated dirty-tree changes.
- Keep artifact-path migration simple: stable root is preferred unless a manifest is clearly lower risk.
- Avoid coupling preview provenance to tmux backend internals more than needed.
