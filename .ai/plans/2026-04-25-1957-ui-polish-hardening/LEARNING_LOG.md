# Learning Log

## 2026-04-25

Started from review findings after the default-sort/UI-persistence slice. The highest-risk item is keeping preference persistence off the navigation hot path while still guaranteeing shutdown flushes.

## 2026-04-25 Completed implementation

The most important validation catch was precedence: applying persisted preferences to `RuntimeConfig` was not enough because `PersistedUiPreferences::apply_to_state` could still overwrite explicit config sort later. The fix is to strip state-level persisted sort when `[ui].default_sort` was explicitly configured. Runtime UI writes now mark dirty on hot actions and flush after a debounce or on shutdown, so navigation no longer performs synchronous disk writes per keypress.
