.PHONY: check fmt fmt-check lint test build

check: fmt-check lint test

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets -- -D warnings
	shellcheck install.sh uninstall.sh

test:
	cargo test --workspace

build:
	cargo build --release
