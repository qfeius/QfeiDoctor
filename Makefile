.PHONY: all build test lint format format-fix format-check dev run clean

# Default target
all: format lint test build

# ---- Build ----
build:
	cd src-tauri && cargo build
	npm run build

build-release:
	cd src-tauri && cargo build --release
	npm run build

# ---- Development ----
dev:
	npm run tauri dev

run: dev

# ---- Testing ----
test: test-rust test-fe

test-rust:
	cd src-tauri && cargo test

test-fe:
	npm run test

# ---- Linting ----
lint: lint-rust lint-fe

lint-rust:
	cd src-tauri && cargo clippy -- -D warnings

lint-fe:
	npm run lint

# ---- Formatting ----
# `make format` checks formatting (used by CI)
# `make format-fix` auto-fixes formatting
format: format-check-rust format-check-fe

format-fix: format-fix-rust format-fix-fe

format-fix-rust:
	cd src-tauri && cargo fmt

format-fix-fe:
	npm run format

format-check: format

format-check-rust:
	cd src-tauri && cargo fmt -- --check

format-check-fe:
	npm run format:check

# ---- Clean ----
clean:
	cd src-tauri && cargo clean
	rm -rf dist node_modules
