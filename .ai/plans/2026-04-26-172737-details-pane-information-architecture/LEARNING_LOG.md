---
id: plan-learning-log
title: Learning Log
description: Dev diary for this unit of work.
---

# Learning Log

## 2026-04-26

- Started from synced `main` after the `1.1.0` squash merge.
- UX review found the Details pane needed stronger section identity rather than another feature surface.

- Refactor kept the existing pane layout and command model, but made Details content stable: Activity, Startup cache, Selected target, Pull request, Diagnostics, Overview, Recent output.
- Release/runtime tests were intentionally updated away from brittle full labels such as `Target pane:` because the selected target section now owns that metadata.
- Local Docker remains blocked by the same Colima daemon issue; CI should provide the clean Docker verdict.
## PR loop

- Opened PR #4 for the details pane information architecture slice after local validation.
## Review iteration

- Codex review caught hard-coded Unicode in Details, Diagnostics still labeled Setup, and plan metadata status drift; fixed before merge.
## Claude native diagnosis

- `danger` only unsets `CLAUDECODE`; the Foreman hook bridge keys off `TMUX_PANE`, so that alias does not break native signals.
- Fresh interactive Claude panes do not write native signals until the first hook event, so Foreman can legitimately show compatibility on a just-opened prompt screen.
- Added deterministic tmux e2e coverage for the provider hook seam: tmux pane env -> hook binary -> native signal file -> Foreman native overlay.
