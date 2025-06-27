# =============================================================================
# CI Targets
# =============================================================================

.PHONY: ci
ci:
	@echo "🚀 Running comprehensive CI validation..."
	$(MAKE) fmt-check
	$(MAKE) check
	$(MAKE) test
	@echo "All binaries clippy:"
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "Publish dry run:"
	cargo publish --dry-run --allow-dirty
	@echo "✅ CI passed"

# Release validation - comprehensive checks before publishing
.PHONY: release-validation
release-validation:
	@echo "🚀 Running release validation..."
	@echo "Verifying tag matches Cargo.toml version..."
	@if [ -n "$$TAG_VERSION" ] && [ -n "$$(grep '^version = ' Cargo.toml | sed 's/version = \"\(.*\)\"/\1/')" ]; then \
		CARGO_VERSION=$$(grep '^version = ' Cargo.toml | sed 's/version = \"\(.*\)\"/\1/'); \
		if [ "$$TAG_VERSION" != "$$CARGO_VERSION" ]; then \
			echo "❌ Tag version $$TAG_VERSION doesn't match Cargo.toml version $$CARGO_VERSION"; \
			exit 1; \
		fi; \
		echo "✅ Tag version matches Cargo.toml version: $$TAG_VERSION"; \
	fi
	$(MAKE) ci
	@echo "✅ Release validation passed"

# Publish to crates.io (requires CARGO_REGISTRY_TOKEN)
.PHONY: publish
publish:
	@echo "📦 Publishing to crates.io..."
	cargo publish --token $$CARGO_REGISTRY_TOKEN
	@echo "✅ Published to crates.io"

# Dogfooding - Use our own tool for releases! 🎯
.PHONY: release-patch release-minor release-major
release-patch:
	@./scripts/release.sh patch

release-minor:
	@./scripts/release.sh minor

release-major:
	@./scripts/release.sh major

# Ecosystem package releases (local prep + GitHub automation)
.PHONY: prepare-ecosystem-release prep ecosystem-list
prepare-ecosystem-release:
	@if [ -z "$(ECOSYSTEM)" ] || [ -z "$(VERSION)" ]; then \
		echo "❌ Usage: make prepare-ecosystem-release ECOSYSTEM=<name> VERSION=<version>"; \
		echo "   Example: make prepare-ecosystem-release ECOSYSTEM=solana-spl-token VERSION=3.4.0"; \
		echo ""; \
		echo "This prepares the ecosystem package locally. Then:"; \
		echo "  git push origin main ecosystem/<name>/v<version>"; \
		echo "  → Triggers GitHub workflow for validation & publication"; \
		exit 1; \
	fi
	@./scripts/prepare-ecosystem-release $(ECOSYSTEM) $(VERSION)

# Short alias for convenience
prep: prepare-ecosystem-release

ecosystem-list:
	@echo "📦 Available ecosystem packages:"
	@find ecosystem -maxdepth 1 -type d -not -name ecosystem | sed 's|ecosystem/|  - |' | sort

# =============================================================================
# Testing
# =============================================================================

.PHONY: test
test: build
	@echo "Running unit tests (no fixtures)..."
	cargo test --lib

# =============================================================================
# Code Quality
# =============================================================================

.PHONY: fmt
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all
	@echo "✅ Code formatted"

.PHONY: fmt-check
fmt-check:
	@echo "🎨 Checking code formatting..."
	cargo fmt --all -- --check
	@echo "✅ Code formatting OK"

# =============================================================================
# Development
# =============================================================================

.PHONY: check
check:
	@echo "🔍 Checking workspace..."
	cargo check --workspace
	@echo "✅ Workspace check passed"

.PHONY: build
build:
	@echo "Building elfmagic library..."
	cargo build

.PHONY: clean
clean:
	cargo clean

# Default target
.PHONY: help
help:
	@echo "ElfMagic Development Commands:"
	@echo ""
	@echo "Testing:"
	@echo "  make test             Run unit tests (fast)"
	@echo ""
	@echo "CI & Quality:"
	@echo "  make ci               Comprehensive CI validation (recommended)"
	@echo "  make fmt              Format code"
	@echo "  make fmt-check        Check code formatting"
	@echo ""
	@echo "Release:"
	@echo "  make publish            Publish to crates.io"
	@echo "  make release-major      Release major version (breaking changes)"
	@echo "  make release-minor      Release minor version (new features)"
	@echo "  make release-patch      Release patch version (bug fixes)"
	@echo "  make release-validation  Complete release validation"
	@echo ""
	@echo "Ecosystem:"
	@echo "  make ecosystem-list                    List available ecosystem packages"
	@echo "  make prepare-ecosystem-release ECOSYSTEM=<name> VERSION=<version>"
	@echo "  make prep ECOSYSTEM=<name> VERSION=<version>  (alias for prepare-ecosystem-release)"
	@echo ""
	@echo "Development:"
	@echo "  make check            Check code without building"
	@echo "  make build            Build project"
	@echo "  make clean            Clean build artifacts and fixtures"

.DEFAULT_GOAL := help 