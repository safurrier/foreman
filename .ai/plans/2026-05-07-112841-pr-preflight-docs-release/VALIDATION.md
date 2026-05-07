# Validation

| Command | Result |
|---|---|
| `cargo check -q` | passed |
| `mise run pr-preflight` | passed; printed docs/context and macOS overlay validation reminders |
| docs workflow `docs_verify.py .` | passed |
| context engineering `validate_frontmatter.py .` | passed |
| context engineering `verify_references.py .` | passed |
| `mise run validate-macos-overlay-change` | passed |
| `mise run pr-preflight` after Codex review fix | passed; version metadata checks still pass for the real 1.3.1 bump |
