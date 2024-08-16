#!/usr/bin/env -S just --justfile

_default:
  @just --list -u

alias r := ready

# Make sure you have cargo-binstall installed.
# You can download the pre-compiled binary from <https://github.com/cargo-bins/cargo-binstall#installation>
# or install via `cargo install cargo-binstall`
# Initialize the project by installing all the necessary tools.
init:
  cargo binstall cargo-watch cargo-insta typos-cli taplo-cli wasm-pack cargo-llvm-cov cargo-shear -y

# When ready, run the same CI commands
ready:
  git diff --exit-code --quiet
  typos
  just fmt
  just check
  just test
  just lint
  just doc
  just ast
  cargo shear
  git status


install-hook:
  echo "#!/bin/sh\njust fmt" > .git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit

# --no-vcs-ignores: cargo-watch has a bug loading all .gitignores, including the ones listed in .gitignore
# use .ignore file getting the ignore list
# Run `cargo watch`
watch command:
  cargo watch --no-vcs-ignores -i '*snap*' -x '{{command}}'

# Format all files
fmt:
  cargo fmt
  taplo format

# Run cargo check
check:
  cargo ck

# Run all the tests
test:
  cargo test

# Lint the whole project
lint:
  cargo lint -- --deny warnings

doc:
  RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items



# Get code coverage
codecov:
  cargo codecov --html

# Run the benchmarks. See `tasks/benchmark`
benchmark:
  cargo benchmark

# Removed Unused Dependencies
shear:
  cargo shear --fix

# Automatically DRY up Cargo.toml manifests in a workspace.
autoinherit:
  cargo binstall cargo-autoinherit
  cargo autoinherit



# Build in release build
turbo:
  cargo build --release -p turbo-remote-cache-rs --bin turbo-remote-cache-rs

# # Generate the JavaScript global variables. See `tasks/javascript_globals`
# javascript-globals:
#   cargo run -p javascript_globals

# Upgrade all Rust dependencies
upgrade:
  cargo upgrade --incompatible

