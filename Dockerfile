# ── Builder ───────────────────────────────────
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

COPY . .
RUN cargo build --release

# ── Runtime ───────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/foreman /usr/local/bin/app

EXPOSE 8080
ENTRYPOINT ["app"]
