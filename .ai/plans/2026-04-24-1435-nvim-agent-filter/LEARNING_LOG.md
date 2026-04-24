# Learning Log

## 2026-04-24

User reported `nvim` appears in agents-only mode. Starting focused slice before opening PR.

## 2026-04-24

Root cause: compatibility recognition ignored shell foreground commands but not editor foreground commands. A stale editor buffer containing Claude/Codex text could match harness tokens and make `nvim` visible in agents-only mode. Added editor foreground command exclusion for common terminal editors.
