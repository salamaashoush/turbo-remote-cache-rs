FROM rust:1.70 as builder
WORKDIR /usr/src/turbo-remote-cache-rs
COPY . .
RUN rustup update nightly
RUN cargo +nightly build --release -Z sparse-registry
FROM debian:bullseye-slim
RUN apt-get update & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/turbo-remote-cache-rs/target/release/turbo-remote-cache /usr/local/bin/turbo-remote-cache
