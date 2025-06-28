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
	@echo "Core package publish dry run:"
	cargo publish --package elf-magic --dry-run --allow-dirty
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
	@echo "📦 Publishing core elf-magic package to crates.io..."
	cargo publish --package elf-magic --token $$CARGO_REGISTRY_TOKEN
	@echo "✅ Published elf-magic to crates.io"

# Dogfooding - Use our own tool for releases! 🎯
.PHONY: release-patch release-minor release-major
release-patch:
	@./scripts/release patch

release-minor:
	@./scripts/release minor

release-major:
	@./scripts/release major

# Ecosystem package releases (maintainer-controlled + GitHub validation)
.PHONY: ecosystem-list validate-ecosystem-package publish-ecosystem-package parse-ecosystem-tag install-solana create-ecosystem-github-release

ecosystem-list:
	@echo "📦 Available ecosystem packages:"
	@find ecosystem -maxdepth 1 -type d -not -name ecosystem | sed 's|ecosystem/|  - |' | sort

# Validate ecosystem package (used by CI)
validate-ecosystem-package:
	@if [ -z "$(MANIFEST_PATH)" ]; then \
		echo "❌ MANIFEST_PATH required (e.g., ecosystem/solana-spl-token/Cargo.toml)"; \
		exit 1; \
	fi
	@if [ -n "$(VERSION)" ]; then \
		echo "🔍 Verifying tag version matches Cargo.toml..."; \
		CARGO_VERSION=$$(grep '^version = ' "$(MANIFEST_PATH)" | sed 's/version = "\(.*\)"/\1/'); \
		if [ "$(VERSION)" != "$$CARGO_VERSION" ]; then \
			echo "❌ Tag version $(VERSION) doesn't match Cargo.toml version $$CARGO_VERSION"; \
			exit 1; \
		fi; \
		echo "✅ Tag version matches Cargo.toml version: $(VERSION)"; \
	fi
	@echo "🔍 Reporting solana version..."
	solana --version
	@echo "🔨 Building ecosystem package..."
	cargo build --manifest-path "$(MANIFEST_PATH)"
	@echo "🧪 Running tests (includes ELF validation)..."
	cargo test --manifest-path "$(MANIFEST_PATH)"
	@echo "📎 Running clippy..."
	cargo clippy --manifest-path "$(MANIFEST_PATH)"
	@echo "📦 Publish dry run..."
	cargo publish --manifest-path "$(MANIFEST_PATH)" --dry-run --no-verify --allow-dirty
	@echo "✅ Ecosystem package validation passed"

# Publish ecosystem package to crates.io (used by CI)
publish-ecosystem-package:
	@if [ -z "$(MANIFEST_PATH)" ] || [ -z "$(CARGO_REGISTRY_TOKEN)" ]; then \
		echo "❌ MANIFEST_PATH and CARGO_REGISTRY_TOKEN required"; \
		exit 1; \
	fi
	@echo "🔨 Building ecosystem package for publication..."
	cargo build --manifest-path "$(MANIFEST_PATH)"
	@echo "📦 Publishing to crates.io..."
	cargo publish --manifest-path "$(MANIFEST_PATH)" --token "$(CARGO_REGISTRY_TOKEN)"
	@echo "✅ Published to crates.io!"


# Parse ecosystem tag (used by CI)
parse-ecosystem-tag:
	@if [ -z "$(TAG)" ]; then \
		echo "❌ TAG required"; \
		echo "   Example: make parse-ecosystem-tag TAG=ecosystem/solana-spl-token/v3.4.0"; \
		exit 1; \
	fi
	@./scripts/ci/parse-tag "$(TAG)"

# Install Solana CLI (used by CI)
install-solana:
	@./scripts/ci/install-solana

# Create ecosystem GitHub release (used by CI)
create-ecosystem-github-release:
	@if [ -z "$(ECOSYSTEM_NAME)" ] || [ -z "$(PACKAGE_NAME)" ] || [ -z "$(VERSION)" ] || [ -z "$(TAG)" ]; then \
		echo "❌ ECOSYSTEM_NAME, PACKAGE_NAME, VERSION, and TAG required"; \
		echo "   Example: make create-ecosystem-github-release ECOSYSTEM_NAME=solana-spl-token PACKAGE_NAME=elf-magic-solana-spl-token VERSION=3.4.0 TAG=ecosystem/solana-spl-token/v3.4.0"; \
		exit 1; \
	fi
	@./scripts/ci/create-github-release "$(ECOSYSTEM_NAME)" "$(PACKAGE_NAME)" "$(VERSION)" "$(TAG)"

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
	@echo "  make release-validation Complete release validation"
	@echo ""
	@echo "Ecosystem:"
	@echo "  make ecosystem-list     List available ecosystem packages"
	@echo ""
	@echo "Ecosystem CI:"
	@echo "  make create-ecosystem-github-release ECOSYSTEM_NAME=<name> PACKAGE_NAME=<name> VERSION=<version> TAG=<tag>"
	@echo "  make install-solana                 Install Solana CLI"
	@echo "  make parse-ecosystem-tag TAG=<tag>  Parse ecosystem release tag"

	@echo "  make publish-ecosystem-package MANIFEST_PATH=<path>   Publish ecosystem package"
	@echo "  make validate-ecosystem-package MANIFEST_PATH=<path>  Validate ecosystem package"
	@echo ""
	@echo "Development:"
	@echo "  make check            Check code without building"
	@echo "  make build            Build project"
	@echo "  make clean            Clean build artifacts and fixtures"

.DEFAULT_GOAL := help 