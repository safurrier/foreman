---
id: foreman-adr-0002
title: ADR 0002 — Source aggregation and remote SSH targets
description: >
  Design and implementation plan for making Foreman aggregate local and remote
  tmux-backed sources, including Alex's Coder SSH workflow, while keeping the
  tmux popup and macOS overlay product surfaces conceptually parallel.
index:
  - id: decision
    keywords: [sources, remote, ssh, coder, tmux, macos-overlay, popup]
  - id: implementation-plan
    keywords: [phases, source-provider, tmux-server-name, aggregation]
  - id: validation-plan
    keywords: [tests, smoke, ssh, coder, validation]
  - id: herdr-comparison
    keywords: [herdr, server-client, deviation, remote-attach]
---

# ADR 0002: Source aggregation and remote SSH targets

**Status**: Proposed — architecture-polish reviewed, revised for implementation
**Date**: 2026-06-04
**Deciders**: Alex Furrier, Foreman maintainers
**Generated from**: Herdr remote attach research, Coder workflow spikes, and architecture polish review

---

## Context

Foreman currently monitors and controls AI coding agents running in tmux on the
host where `foreman` executes. It has two user-facing surfaces:

- the terminal/Ratatui dashboard, often launched as a tmux popup with
  `foreman --popup`
- the native macOS `Foreman.app` overlay, which shells out to Foreman's control
  API commands

Alex's common work setup is local Mac → SSH to Coder → `tmux -L user`. The
existing dots binding launches the terminal popup directly to avoid shell
startup latency:

```tmux
# foreman floating popup (q to dismiss)
# '--' makes tmux exec directly, bypassing $SHELL startup (~6s zsh init)
bind a display-popup -h 80% -w 80% -E -- "$HOME/.cargo/bin/foreman" --popup
```

The desired product behavior is not just remote attach to one host. It is an
operator overview of all active agent work, local and remote, without needing to
cycle between targets. The terminal popup and macOS overlay should stay
conceptually parallel: if Foreman can show a cross-source overview, that overview
should not be Mac-only.

## Research and spikes

### Herdr design

Herdr uses a persistent server/client split. The remote path prepares a remote
Herdr binary/server, opens a local temporary Unix socket, starts an SSH stdio
bridge, and runs the local client against that forwarded socket. This is a good
fit for Herdr because Herdr owns terminal panes and pane lifetime.

Foreman should not copy this as the first design because Foreman is not a tmux
replacement and does not own pane lifetime. Foreman already has one-shot JSON
and action commands (`agents`, `focus`, `send`, `extensions`) that can be run on
the machine that owns the tmux server.

### Coder SSH config spike

`ssh -G coder.alex-furrier-dev-gpu-1` resolves a normal SSH host with:

- `hostname coder.alex-furrier-dev-gpu-1`
- `user alex.furrier`
- `controlmaster auto`
- `controlpath /Users/alex.furrier/.ssh/control-alex.furrier@coder.alex-furrier-dev-gpu-1:22`
- `forwardagent yes`
- keepalives already set

This means repeated SSH command execution can likely rely on existing SSH
ControlMaster behavior instead of Foreman inventing its own persistent transport
in v1.

### Remote availability spike

A noninteractive command can reach the Coder workspace:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=8 coder.alex-furrier-dev-gpu-1 \
  'printf "host="; hostname; printf "tmux="; command -v tmux; printf "foreman="; command -v foreman || true'
```

Observed result:

```text
host=alex-furrier-dev-gpu-1
tmux=/usr/bin/tmux
foreman=/home/discord/.cargo/bin/foreman
```

### tmux server-name spike

Local and remote spikes confirmed that `tmux -L <name> display-message -p
'#{socket_path}'` returns a concrete socket path. Foreman can therefore support a
server-name target either by passing `tmux -L <name>` directly or by resolving it
to the socket path Foreman already knows how to pass as `tmux -S <path>`.

A remote fresh tmux server worked with today's hidden `--tmux-socket` path:

```bash
ssh coder.alex-furrier-dev-gpu-1 \
  'set -e; name=foreman_spike_$$; \
   tmux -L $name new-session -d -s spike "sh -lc sleep\\ 20"; \
   /home/discord/.cargo/bin/foreman --tmux-socket /tmp/tmux-$(id -u)/$name agents --json --all-panes | head -80; \
   tmux -L $name kill-server'
