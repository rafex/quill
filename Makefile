# Makefile — build and code-quality targets only.
# For operational tasks (run services, DB, MQTT) use: just <recipe>

CARGO ?= cargo

.DEFAULT_GOAL := help

include scripts/build.mk
include scripts/lint.mk

.PHONY: help
help: ## Show available targets
	@echo "Usage: make <target>"
	@echo ""
	@grep -hE '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "  For operational tasks run: just --list"
