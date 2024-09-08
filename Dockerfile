FROM rust:latest AS builder
WORKDIR /app/
COPY . .
RUN cargo build --release

# Use a newer Debian version with a more recent glibc
FROM debian:bookworm-slim
RUN apt-get update && \
  rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/turbo-remote-cache-rs /usr/local/bin/turbo-remote-cache-rs
