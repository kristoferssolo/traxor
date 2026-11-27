export RUST_LOG := env("RUST_LOG", "warn")

set shell := ["bash", "-euo", "pipefail", "-c"]

alias b := build
alias c := check
alias f := fmt
alias r := run
alias t := test

# List available recipes
default:
    @just --list

# Run the full pre-commit workflow
[group("checks")]
ci: fmt check

# Run formatting, Clippy, test, and documentation checks
[group("checks")]
check: fmt-check clippy test doc

# Format the crate
[group("checks")]
fmt:
    cargo fmt --all

# Fail if anything is unformatted
[group("checks")]
fmt-check:
    cargo fmt --all -- --check

# Lint all targets
[group("checks")]
clippy:
    cargo clippy --all-features --all-targets -- --deny warnings

# Run the test suite
[group("checks")]
test *ARGS:
    cargo nextest run --all-features --no-tests=warn {{ ARGS }}

# Build documentation, including private items
[group("checks")]
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --all-features --document-private-items --no-deps

# Run traxor
[group("run")]
run *ARGS:
    cargo run -- {{ ARGS }}

# Build traxor
[group("build")]
build:
    cargo build --all-features

# Install traxor from this checkout
[group("build")]
install:
    cargo install --path . --locked

# Watch the crate during development
[group("development")]
watch:
    bacon

# Show dependency updates without modifying Cargo.lock
[group("dependencies")]
outdated:
    cargo update --dry-run --verbose

# Update dependencies within Cargo.toml constraints
[group("dependencies")]
update:
    cargo update

# Audit dependencies for known advisories
[group("dependencies")]
audit:
    cargo audit

# cliff.toml falls back to GITHUB_REPOSITORY for links when the branch has no upstream
# Regenerate CHANGELOG.md; pass --tag vX.Y.Z to name unreleased changes
[group("release")]
changelog *ARGS:
    origin="$(git remote get-url origin 2>/dev/null || true)"; repo="${origin#*github.com[:/]}"; \
        GITHUB_REPOSITORY="${GITHUB_REPOSITORY:-${repo%.git}}" git cliff --output CHANGELOG.md {{ ARGS }}

# Remove build artifacts
[group("maintenance")]
clean:
    cargo clean

# mise builds cargo tools outside the checkout, where rust-toolchain.toml does not apply,
# so RUSTUP_TOOLCHAIN passes the repository toolchain to them.
# Install the Rust toolchain and the tools pinned in mise.toml
[group("maintenance")]
setup:
    rustup toolchain install
    toolchain="$(rustup show active-toolchain)"; RUSTUP_TOOLCHAIN="${toolchain%% *}" mise install
