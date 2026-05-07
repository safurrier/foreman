# Learning Log

- `mise run pr-preflight` should remain a lightweight human checklist rather than part of `mise run check`; it is useful for large PR closeout but too noisy for every small edit.
- Include untracked files in preflight category counts so new plan/task/docs files are visible, but ignore untracked `.pi/` while still failing if `.pi/` is staged.
- The macOS overlay verifier needs an explicit `swift build --product foreman-overlay` step before the UI gauntlet runs with `FOREMAN_OVERLAY_SKIP_BUILD=1`.
- Codex review caught that Cargo lockfile-only or non-version Cargo.toml changes would be misread as release bumps. Gate README/CHANGELOG checks on an actual package version difference from `origin/main`, not on any Cargo metadata file change.
