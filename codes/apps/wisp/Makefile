ROOT := $(abspath ../../..)

.PHONY: help check build build-wayland build-compositor run test webxdc clean

help: ## Show available targets for Wisp (SpectreOS Compositor & Launcher)
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_-]+:.*## / {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

check: ## Type-check library, Wayland preview, and DRM/KMS compositor features
	cargo check --locked
	cargo check --locked --features wayland
	cargo check --locked --features compositor

build: ## Build Wisp library (debug)
	cargo build --locked

build-wayland: ## Build desktop preview Wisp binary (release)
	cargo build --locked --release --features wayland --bin wisp

build-compositor: ## Build unified Wisp DRM/KMS compositor + launcher binary (release)
	cargo build --locked --release --features compositor --bin wisp

run: ## Run desktop preview Wisp window (release)
	cargo run --locked --release --features wayland --bin wisp -- --window

test: ## Run Wisp unit specs
	RUST_MIN_STACK=16777216 cargo test --locked

webxdc: ## Package the Wisp launcher webxdc demo via bhoo-web
	python3 $(ROOT)/tools/build-bhoo-webxdc.py --launcher-demo --output "$(ROOT)/context/dist/bhoo-launcher.xdc"

clean: ## Remove Cargo build artifacts
	cargo clean

.PHONY: build_development
build_development: ## Build optimized development compositor with remote_control
	cargo build --locked --release --features development --bin wisp
