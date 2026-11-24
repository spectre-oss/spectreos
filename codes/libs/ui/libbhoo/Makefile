ROOT := $(abspath ../../../..)
PORT ?= 8766
OUT ?= $(ROOT)/context/dist/bhoo-demo.xdc
COMPONENT ?=
EXTRA_ARGS ?=

.DEFAULT_GOAL := help

.PHONY: help build build-wayland build-release check clippy fmt fmt-check \
	test test-unit test-core test-material test-android test-alloc test-wayland \
	run-launcher run-components \
	web web-full serve \
	webxdc webxdc-gallery webxdc-launcher webxdc-contacts webxdc-gmail webxdc-android webxdc-app-tests \
	review-status review-package \
	clean clean-dev

help: ## Show available targets
	@grep -E '^[a-zA-Z0-9_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# --- Build & lint ---

build: ## Build core libbhoo (no Wayland features)
	cargo build --locked

build-wayland: ## Build libbhoo with Wayland/GLES backend
	cargo build --locked --features wayland

build-release: ## Release build with Wayland/GLES backend
	cargo build --locked --release --features wayland

check: ## Check default and all-features + examples builds
	cargo check --locked --no-default-features
	cargo check --locked --all-features --examples

clippy: ## Run clippy with -D warnings on all targets & features
	cargo clippy --locked --all-targets --all-features -- -D warnings

fmt: ## Format Rust sources
	cargo fmt

fmt-check: ## Check Rust formatting without modifying files
	cargo fmt -- --check

# --- Tests ---

test: ## Run all unit specs (*.spec.rs) and integration suites
	cargo test --locked --all-features

test-unit: ## Run only colocated unit specs & inline unit tests
	cargo test --locked --all-features --lib

test-core: ## Run core integration test suite (tests/core)
	cargo test --locked --test core

test-material: ## Run Material integration test suite (tests/material)
	cargo test --locked --test material

test-android: ## Run Android XML integration test suite (tests/android)
	cargo test --locked --test android

test-alloc: ## Run standalone heap-allocation test (tests/allocations)
	cargo test --locked --test allocations

test-wayland: ## Run headless Weston/GLES pixel test suite
	python3 $(ROOT)/tools/test-bhoo-wayland.py

# --- Native Wayland examples ---

run-launcher: ## Run native Wayland launcher example (release)
	cargo run --locked --release --features wayland --example launcher

run-components: ## Run native Wayland components gallery example (release)
	cargo run --locked --release --features wayland --example components

# --- Web / WASM / WebXDC ---

web: ## Build Rust-only WASM web playground
	python3 $(ROOT)/tools/build-bhoo-web.py --rust-only

web-full: ## Build full Bhoo + Angular comparison web site
	python3 $(ROOT)/tools/build-bhoo-web.py

serve: ## Build and serve Rust-only web playground on 127.0.0.1:$(PORT)
	python3 $(ROOT)/tools/serve-bhoo-web.py --rust-only --port $(PORT)

webxdc: ## Package a WebXDC (set COMPONENT=<slug> for focused plain build, or defaults to gallery)
	@if [ -n "$(COMPONENT)" ]; then \
		python3 $(ROOT)/tools/build-bhoo-webxdc.py --plain --component "$(COMPONENT)" --output "$(OUT)" $(EXTRA_ARGS); \
	else \
		python3 $(ROOT)/tools/build-bhoo-webxdc.py --gallery --output "$(OUT)" $(EXTRA_ARGS); \
	fi

webxdc-gallery: ## Package the Bhoo-only single-page gallery WebXDC
	python3 $(ROOT)/tools/build-bhoo-webxdc.py --gallery --output "$(ROOT)/context/dist/bhoo-gallery.xdc" $(EXTRA_ARGS)

webxdc-launcher: ## Package the Android System UI & Launcher WebXDC demo
	python3 $(ROOT)/tools/build-bhoo-webxdc.py --launcher-demo --output "$(ROOT)/context/dist/bhoo-launcher.xdc" $(EXTRA_ARGS)

webxdc-contacts: ## Package the Android Contacts WebXDC demo
	python3 $(ROOT)/tools/build-bhoo-webxdc.py --contacts-demo --output "$(ROOT)/context/dist/bhoo-contacts.xdc" $(EXTRA_ARGS)

webxdc-gmail: ## Package the Gmail UI WebXDC mockup
	python3 $(ROOT)/tools/build-bhoo-webxdc.py --gmail-demo --output "$(ROOT)/context/dist/bhoo-gmail.xdc" $(EXTRA_ARGS)

webxdc-android: ## Package the Android XML chat WebXDC demo
	python3 $(ROOT)/tools/build-bhoo-webxdc.py --android-demo --output "$(ROOT)/context/dist/bhoo-android.xdc" $(EXTRA_ARGS)

webxdc-app-tests: ## Build the Bhoo Android app tests WebXDC
	python3 $(ROOT)/tools/build-bhoo-app-tests.py $(EXTRA_ARGS)

# --- Human review loop ---

review-status: ## Show review ledger status and unresolved rejections
	python3 $(ROOT)/tools/bhoo-review.py status

review-package: ## Package a review WebXDC (requires TEST_ID, SPEC, SOURCE, OUT; optional REPAIRS)
	@test -n "$(TEST_ID)" || (echo "Usage: make review-package TEST_ID=... SPEC=... SOURCE=... OUT=... [REPAIRS=...]"; exit 1)
	@test -n "$(SPEC)" || (echo "Missing SPEC"; exit 1)
	@test -n "$(SOURCE)" || (echo "Missing SOURCE"; exit 1)
	@test -n "$(OUT)" || (echo "Missing OUT"; exit 1)
	python3 $(ROOT)/tools/bhoo-review.py package \
		--test-id "$(TEST_ID)" \
		--spec "$(SPEC)" \
		--source "$(SOURCE)" \
		--output "$(OUT)" \
		$(if $(REPAIRS),--repairs "$(REPAIRS)",) \
		$(EXTRA_ARGS)

# --- Cleanup ---

clean: ## Remove all Cargo build artifacts
	cargo clean

clean-dev: ## Reclaim dev/test build artifacts while keeping release builds
	cargo clean --profile dev
