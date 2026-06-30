# Build targets — included by root Makefile
# Variables: CARGO (default: cargo), FEATURES (optional extra --features)

.PHONY: build build-release build-onnx test test-onnx clean doc

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
