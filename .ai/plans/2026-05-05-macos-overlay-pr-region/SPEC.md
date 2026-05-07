---
id: macos-overlay-pr-region-spec
title: Spec — macOS Overlay PR Focus Region
---

# Specification — macOS Overlay PR Focus Region

## Problem

The overlay can render PR metadata and has an `Open PR` button. Keyboard support
is too implicit: Details currently opens the PR on Enter when a PR exists, but
there is no distinct tabbable PR section. That makes the PR card feel like static
detail metadata rather than an actionable region.

## Goal

Make the PR card an explicit keyboard focus region:

```text
List → Pull Request → Details → Compose
```

When the selected agent has no PR, skip the PR region:

```text
List → Details → Compose
```

## Behavior

- Tab moves into `Pull Request` only when the selected entry has PR metadata.
- Shift+Tab moves backward using the same dynamic region list.
- The footer chip shows `Region: PR` when focused.
- The PR card gets the same subtle active border as List/Details/Compose.
- Enter while `PR` is active opens the PR URL in the browser.
- Enter while `Details` is active focuses the pane; PR opening is no longer hidden
  behind the generic Details region.
- Clicking the PR card activates the PR region.
- The `Open PR` link button still works.

## Validation

- Unit tests cover region cycling with and without PR metadata.
- Unit tests cover Enter behavior in PR vs Details.
- Swift tests pass.
- Full macOS overlay verifier passes.