```

This proves the core remote execution path can work without a Foreman daemon.

### Open Coder/tmux issue discovered

The existing remote `tmux -L user` socket currently returned `server exited
unexpectedly` during a direct SSH spike. A fresh `tmux -L foreman_spike_*` server
worked. Source health checks must distinguish Foreman failure from stale/broken
remote tmux socket state and report a useful diagnostic instead of failing the
whole dashboard.

## Decision

Foreman will introduce a **source aggregation model**.

A source is a place Foreman can query and control tmux-backed agent work. Sources
are persisted in Foreman's Rust-owned config, not only in the macOS overlay.
Both the terminal dashboard/popup and the macOS overlay consume the same
source-aware inventory and action model.

The product direction is:

- configured sources can be shown together from both the terminal dashboard and
  macOS overlay
- rows are source-badged
- the current tmux source, when known, is emphasized but not exclusive
- actions route through the selected row's source
- unreachable sources produce source-scoped diagnostics and do not hide healthy
  sources

Rollout rule: all-source aggregation may be implemented behind an explicit
opt-in before both surfaces are source-aware, but `default_scope = "all"` must
not become the default product behavior until the Ratatui popup/dashboard and
macOS overlay both render and act on source-scoped rows. This protects the
product invariant that cross-source visibility is not a Mac-only capability and
prevents temporary surface divergence.

The first remote transport will be SSH command execution of Foreman's existing
control commands on the remote host. A persistent Foreman daemon or Herdr-style
client protocol is deferred until the product needs lower-latency subscriptions,
remote binary lifecycle management, or richer streaming behavior.

## Product contract

### Source visibility

Foreman inventory is the merged view of configured sources:

```text
Foreman inventory = local source + enabled remote sources + diagnostics
```

Both main surfaces render that inventory:

```text
Ratatui dashboard / tmux popup  → source-aware merged inventory
macOS overlay                  → same source-aware merged inventory
```

The surfaces may present differently, but they must not have different default
scope semantics.

### Scope controls

Foreman supports explicit source scope controls:

```bash
foreman --sources all
foreman --sources current
foreman --source coder-dev-gpu-1
```

Proposed final defaults once both surfaces are source-aware:

```toml
[sources]
default_scope = "all"          # all | current | local
current_source_first = true
```

Implementation default before parity ships:

```toml
[sources]
default_scope = "current"      # or local/current-compatible behavior
current_source_first = true
```

New installs can migrate to `all` only when source badges, source-scoped actions,
and source diagnostics are implemented in both surfaces. Existing configs should
not silently switch behavior unless the migration is explicit and documented.

The direct tmux popup binding can remain unchanged. It inherits the configured
default scope:

```tmux
bind a display-popup -h 80% -w 80% -E -- "$HOME/.cargo/bin/foreman" --popup
```

### Source badges and actions

Rows carry source identity. Pane ids are only unique within a source, so Foreman
must not treat `%42` as globally unique.

```rust
struct SourcePaneId {
    source_id: SourceId,
    pane_id: PaneId,
}
```

`SourcePaneId` is the canonical identity for every merged-inventory concern:
selection, flash targets, focus/send actions, source diagnostics attached to
rows, extension-card merge maps, linked repository records, PR cache keys,
notification cooldowns, stale snapshot cache entries, startup cache keys, and
Swift row identity. Bare `paneId` remains present only as source-local display and
compatibility data.

Control API entries gain source fields and a stable composite id:

```json
{
  "id": "source:coder-dev-gpu-1:pane:%42",
  "sourcePaneId": "source:coder-dev-gpu-1:pane:%42",
  "sourceId": "coder-dev-gpu-1",
  "sourceLabel": "Coder dev-gpu-1",
  "sourceKind": "ssh",
  "paneId": "%42",
  "status": "working"
}
```

Action responses also include source identity:

```json
{
  "schemaVersion": 2,
  "ok": true,
  "action": "focus",
  "sourceId": "coder-dev-gpu-1",
  "paneId": "%42",
  "sourcePaneId": "source:coder-dev-gpu-1:pane:%42"
}
```

Focus/send include source when operating on merged inventory:

```bash
foreman focus --source coder-dev-gpu-1 --pane %42 --json
foreman send --source coder-dev-gpu-1 --pane %42 --stdin --json
```

For compatibility, omitting `--source` targets the current/local source when the
pane id is unambiguous in that command context. Ambiguous pane ids should produce
a structured error that lists candidate sources.

## Source config

Persist sources in Foreman's normal config file so Rust owns the product model
and every surface shares it.

Example:

```toml
[sources]
default_scope = "all"
current_source_first = true
query_timeout_ms = 5000

