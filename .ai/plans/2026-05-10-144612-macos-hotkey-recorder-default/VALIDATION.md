# Validation

| Command | Result |
|---|---|
| `cargo test -q adapters::tmux::tests::parse_pane_records` | passed |
| `swift test --package-path apps/macos-overlay` | passed |
| `mise run validate-macos-overlay-change` | passed |
| `mise run install-macos-overlay-app` | passed; installed app logged the restored default shortcut and loaded 13 visible entries |
| scripted macOS overlay gauntlet shortcut case | passed inside `mise run validate-macos-overlay-change`; logs showed clear (`—`) then restore to the default shortcut |
| docs workflow `docs_verify.py .` | passed |
| context engineering `validate_frontmatter.py .` | passed |
| context engineering `verify_references.py .` | passed |
| `mise run pr-preflight` | passed with expected HK/docs/context and macOS overlay reminders |
| `mise run plan -- hk-migration-demo` | failed intentionally after printing HK migration guidance |
| `hk validate --check focused-rust-tests --why ... -- cargo test -q adapters::tmux::tests::parse_pane_records_skips_malformed_lines` | passed |
| `hk validate --check macos-overlay-required-lane --why ... -- mise run validate-macos-overlay-change` | passed |
| `hk validate --check fast-gate --why ... -- mise run check` | passed |
| fresh-context reviewer subagent | found two blocking concerns; both fixed and recorded with `hk review add --review codex-review` |
| `hk ready --target .` | passed after sync with `.pi/` and unrelated provider exploration excluded |

Notes:
- Existing UI gauntlet now includes shortcut clear/restore coverage.
- Headless snapshots/OCR were regenerated for help/settings text showing `Cmd+Option+F`.
