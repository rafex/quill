# Build targets — included by root Makefile
# Variables: CARGO (default: cargo), FEATURES (optional extra --features)

WASM_PACK ?= wasm-pack
CARGO_BIN ?= $(shell find $$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin -name cargo 2>/dev/null | head -1)

.PHONY: build build-release build-onnx test test-onnx clean doc build-wasm build-web

build: ## Compile all services (debug)
	$(CARGO) build

build-release: ## Compile all services (release)
	$(CARGO) build --release

build-onnx: ## Compile search-service with real ONNX/MiniLM embeddings
	$(CARGO) build -p search-service --features onnx-embeddings

build-onnx-release: ## Compile search-service with ONNX (release)
	$(CARGO) build -p search-service --release --features onnx-embeddings

test: ## Run all tests (stub embeddings)
	$(CARGO) test

test-onnx: ## Run search-service tests with ONNX feature
	$(CARGO) test -p search-service --features onnx-embeddings

test-verbose: ## Run all tests with output
	$(CARGO) test -- --nocapture

clean: ## Remove build artifacts
	$(CARGO) clean

doc: ## Generate and open rustdoc
	$(CARGO) doc --no-deps --open

build-wasm: ## Compile quill-wasm to WebAssembly (outputs to web/wasm/pkg/)
	PATH="$(dir $(CARGO_BIN)):$$PATH" $(WASM_PACK) build quill-wasm --target web --out-dir ../web/wasm/pkg

build-web: build-wasm ## Build wasm then compile TypeScript frontend (production)
	npm --prefix web run build
