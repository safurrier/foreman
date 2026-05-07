# Validation

| Check | Status | Notes |
|---|---|---|
| Swift tests | passed | `swift test --package-path apps/macos-overlay` |
| Overlay validation lane | passed | `mise run validate-macos-overlay-change`; gauntlet now includes arrow-key navigation marker. |
| Install/relaunch | passed | Installed `~/Applications/Foreman.app` and relaunched process `33884`. |
| Manual arrow-key confirmation | pending | User to confirm after install. |
