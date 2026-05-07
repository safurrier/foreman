# macOS Overlay Design / UX Review

Reviewed against:

- `.agent/skills/foreman-swift-overlay-ux/SKILL.md`
- `.agent/skills/macos-design-guidelines/SKILL.md`
- `.agent/skills/macos-app-design/SKILL.md`
- `../dotfiles/config/ai-config/plugins/design-ux/skills/userinterface-wiki/SKILL.md`
- `../dotfiles/config/ai-config/plugins/design-ux/skills/web-design-foundations/SKILL.md`
- headless snapshots in `.ai/validation/macos-overlay/headless-snapshots/`

## Passes

- **Command-palette happy path is clear.** Open → type → move → focus/send is supported and validated.
- **Progressive disclosure is improving.** The new PR card appears only when metadata exists; help remains behind `?`.
- **Keyboard-first behavior is validated.** Core reducer tests and UI event gauntlet cover movement, focus, help, compose, refresh, and send.
- **Visual state coverage is strong.** Empty, error, long path, long preview, many agents, help, scrolled help, PR card, and theme variants render through headless snapshots with OCR assertions.
- **Mac citizenship improved.** Hotkey recording uses `KeyboardShortcuts.Recorder`; Settings lives in the menu bar; native materials/system controls remain the baseline.

## Findings / Follow-ups

1. **Theme polish is intentionally lightweight.** Current themes are accent/background treatments, not full design systems. If themes become user-facing polish, add per-theme status colors and contrast checks.
   - Related rule: every color needs a purpose; validate foreground/background contrast.

2. **PR card is useful but action-light.** The clickable `Open PR` link solves the immediate need. Full parity with TUI PR actions (`copy`, refresh, expanded details) remains intentionally scoped separately.
   - Recommendation: add Copy URL only if you use it often; otherwise keep card minimal.

3. **Footer hint density is close to the limit.** It is still readable in normal snapshots, but the footer now carries mode, help, theme, and action hints.
   - Recommendation: if more commands are added, move rarely used hints into `?` help and keep footer to 2-3 immediate actions.

4. **The list/detail split is productive but fixed-width.** Current 360px list works for snapshots, but very long agent names may benefit from user-resizable width later.
   - Recommendation: defer until real daily use shows pain.

5. **Animations should stay minimal.** Keyboard navigation should remain instant; avoid animated row movement or delayed transitions.
   - Related rule: no animation for high-frequency keyboard navigation.

6. **Terminal activation remains OS-policy-sensitive.** The adapter is correctly isolated. Keep activation opt-in because forced focus changes can be annoying.

## Current recommendation

Ship the overlay as a validated local MVP after one more manual smoke:

```bash
FOREMAN_OVERLAY_FOREMAN_PATH=$PWD/target/debug/foreman \
FOREMAN_OVERLAY_SHOW_ON_LAUNCH=1 \
apps/macos-overlay/.build/debug/foreman-overlay
```

Manual checks:

- default hotkey is `Ctrl+Option+Shift+A`
- Settings recorder can change it
- PR card appears for a workspace with a PR
- `Open PR` opens browser
- `Cmd+T` cycles themes without reducing readability
- optional terminal activation works for the terminal app you use
