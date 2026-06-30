# Lint and format targets — included by root Makefile

.PHONY: check clippy fmt fmt-check audit

check: ## Fast syntax check (no codegen)
	$(CARGO) check

clippy: ## Run Clippy linter (warnings as errors)
	$(CARGO) clippy -- -D warnings

clippy-fix: ## Run Clippy and apply safe fixes automatically
	$(CARGO) clippy --fix --allow-staged

fmt: ## Format all source files
	$(CARGO) fmt

fmt-check: ## Check formatting without modifying files (CI)
	$(CARGO) fmt -- --check

ci: fmt-check clippy test ## Full CI gate: format + lint + tests
