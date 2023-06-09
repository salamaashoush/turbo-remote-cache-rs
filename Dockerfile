FROM rust:1.70 as builder
WORKDIR /app/
COPY . .
RUN rustup update nightly
RUN \
  --mount=type=cache,target=/usr/local/cargo/registry \
  --mount=type=cache,target=/app/target \
  cargo +nightly build --release
FROM debian:bullseye-slim
RUN --mount=type=cache,target=/var/cache/apt \
  apt-get update && \
  rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/turbo-remote-cache /usr/local/bin/turbo-remote-cache
