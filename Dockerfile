# Stage 1: Frontend build
FROM oven/bun:1 AS frontend-builder
WORKDIR /app
COPY package.json bun.lock ./
COPY frontend/package.json frontend/
RUN bun install --frozen-lockfile
COPY frontend/ frontend/
RUN cd frontend && bun run build

# Stage 2: Rust backend build
FROM rust:1.85-slim-bookworm AS builder
WORKDIR /app/
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && \
  apt-get install -y --no-install-recommends ca-certificates curl && \
  rm -rf /var/lib/apt/lists/* && \
  useradd -r -s /bin/false app
COPY --from=builder /app/target/release/turbo-remote-cache-rs /usr/local/bin/turbo-remote-cache-rs
COPY --from=frontend-builder /app/frontend/dist /app/frontend/dist
WORKDIR /app
RUN chown -R app:app /app
USER app
EXPOSE 4000
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
  CMD curl -f http://localhost:4000/health || exit 1
CMD ["turbo-remote-cache-rs"]
