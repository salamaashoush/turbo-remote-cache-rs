FROM rust:latest as builder
WORKDIR /app/
COPY . .
RUN cargo build --release
FROM debian:bullseye-slim
RUN apt-get update && \
  rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/turbo-remote-cache-rs /usr/local/bin/turbo-remote-cache-rs
