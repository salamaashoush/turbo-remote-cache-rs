#!/usr/bin/env -S just --justfile

set dotenv-load

export DATABASE_URL := env("DATABASE_URL", "postgres://turbo:turbo_dev_password@localhost:5432/turbo_cache")
export JWT_SECRET := env("JWT_SECRET", "dev-secret-do-not-use-in-production")
export SQLX_OFFLINE := "true"

_default:
  @just --list -u

# --- Setup ---

# Install dev tooling (cargo-watch, typos, etc.)
init:
  cargo binstall cargo-watch typos-cli -y
  bun install

# --- Dev ---

# Start PostgreSQL via docker compose
db:
  docker compose up -d db

# Stop PostgreSQL
db-down:
  docker compose down

# Run backend (auto-reload on changes)
dev-be:
  cargo watch -x run

# Run frontend dev server (Vite, port 5173, proxies /api + /v8 to :4000)
dev-fe:
  bun run --filter turbo-remote-cache-dashboard dev

# Run both backend + frontend in parallel
dev: db mailpit
  just dev-be & just dev-fe & wait

# Start MailPit dev email server (UI at http://localhost:8025)
mailpit:
  docker compose up -d mailpit

# Build frontend for production
build-fe:
  bun run --filter turbo-remote-cache-dashboard build

# Build backend in release mode
build-be:
  SQLX_OFFLINE=true cargo build --release

# Build everything
build: build-fe build-be

# --- Quality ---

# Format all code (Rust + frontend)
fmt:
  cargo fmt
  bun run --filter turbo-remote-cache-dashboard format

# Check formatting without writing
fmt-check:
  cargo fmt -- --check
  bun run --filter turbo-remote-cache-dashboard format:check

# Lint Rust (clippy)
lint-be:
  SQLX_OFFLINE=true cargo clippy -- -D warnings

# Lint frontend (ESLint + typecheck)
lint-fe:
  bun run --filter turbo-remote-cache-dashboard lint
  bun run --filter turbo-remote-cache-dashboard typecheck

# Lint everything
lint: lint-be lint-fe

# Run Rust tests
test:
  SQLX_OFFLINE=true cargo test

# Check for typos in the codebase
typos:
  typos

# Full CI-style check: format, lint, test, typos
check: fmt-check lint test typos

# Same as check but also auto-format first
ready: fmt lint test typos

# --- Docker ---

# Build Docker image
docker-build:
  docker build -t turbo-remote-cache-rs .

# Run full stack via docker compose
docker-up:
  docker compose up --build

# --- Utilities ---

# Watch and re-run tests on changes
watch-test:
  cargo watch -x test

# Watch and re-run clippy on changes
watch-lint:
  cargo watch -x 'clippy -- -D warnings'

# Upgrade all Rust dependencies
upgrade:
  cargo upgrade --incompatible

# --- Turbo CLI ---

# Turbo CLI: login to local server
turbo-login:
  cd test-turbo-app && npx turbo login --api http://localhost:4000 --login http://localhost:4000

# Turbo CLI: link repo to team
turbo-link:
  cd test-turbo-app && npx turbo link --api http://localhost:4000

# Turbo CLI: test build with remote cache
turbo-build:
  cd test-turbo-app && npx turbo build --api http://localhost:4000
