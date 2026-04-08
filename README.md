# foreman

TUI for managing AI agents across tmux

## Quick Start

```bash
# Install dependencies
mise run setup

# Run quality checks
mise run check

# Start development
mise run dev```

## Starting New Work

```bash
git checkout -b feat/<slug>
mise run plan -- <slug>
```

## Task Reference

| Command | Purpose |
|---------|---------|
| `mise run setup` | Install dependencies and prepare the environment |
| `mise run fmt` | Auto-format code |
| `mise run lint` | Run lint checks (non-modifying) |
| `mise run typecheck` | Run static type analysis |
| `mise run test` | Run unit tests |
| `mise run build` | Build artifacts |
| `mise run check` | Fast quality gate (fmt + lint + typecheck + test) |
| `mise run dev` | Start local development || `mise run ci` | CI entrypoint (= check) |
| `mise run plan -- <slug>` | Create a plan directory for a unit of work |
| `mise run verify` | Heavy validation (integration, docker, security) |

## Project Structure

```
foreman/
├── src/                    # Source code
│   ├── main.rs             # Entry point
│   └── lib.rs              # Library with examples
├── .mise.toml              # Task runner config
├── Cargo.toml              # Package manifest
├── rustfmt.toml            # Formatter config
├── Dockerfile              # Multi-stage build
└── README.md
```

## Development

This project uses [mise](https://mise.jdx.dev/) as the task runner. All quality gates
are accessible via `mise run <task>`.

CI calls a single entrypoint: `mise run ci`.
