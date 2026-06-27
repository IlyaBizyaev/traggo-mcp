# List available recipes
default:
    @just --list

format:
    cargo fmt

lint:
    cargo clippy --all-targets --all-features --locked -- -D warnings

test:
    cargo test --all-features --locked

check: lint test

build:
    cargo build --release --locked

docker:
    docker build -t traggo-mcp:local .
