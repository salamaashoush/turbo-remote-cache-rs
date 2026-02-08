FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app/
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && \
  apt-get install -y --no-install-recommends ca-certificates && \
  rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/turbo-remote-cache-rs /usr/local/bin/turbo-remote-cache-rs
EXPOSE 4000
CMD ["turbo-remote-cache-rs"]
