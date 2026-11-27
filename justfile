export RUST_LOG := env("RUST_LOG", "warn")

set shell := ["bash", "-euo", "pipefail", "-c"]

# List available recipes
default:
    @just --list

alias b := build
alias c := check
alias d := docs
alias f := fmt
alias r := run
alias t := test

[group("build")]
build:
    cargo build --release

[group("run")]
run:
    cargo run --release

# Run all checks (fmt, clippy, docs, test)
[group("dev")]
check: fmt clippy docs test

[group("dev")]
fmt:
    cargo fmt --all

# Fail if anything is unformatted
[group("dev")]
fmt-check:
    cargo fmt --all -- --check

[group("dev")]
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

[group("dev")]
docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features --document-private-items

[group("dev")]
test:
    cargo nextest run --all-features
    cargo test --doc

[group("dev")]
clean:
    cargo clean

[group("dev")]
setup:
    cargo install cargo-nextest sccache
