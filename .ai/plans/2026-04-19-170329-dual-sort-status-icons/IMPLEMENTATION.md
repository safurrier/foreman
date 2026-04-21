# Implementation Plan

## Slice Shape

Keep this as a narrow operator-surface pass, not a full information-architecture
change.

- Reuse the existing `SortMode` concept.
- Make both sort modes explicitly compound.
- Teach the current mode in the UI.
- Update semantic status colors without changing lifecycle meaning.

## Proposed Changes

### 1. Make sort presets explicit

Update the current sort behavior so both modes are clearly dual-axis:

- `RecentActivity`
  - compare by `recent_activity()` descending
  - then by `attention_rank()` ascending
- `AttentionFirst`
  - compare by `attention_rank()` ascending
  - then by `recent_activity()` descending

This keeps the current mental model, but removes the asymmetry where one mode is
obviously compound and the other is not.

### 2. Clarify the mode labels in the UI

Keep the product terms short, but expose the axis order somewhere visible:

- header/context could show `View: recent (recent → status)` or a shorter equivalent
- help/footer should explain that `o` cycles operator sort order, not just “view”

The goal is operator clarity, not extra controls.

### 3. Update status icon semantics

Treat the existing status symbols as semantic icons:

- working/active: orange or amber
- needs attention: yellow
- idle: green
- error: red
- unknown: muted

Do this per theme token, not with one shared absolute color table:

- palette themes should keep their own tuned `working`, `attention`, `idle`, and `error` values
- `terminal_theme()` should use terminal-safe equivalents
- `no_color_theme()` should keep the same monochrome behavior

Audit:

- `terminal_theme()`
- palette-backed themes
- no-color fallback should keep the same glyphs and text semantics

### 4. Test strategy

Start with state and render tests, then broaden if needed.

- state/reducer:
  - `recent` sorts by recency, then state
  - `attention` sorts by state, then recency
  - logical selection is preserved when cycling sort mode
- render:
  - visible sort wording reflects the new compound semantics
  - status legend/icons use the intended semantic styles
- runtime/release:
  - only extend compiled-binary smokes if the wording or ordering proof is not already covered well enough by the existing gauntlet

## Expected Validation

Targeted first:

```bash
cargo test app::reducer::tests::sort_mode_change_preserves_logical_selection -- --nocapture
cargo test ui::render::tests -- --nocapture
```

Then repo gates:

```bash
mise run check
mise run verify
FOREMAN_REQUIRE_REAL_E2E=1 \
FOREMAN_REAL_CLAUDE_E2E=1 \
FOREMAN_REAL_CODEX_E2E=1 \
FOREMAN_REAL_PI_E2E=1 \
mise run verify-native
```

## Open Question To Revisit During Implementation

If the user really wants more than two presets, the clean follow-on would be a
separate slice for configurable primary/secondary axes. This slice should not
quietly grow into that.