[sources.local]
kind = "local"
label = "Local Mac"
enabled = true

[sources.coder-dev-gpu-1]
kind = "ssh"
label = "Coder dev-gpu-1"
host = "coder.alex-furrier-dev-gpu-1"
foreman = "/home/discord/.cargo/bin/foreman"
tmux_server_name = "user"
enabled = true
query_timeout_ms = 5000
```

Source management commands:

```bash
foreman sources list --json
foreman sources add ssh coder-dev-gpu-1 \
  --host coder.alex-furrier-dev-gpu-1 \
  --foreman /home/discord/.cargo/bin/foreman \
  --tmux-server-name user \
  --label "Coder dev-gpu-1"
foreman sources doctor coder-dev-gpu-1
foreman sources remove coder-dev-gpu-1
```

A later convenience command may infer Alex's common Coder settings:

```bash
foreman sources add-coder alex-furrier-dev-gpu-1
```

## Architecture

### Core modules

Proposed Rust ownership:

| Module | Ownership |
|---|---|
| `config` | parse and validate source config |
| `sources` | source ids, source config models, provider trait, source aggregation service, deadlines, stale snapshots, diagnostic normalization |
| `sources::local` | local tmux-backed provider |
| `sources::ssh` | SSH command provider for remote Foreman control commands |
| `adapters::tmux` | tmux subprocess adapter, extended for `tmux -L` server names |
| `services::control_api` | source-aware JSON schema and action responses |
| `runtime` | periodic source refresh, source diagnostics, action routing |
| `apps/macos-overlay` | display source-aware control API results; no independent SSH implementation |

### Source provider and aggregator seam

Use a source provider interface that can start with one-shot SSH and later accept
a daemon-backed transport without rewriting the UI model.

```rust
trait ForemanSource: Send + Sync {
    fn id(&self) -> &SourceId;
    fn label(&self) -> &str;
    fn kind(&self) -> SourceKind;
    fn agents(&self, request: AgentsRequest) -> SourceResult<AgentsResponse>;
    fn extensions(&self, pane: &PaneId) -> SourceResult<ExtensionCardsResponse>;
    fn focus(&self, pane: &PaneId) -> SourceResult<ActionResponse>;
    fn send(&self, pane: &PaneId, text: &str) -> SourceResult<ActionResponse>;
    fn doctor(&self) -> SourceDoctorReport;
}
```

Providers own transport. A dedicated `SourceAggregator` owns fan-out,
per-source deadlines, cancellation/supersession, stale snapshot policy,
diagnostic normalization, source wrapping, and schema compatibility.

```rust
struct SourceSnapshot {
    source: SourceDescriptor,
    entries: Vec<AgentEntry>,
    diagnostics: Vec<SourceDiagnostic>,
    generated_at_unix_ms: u128,
    duration_ms: u64,
    stale: bool,
    schema_version: u16,
}

