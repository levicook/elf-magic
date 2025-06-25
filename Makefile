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
	@echo "Development:"
	@echo "  make check            Check code without building"
	@echo "  make build            Build project"
	@echo "  make clean            Clean build artifacts and fixtures"

.DEFAULT_GOAL := help 