# GitHub Workflows Setup Guide

This repository uses **ultra-simple workflows** powered by **Docker + Makefile** architecture.

## 🏗️ **Architecture: Docker + Makefile = Simple + Powerful**

**Core insight**: Keep GitHub workflows **trivially simple** by moving all complexity into Makefile Docker targets.

```yaml
# GitHub workflows are dead simple:
steps:
  - uses: actions/checkout@v4
  - run: make ci-docker-full
```

```makefile
# Makefile handles Docker complexity:
ci-docker-full:
    docker run --rm -v $(PWD):/workspace -w /workspace \
        rust:latest sh -c "rustup component add clippy rustfmt && make ci-full"
```

**Benefits:**

- ✅ **Dead-simple workflows** - Minimal YAML, maximum power
- ✅ **Local = CI** - Same Docker environment everywhere
- ✅ **Battle-tested tools** - Official Rust Docker image
- ✅ **Zero drift** - One source of truth in Makefile
- ✅ **Easy debugging** - `make ci-docker-full` reproduces CI exactly

## 🔧 **The Two Workflows**

### 1. **CI Workflow** (`.github/workflows/ci.yml`)

**Triggers:** Push to `main`, all pull requests  
**Jobs:**

- **CI Validation** → `make ci` - Comprehensive validation including integration tests
- Uses Docker environment matching production for consistency

### 2. **Release Workflow** (`.github/workflows/release.yml`)

**Triggers:** Version tags (`v0.1.0`, `v1.2.3`, etc.)  
**Jobs:**

1. **Release Validation** → `make release-validation` - Complete validation in Docker
2. **Publish** → `make publish` - Push to crates.io
3. **GitHub Release** - Extract changelog and create release

## 🎯 **Docker Container: rust:latest**

This is the **official Rust Docker image**:

- ✅ **Rust/Cargo** (latest stable)
- ✅ **Fresh dependencies** (no version lock issues)
- ✅ **Consistent environment** (same as CI)

**No manual installation required** - everything just works!

## 🚀 **Makefile Targets**

```bash
# Development (native)
make check           # Fast workspace check
make test            # Unit tests only
make test-all        # Unit + integration tests
make fmt            # Format code
make ci-local       # Local CI with full tools

# Docker CI (matches production)
make ci-docker-quick    # Fast CI in Docker
make ci-docker-full     # Complete CI in Docker
make ci                 # Main CI target (Docker-based)

# Release
make release-validation # Complete release checks
make publish           # Publish to crates.io
```

## ⚙️ **Setup Requirements**

### **Required GitHub Secrets**

**Settings → Secrets and variables → Actions:**

1. **`CARGO_REGISTRY_TOKEN`** (Required for releases)

   ```bash
   # Get from: https://crates.io/me
   # Permissions: "Publish new crates" + "Publish updates"
   ```

2. **`GITHUB_TOKEN`** - Automatically provided, no setup needed

### **Branch Protection**

**Settings → Branches → Add rule** for `main`:

- ✅ Require status checks: `Quick Checks (Native)`, `Docker Validation (Complete)`
- ✅ Require up-to-date branches before merging
- ✅ Include administrators

## 🚀 **Publishing Process**

### **Automatic (Recommended) - Using Our Own Tool! 🎯**

```bash
# 1. Use odo to bump version (patch/minor/major)
odo roll patch    # For bug fixes
# odo roll minor  # For new features
# odo roll major  # For breaking changes

# 2. Update changelog (optional)
vim CHANGELOG.md

# 3. Commit and tag using the version odo just set
VERSION=$(odo show --format version)  # Get current version
git add -A && git commit -m "Release v$VERSION"
git tag v$VERSION && git push origin main v$VERSION

# 4. Watch automation ✨
# → Docker validation
# → crates.io publishing
# → GitHub release creation
```

### **One-liner Release Script**

```bash
# Create a simple release script
cat > scripts/release.sh << 'EOF'
#!/bin/bash
set -e

BUMP_TYPE=${1:-patch}
echo "🚀 Rolling $BUMP_TYPE version with odo..."
odo roll $BUMP_TYPE

VERSION=$(odo show --format version)
echo "📝 Version is now: $VERSION"

echo "💾 Committing and tagging..."
git add -A
git commit -m "Release v$VERSION"
git tag v$VERSION

echo "📤 Pushing to GitHub..."
git push origin main v$VERSION

echo "✅ Release v$VERSION triggered! Check GitHub Actions."
EOF

chmod +x scripts/release.sh

# Usage:
./scripts/release.sh patch   # Bug fix release
./scripts/release.sh minor   # Feature release
./scripts/release.sh major   # Breaking change release
```

### **Legacy Manual Process (Still Works)**

```bash
# 1. Update version manually
vim Cargo.toml

# 2. Update changelog (optional)
vim CHANGELOG.md

# 3. Commit and tag
git add -A && git commit -m "Release v0.2.0"
git tag v0.2.0 && git push origin main v0.2.0
```

## 🐛 **Debugging CI Issues**

**The magic of Docker-based CI:**

```bash
# Reproduce CI failure exactly:
make ci-docker-full

# Or just the quick checks:
make ci-docker-quick

# Debug step by step:
docker run --rm -it -v $(pwd):/workspace -w /workspace \
  rust:latest bash
# Then run individual commands inside container
```

**If `make ci-docker-full` passes locally, CI will pass!**

## 📊 **Local Development Workflow**

```bash
# Fast development loop (native tools)
make check          # Quick validation
make test           # Run tests
make fmt            # Format code

# Before pushing (Docker validation)
make ci-docker-full # Exact CI environment

# Quick Docker check
make ci-docker-quick # Fast Docker validation
```

## 🎯 **Key Design Principles**

1. **Workflows are thin wrappers** - Real logic in Makefile
2. **Docker for consistency** - Same environment everywhere
3. **Official containers** - Battle-tested, maintained by Rust team
4. **Parallel jobs** - Fast feedback + thorough validation
5. **Version validation** - Tag must match `Cargo.toml`

## 🔍 **Quality Standards**

Enforced automatically:

- ✅ **Zero clippy warnings** (`-D warnings`)
- ✅ **Proper formatting** (`cargo fmt --check`)
- ✅ **All tests pass** (27 unit tests + integration)
- ✅ **Documentation builds** cleanly
- ✅ **Publish dry-run** succeeds
- ✅ **Version consistency** (tag ↔ Cargo.toml)

## 🛠️ **Troubleshooting**

### **Common Issues**

**"Tag version doesn't match Cargo.toml"**

```bash
# Check versions match:
git describe --tags           # v0.2.0
grep '^version = ' Cargo.toml  # version = "0.2.0"
```

**"Docker container fails"**

```bash
# Test Docker setup locally:
docker pull rust:latest
make ci-docker-quick
```

**"crates.io token invalid"**

```bash
# Verify secret is set in GitHub:
# Settings → Secrets → CARGO_REGISTRY_TOKEN
```

### **Advanced Debugging**

```bash
# Interactive Docker debugging:
docker run -it --rm -v $(pwd):/workspace -w /workspace \
  rust:latest bash

# Inside container:
make ci-full          # Run full CI
rustc --version       # Check tool versions
cargo clippy          # Test individual commands
```

This architecture ensures **reliable, maintainable, and debuggable** CI/CD! 🎉
