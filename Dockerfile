FROM rust:1.69 as builder
WORKDIR /usr/src/turbo-remote-cache-rs
COPY . .
RUN cargo install --path .
FROM debian:bullseye-slim
RUN apt-get update & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/turbo-remote-cache /usr/local/bin/turbo-remote-cache
