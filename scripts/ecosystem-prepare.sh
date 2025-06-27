#!/bin/bash
set -e

# Ecosystem package preparation script (local, lightweight)
# Usage: ./scripts/ecosystem-prepare.sh <ecosystem-name> <upstream-version>
# Example: ./scripts/ecosystem-prepare.sh solana-spl-token 3.5.0

ECOSYSTEM_NAME=${1}
UPSTREAM_VERSION=${2}

if [[ -z "$ECOSYSTEM_NAME" || -z "$UPSTREAM_VERSION" ]]; then
    echo "❌ Usage: $0 <ecosystem-name> <upstream-version>"
    echo "   Example: $0 solana-spl-token 3.5.0"
    exit 1
fi

ECOSYSTEM_DIR="ecosystem/${ECOSYSTEM_NAME}"
SUBMODULE_DIR="${ECOSYSTEM_DIR}/upstream"

# Validate ecosystem package exists
if [[ ! -d "$ECOSYSTEM_DIR" ]]; then
    echo "❌ Ecosystem package not found: $ECOSYSTEM_DIR"
    exit 1
fi

# Ensure we're in a clean git state
if [[ -n $(git status --porcelain) ]]; then
    echo "❌ Working directory is not clean. Please commit or stash changes first."
    git status --short
    exit 1
fi

echo "🚀 Preparing ecosystem package: $ECOSYSTEM_NAME v$UPSTREAM_VERSION"

# Update git submodule to target version
echo "📎 Updating submodule to v$UPSTREAM_VERSION..."
cd "$SUBMODULE_DIR"

# Fetch latest tags
git fetch --tags

# Check if the tag exists
if ! git rev-parse "v$UPSTREAM_VERSION" >/dev/null 2>&1; then
    echo "❌ Tag v$UPSTREAM_VERSION not found in upstream repository"
    echo "Available tags:"
    git tag --sort=-version:refname | head -10
    exit 1
fi

# Checkout the specific version
git checkout "v$UPSTREAM_VERSION"
cd - > /dev/null

# Update Cargo.toml version to match upstream
echo "📝 Updating Cargo.toml version to $UPSTREAM_VERSION..."
CARGO_TOML="${ECOSYSTEM_DIR}/Cargo.toml"

# Use sed to update version field (cross-platform)
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' "s/^version = \".*\"/version = \"$UPSTREAM_VERSION\"/" "$CARGO_TOML"
else
    # Linux
    sed -i "s/^version = \".*\"/version = \"$UPSTREAM_VERSION\"/" "$CARGO_TOML"
fi

# Quick local validation that structure is sane
echo "🔍 Quick local validation..."
if ! cargo metadata --manifest-path "$CARGO_TOML" --format-version 1 >/dev/null 2>&1; then
    echo "❌ Invalid Cargo.toml after version update"
    exit 1
fi

# Commit the preparation changes
echo "💾 Committing ecosystem preparation..."
git add "$ECOSYSTEM_DIR" "$SUBMODULE_DIR"
git commit -m "ecosystem: Prepare $ECOSYSTEM_NAME v$UPSTREAM_VERSION

- Update submodule to upstream v$UPSTREAM_VERSION
- Update package version to match upstream

Ready for release workflow."

# Create the ecosystem tag
ECOSYSTEM_TAG="ecosystem/${ECOSYSTEM_NAME}/v${UPSTREAM_VERSION}"
git tag "$ECOSYSTEM_TAG"

echo "✅ Ecosystem package prepared!"
echo "🏷️  Tag: $ECOSYSTEM_TAG"
echo ""
echo "Next steps:"
echo "  git push origin main $ECOSYSTEM_TAG"
echo "  → This will trigger the ecosystem release workflow"
echo "  → ELF generation, validation, and publication happen in GitHub" 