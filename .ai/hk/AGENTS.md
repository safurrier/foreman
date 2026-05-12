# Harness Kit Exports

`.ai/hk/**` contains generated Harness Kit handoff exports for Foreman work items.

## Rules

- **DO** generate these directories with `hk export`. **NOT** hand-edit export files. **BECAUSE** the HK ledger is the lifecycle source of truth and exports are review snapshots.
- **DO** commit compact export metadata such as `README.md`, `meta.json`, and `artifacts/README.md`. **NOT** commit arbitrary artifact payloads under `artifacts/` unless a reviewer explicitly needs them. **BECAUSE** `.ai/validation/**` remains Foreman's durable validation evidence root and HK artifacts can grow quickly.
- **DO** treat `.ai/plans/**` as legacy historical evidence. **NOT** create new `.ai/plans/**` directories for normal work. **BECAUSE** new meaningful work should use HK lifecycle commands and export to `.ai/hk/<work-id>/` when a handoff package is needed.

## Expected Export Shape

```text
.ai/hk/<work-id>/
  README.md
  meta.json
  artifacts/
    README.md
```

If an export includes additional artifact files, make sure they are intentional, privacy-checked, and small enough for review.