struct AggregateSnapshot {
    entries: Vec<AgentEntry>,
    source_diagnostics: Vec<SourceDiagnostic>,
    inventory: ControlInventorySummary,
    partial_failure_count: usize,
}
```

`LocalSource` calls today's bootstrap/tmux path. `SshSource` shells out to the
remote `foreman` binary in internal `source-probe --local-only` mode to prevent
recursive source aggregation on the remote host. Runtime consumes a single
`AggregateSnapshot`; it should not implement per-source timeout or stale-cache
policy itself.

### Avoid recursive source fan-out

Remote source commands must query only the remote host's local tmux. If local
Foreman calls remote Foreman and remote Foreman also loads all of its configured
sources, two hosts could recursively query each other.

Use an internal non-recursive probe mode rather than relying on the public
`--sources local` convention alone:

```bash
ssh coder 'foreman source-probe --local-only --tmux-server-name user agents --json'
```

The local-only probe mode must:

- bypass remote provider construction entirely
- ignore the remote host's `default_scope = "all"`
- read only local config needed for the requested tmux target and control API
  shape
- never instantiate `SshSource`
- return source-local JSON that the caller wraps with the caller's configured
  source id/label
- be covered by a mutual-recursion fake test where host A points at B and B
  points at A, and querying A performs exactly one remote probe

The public contract should make recursion impossible by construction.

### SSH command construction

The SSH provider must not build shell strings with untrusted text. Prefer a
remote helper/probe contract with fixed shell shape and JSON/stdin for variable
payloads. If a shell command string is unavoidable, all quoting must live in one
small module with POSIX-shell fixture tests.

The provider should:

- construct local `ssh` argv as a vector
- use a fixed remote entrypoint such as `foreman source-probe --local-only ...`
- validate `host`, `source_id`, `tmux_server_name`, configured `foreman` path,
  and any user-provided SSH options
- pipe `send --stdin` text over stdin, never shell arguments or logs
- set `BatchMode=yes` and short connect/query timeouts for background refreshes
- allow users to opt into custom SSH config/flags only through validated config
- test pathological values containing spaces, quotes, semicolons, newlines,
  `$()`, and leading dashes

### Source diagnostics

A source query returns inventory and/or structured source diagnostics. Diagnostics
must have stable fields for renderers; `message` is for humans, not control flow.

```json
{
  "level": "warning",
  "code": "source.ssh.timeout",
  "sourceId": "coder-dev-gpu-1",
  "sourceLabel": "Coder dev-gpu-1",
  "sourceKind": "ssh",
  "message": "Coder dev-gpu-1 unreachable: ssh timed out after 2500ms",
  "retryable": true,
  "durationMs": 2500,
  "lastSuccessUnixMs": 1780000000000
}
```

Important diagnostic cases:

- SSH host unreachable
- remote Foreman missing
- remote Foreman version/schema unsupported
- remote tmux missing
- remote tmux server/socket stale or exited unexpectedly
- remote command timed out
- ambiguous pane id across sources

## Implementation plan

### Phase 0 — product/schema design freeze

- Finalize source config names and defaults.
- Record the rollout rule: backend aggregation can be opt-in before parity, but
  default `all` waits until both Ratatui and macOS overlay are source-aware.
- Decide source ordering: attention globally, with current source first as a
  grouping/sort modifier.
- Freeze `SourcePaneId`, composite row id, action response, and structured
  source diagnostic schema.
- Freeze internal `source-probe --local-only` behavior and recursion-prevention
  tests.
- Add examples to `docs/operator-guide.md` for Coder and source diagnostics.

### Phase 1 — tmux server-name support

Goal: make Coder's `tmux -L user` a first-class Foreman target.

Changes:

- Replace `Option<PathBuf>` tmux socket plumbing with a typed target:

  ```rust
  enum TmuxTarget {
      Default,
      Socket(PathBuf),
      ServerName(String),
  }
  ```

- Add visible CLI flag:

  ```bash
  foreman --tmux-server-name user agents --json
  ```

- Keep hidden/current `--tmux-socket` compatibility.
- Update startup cache keying to include server name vs socket identity.
- Update docs for tmux popup in `tmux -L user` environments.

Validation:

- Unit test tmux argv construction for default, `-S`, and `-L`.
- Real tmux smoke using temporary `tmux -L foreman-test-*`.
- Existing `mise run check`.

### Phase 2 — source config and local source provider

Goal: introduce source identity without remote complexity.

Changes:

- Add `sources` config model with a default local source.
- Add source-aware entry fields in the Rust model and JSON control API.
- Add `--source local`, `--sources all|current|local` parsing.
- Route all merged-inventory identity through `SourcePaneId`, including
  selection, flash targets, focus/send, extension-card merge maps, linked repos,
  PR/cache keys, notification cooldowns, startup cache, stale snapshots, and
  Swift row identity.
- Render source badges in Ratatui when more than one source exists or when a row
  is not from the current source.
- Update Swift decoders/fixtures for additive source fields.

Validation:

- Control API fixture tests preserve backward-compatible shape or intentional
  schema bump.
- Duplicate-pane fixture with local `%42` and remote `%42` proves composite
  identity throughout Rust models.
- Reducer/action tests prove selected remote `%42` does not route to local `%42`.
- Ambiguous bare `--pane %42` action returns a structured error listing
  candidate sources.
- Ratatui rendering tests for source badges.
- Swift decoder tests with source-aware fixtures.

### Phase 3 — SSH source provider

Goal: one-shot remote source queries/actions over SSH.

Changes:

- Add `SshSource` provider.
- Add `foreman sources list/add/remove/doctor` commands.
- Implement internal `source-probe --local-only` so SSH provider calls cannot
  recursively fan out into the remote host's configured sources.
- Implement `sources doctor` checks:
  - SSH resolves/connects
  - remote `foreman --version`
  - remote `foreman agents --json --sources local --tmux-server-name <name>`
  - remote tmux stale socket diagnostic
- Query sources in parallel with per-source timeout.
- Merge successful entries and diagnostics.
- Ensure remote provider calls never recurse into remote configured sources.
- Support action routing over SSH for `focus`, `send`, and `extensions`.
- Add remote schema/version checks so unsupported remote Foreman versions become
  source diagnostics rather than aggregate failures.

Validation:

- Fake SSH binary records argv/stdin and emits fixture JSON.
- Unit tests for quoting, stdin forwarding, timeout, and diagnostics, including
  spaces, quotes, semicolons, newlines, `$()`, and leading dashes.
- Mutual-recursion fake config test: host A points to B and B points to A;
  querying A performs exactly one remote local-only probe.
- Unsupported remote schema fixture produces a source diagnostic.
- Integration smoke against a temporary local `ssh` substitute script.
- Manual Coder smoke:

  ```bash
  foreman sources doctor coder-dev-gpu-1
  foreman agents --json --sources all
  foreman focus --source coder-dev-gpu-1 --pane <pane> --json
  ```

### Phase 4 — unified TUI/popup merged inventory

Goal: terminal Foreman has parity with macOS overlay.

Changes:

- Runtime refresh uses source aggregator.
- Sorting supports global attention/recent and current-source-first grouping.
- Source diagnostics show in the dashboard without hiding healthy sources.
- Selection remains stable across source refreshes.
- Focus/send/flash navigation route to source-scoped targets.

Validation:

- Reducer tests for selection stability across source insert/remove/failure.
- Render tests for multi-source grouped and globally sorted inventory.
- Runtime fake-source tests for slow/unreachable source degradation, including
  a timed-out source that does not block healthy local rows and stale snapshot
  marking from last successful refresh.
- Real tmux popup smoke remains fast and current-pane focus works.

### Phase 5 — macOS overlay consumes merged API

Goal: Mac overlay displays the same source-aware inventory as the TUI.

Changes:

- Update Swift models to include sources/source diagnostics and composite row
  identity.
- Add source badges/group labels to overlay rows.
- Add settings UI for selecting scope if needed, but keep configured default.
- Ensure Swift does not implement SSH; it shells out to local `foreman`, which
  owns source aggregation.
- Focus/send calls pass `--source <id>` for selected row.
- Land default-scope parity with the Ratatui surface in the same release gate;
  do not make all-source overview default in one surface only.

Validation:

- Swift decoder fixture tests.
- Fake Foreman overlay tests with multiple sources and source diagnostics.
- Swift extension-card merge test keys cards by composite source-pane identity,
  not bare pane id.
- Snapshot tests for grouped sources and unreachable Coder.
- Required overlay lane: `mise run validate-macos-overlay-change`.

### Phase 6 — performance and persistence polish

Goal: make all-sources overview feel boring and fast.

Changes:

- Use SSH ControlMaster when available; document recommended SSH config.
- Add short-lived per-source cache with stale markers for overlay first paint.
- Add source-level query timing logs.
- Add config reload/source doctor guidance.

Validation:

- Performance smoke with fake slow source.
- Log assertions where practical.
- Manual Coder refresh latency check.

## Validation plan summary

### Fast checks

```bash
cargo test sources --lib
cargo test control_api --lib
cargo test tmux --lib
swift test --package-path apps/macos-overlay
mise run check
```

### Remote/source-specific checks

```bash
# local temporary tmux server-name smoke
name=foreman-test-$$
tmux -L "$name" new-session -d -s spike 'sh -lc "sleep 30"'
foreman --tmux-server-name "$name" agents --json --all-panes
tmux -L "$name" kill-server

