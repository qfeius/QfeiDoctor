.PHONY: all build test lint format format-fix format-check dev run preview clean

# Default target
all: format lint test build

# ---- Build ----
build: node_modules
	cd src-tauri && cargo build
	npm run build

build-release: node_modules
	cd src-tauri && cargo build --release
	npm run build

# ---- Development ----
node_modules: package.json
	npm install
	@touch node_modules

dev: node_modules
	npm run tauri dev

run: dev

preview: node_modules
	npm run tauri build -- --debug
	open src-tauri/target/debug/bundle/macos/QfeiDoctor.app

# ---- Testing ----
test: test-rust test-fe

test-rust:
	cd src-tauri && cargo test

test-fe: node_modules
	npm run test

# ---- Linting ----
lint: lint-rust lint-fe

lint-rust:
	cd src-tauri && cargo clippy -- -D warnings

lint-fe: node_modules
	npm run lint

# ---- Formatting ----
# `make format` checks formatting (used by CI)
# `make format-fix` auto-fixes formatting
format: format-check-rust format-check-fe

format-fix: format-fix-rust format-fix-fe

format-fix-rust:
	cd src-tauri && cargo fmt

format-fix-fe: node_modules
	npm run format

format-check: format

format-check-rust:
	cd src-tauri && cargo fmt -- --check

format-check-fe: node_modules
	npm run format:check

# ---- Clean ----
clean:
	cd src-tauri && cargo clean
	rm -rf dist node_modules
