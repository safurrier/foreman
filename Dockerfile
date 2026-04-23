# ── Builder ───────────────────────────────────
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.docker.toml Cargo.toml
COPY Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src Cargo.toml

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --bins

# ── Runtime ───────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/foreman /usr/local/bin/foreman
COPY --from=builder /build/target/release/foreman-claude-hook /usr/local/bin/foreman-claude-hook
COPY --from=builder /build/target/release/foreman-codex-hook /usr/local/bin/foreman-codex-hook
COPY --from=builder /build/target/release/foreman-pi-hook /usr/local/bin/foreman-pi-hook

EXPOSE 8080
ENTRYPOINT ["foreman"]