# fake SSH provider tests should run in cargo test; manual Coder smoke is opt-in
foreman sources doctor coder-dev-gpu-1
foreman agents --json --sources all
```

Required negative/contract cases:

- duplicate `%42` across local and remote remains two rows with distinct
  `sourcePaneId` values
- ambiguous bare `--pane %42` error lists candidate sources
- selected remote row calls `focus --source remote --pane %42`
- Swift extension cards merge by `sourcePaneId`
- mutual-recursion fake config does not fan out
- unsupported remote schema produces source diagnostic
- one source timeout does not block healthy local inventory
- TUI and overlay consume the same multi-source fixture semantics before
  `default_scope = "all"` ships

### macOS overlay checks

```bash
mise run validate-macos-overlay-change
```

### Heavy/release checks

```bash
mise run verify
```

Run `mise run verify-native` only when native hook behavior or real provider E2Es
are touched.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Source recursion between hosts | Remote provider always forces local-only scope on the remote call. |
| SSH latency makes overview slow | Per-source timeouts, parallel queries, ControlMaster, stale cache. |
| Surface divergence during rollout | Keep all-source default behind opt-in until Ratatui and macOS overlay both support source-aware display/actions. |
| Local and remote `%42` collide | Store and act on `SourcePaneId`, never bare pane id in merged state. |
| Product surfaces diverge | Rust core owns aggregation; both TUI and macOS overlay consume same source-aware API. |
| Coder tmux stale socket breaks dashboard | Source-scoped diagnostics; healthy sources still render. |
| Remote shell quoting bugs | Vectorized SSH argv, stdin for send text, fake SSH tests. |
| Schema churn breaks overlay | Additive fields first where possible; fixture decoder tests; schema bump only when necessary. |

## Deferred decisions

- Whether remote Foreman binary auto-install belongs in Foreman v1. Initial plan:
  no; `sources doctor` should report missing binary and suggest install.
- Whether a future Foreman daemon should provide subscriptions/caching. Initial
  plan: design provider seam so daemon transport can replace SSH one-shot later.
- Whether the terminal popup should visually group by current source first or use
  pure global attention sort. Initial plan: configurable, default
  `current_source_first = true`.
- The exact visual treatment for source groups/badges. This can be iterated in
  Ratatui and Swift snapshots, but it must preserve source visibility and action
  clarity.

## Consequences

**Positive**:

- One persisted configuration can make Coder visible by default once both
  product surfaces are source-aware.
- Tmux popup and Mac overlay stay product-parallel.
- Foreman remains a tmux control surface rather than becoming a tmux replacement.
- Remote source support starts small and testable.

**Negative / trade-offs**:

- One-shot SSH source refresh may be slower than a persistent protocol.
- The control API schema and selection model become more complex because source
  identity is first-class.
- Some remote desktop affordances, such as browser/clipboard behavior, may need
  later design once source actions extend beyond focus/send/inspect.

## Herdr comparison

Herdr remote attach is a server/client protocol bridge for one Herdr session.
That is the right architecture for Herdr because Herdr owns panes.

Foreman's proposed design is source aggregation across multiple tmux-backed
control planes. That is the right architecture for Foreman because tmux owns
panes and Foreman owns operator overview, interpretation, and routing.

The main Herdr ideas Foreman should copy are:

- make remote access persistent in config
- fail with actionable diagnostics
- use SSH keepalive/multiplexing behavior rather than assuming perfect network
- leave room for a stronger transport later

The main Herdr ideas Foreman should not copy in v1 are:

- remote binary auto-install/update as a prerequisite
- a custom client/server terminal streaming protocol
- a single-active-session mental model
