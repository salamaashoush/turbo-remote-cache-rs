FROM rust:latest AS builder
WORKDIR /app/
COPY . .

# Install the musl target
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools

# Build with musl instead of glibc
RUN cargo build --release --target x86_64-unknown-linux-musl

# Use a minimal base image since we are now statically linked
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/turbo-remote-cache-rs /usr/local/bin/turbo-remote-cache-rs

