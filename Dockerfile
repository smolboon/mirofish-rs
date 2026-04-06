# MiroFish Rust/Wasm Docker Setup
# Multi-stage build for Rust backend + Wasm frontend

# Stage 1: Build Wasm frontend
FROM rust:1.85-slim AS wasm-builder

RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install trunk --locked

WORKDIR /app
COPY . .
RUN cd crates/mirofish-web && trunk build --release

# Stage 2: Build Rust backend
FROM rust:1.85-slim AS backend-builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
COPY --from=wasm-builder /app/crates/mirofish-web/dist ./static

RUN cargo build --release -p mirofish-server

# Stage 3: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend-builder /app/target/release/mirofish-server /usr/local/bin/
COPY --from=backend-builder /app/static ./static

ENV RUST_LOG=info
EXPOSE 8080

CMD ["mirofish-server"]