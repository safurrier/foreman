# macOS Overlay A+ Hardening

## Goal

Turn the B+ Swift macOS overlay prototype into a boringly-correct A-level native app by applying the code review recommendations in small, validated slices.

## Non-goals

- No broad feature expansion.
- No large rewrite without behavior-preserving tests.
- Do not touch unrelated `.pi/` files.

## Slices

1. **Hotkey reliability / Ctrl+F fix** — make shortcut registration truthful, live-updating, and lifecycle-safe.
2. **Native text input routing** — preserve normal Mac text editing while keeping type-anywhere search.
3. **Process execution hardening** — absolute path resolution, typed missing executable errors, hard timeout behavior, stderr/JSON coverage.
4. **Deterministic reloads** — cancel/ignore stale reloads and remove double reloads.
5. **Selection normalization** — keep visible list highlight and action target in sync after filters/search/sort.
6. **Core.swift module split** — split Core into focused files without behavior change.
7. **Validation lane promotion** — make the required overlay validation lane explicit and documented for macOS changes.
8. **Accessibility/performance cleanup** — add accessibility labels/non-color status signals and precompute row presentations/duplicate counts.

## Validation contract

Each slice must run at least:

```bash
swift test --package-path apps/macos-overlay
mise run verify-macos-overlay
```

Final completion must run:

```bash
mise run verify-macos-overlay
mise run install-macos-overlay-app
```
