---
id: plan-spec
title: Sidebar List Virtualization
description: >
  Narrow slice for making sidebar navigation render only visible rows while preserving
  stable selection and scrolling behavior.
---

# Spec

## Problem

The sidebar still renders all visible rows as a single `Paragraph`, even after caching the
derived visible tree. Large tmux inventories still feel slow because every frame builds and
lays out the entire row set.

## Goal

Make sidebar navigation render only the rows that fit in the sidebar viewport while keeping the
selected row visible across navigation, filtering, sorting, search, and session collapse/expand.

## Constraints

- Keep the slice narrow to the sidebar.
- Preserve current keyboard behavior and row content.
- Keep render output stable enough for existing dashboard/release tests.
- Run the full validation ladder, including strict native E2Es.

## Acceptance

- Sidebar rendering is `O(visible_rows)` rather than `O(total_rows)`.
- `j/k` selection keeps the selected row visible in small viewports.
- Search, sort, inventory refresh, and collapse/expand do not leave the sidebar viewport stale.
- Full validation passes:
  - `mise run check`
  - `mise run verify`
  - `mise run native-preflight`
  - strict `mise run verify-native`
