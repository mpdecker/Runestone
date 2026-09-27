FROM node:22-bookworm AS frontend-builder

WORKDIR /app

COPY package.json pnpm-lock.yaml ./
RUN corepack enable && pnpm install --frozen-lockfile

COPY . .
RUN pnpm build

FROM rust:1.94-bookworm AS backend-builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/runestone-core/Cargo.toml crates/runestone-core/Cargo.toml
COPY crates/runestone-server/Cargo.toml crates/runestone-server/Cargo.toml
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml

RUN mkdir -p crates/runestone-core/src crates/runestone-server/src src-tauri/src && \
    echo "fn main() {}" > crates/runestone-server/src/main.rs && \
    echo "pub fn dummy() {}" > crates/runestone-core/src/lib.rs && \
    echo "fn main() {}" > src-tauri/src/main.rs

RUN cargo build --release -p runestone-server 2>/dev/null || true

COPY crates/runestone-core crates/runestone-core
COPY crates/runestone-server crates/runestone-server

RUN cargo build --release -p runestone-server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /app/target/release/runestone-server /usr/local/bin/runestone-server
COPY --from=frontend-builder /app/dist /app/static

ENV RUNESTONE_SERVER_PORT=3000
ENV STATIC_DIR=/app/static

EXPOSE 3000

CMD ["runestone-server"]
