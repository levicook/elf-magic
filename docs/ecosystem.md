# Ecosystem Packages 🌟

**Get popular Solana program binaries instantly - no build tools, no wait time, just add the dependency.**

Ecosystem packages are pre-compiled ELF exports for popular Solana programs. Instead of building programs from source every time, just add a dependency and get instant access to production-ready program binaries.

## Why Ecosystem Packages?

**🚀 Zero Build Time** - No more waiting for `cargo build-sbf`  
**🔒 Version Locked** - Exact mainnet program versions  
**🛠️ No Dependencies** - No Solana CLI tools required  
**✅ CI/CD Friendly** - Fast, reproducible builds

## Quick Start

Add an ecosystem package to your project:

```toml
[dependencies]
elf-magic-solana-spl-token = "3.4.0"
```

Use the ELF binaries immediately:

```rust
use elf_magic_solana_spl_token::{SPL_TOKEN_PROGRAM_ELF, SPL_TOKEN_P_TOKEN_ELF};

// Deploy the standard SPL Token program
let token_program_id = deploy_program(SPL_TOKEN_PROGRAM_ELF)?;

// Or use the Pinocchio-optimized version for better performance
let p_token_program_id = deploy_program(SPL_TOKEN_P_TOKEN_ELF)?;
```

That's it! No build configuration, no Solana CLI setup, no waiting.

## Available Packages

### SPL Token Program

```toml
[dependencies]
elf-magic-solana-spl-token = "3.4.0"
```

**What's included:**

- `SPL_TOKEN_PROGRAM_ELF` - Standard SPL Token program
- `SPL_TOKEN_P_TOKEN_ELF` - Pinocchio-optimized version

**Program ID:** `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`  
**Upstream:** [solana-program/token](https://github.com/solana-program/token)  
**Mainnet Version:** 3.4.0

## Usage Examples

### Testing

```rust
use elf_magic_solana_spl_token::SPL_TOKEN_PROGRAM_ELF;

#[test]
fn test_token_operations() {
    let program_id = deploy_program_to_test_validator(SPL_TOKEN_PROGRAM_ELF);
    // Your token tests here...
}
```

### Deployment Scripts

```rust
use elf_magic_solana_spl_token::SPL_TOKEN_P_TOKEN_ELF;

fn deploy_optimized_token_program() -> Result<Pubkey, Error> {
    // Deploy the performance-optimized version
    deploy_program(SPL_TOKEN_P_TOKEN_ELF)
}
```

### Integration with your elf-magic generated programs

```rust
// Mix ecosystem packages with your own programs
use my_programs::{MY_DEX_ELF, MY_VAULT_ELF};  // Your elf-magic generated programs
use elf_magic_solana_spl_token::SPL_TOKEN_PROGRAM_ELF;  // Ecosystem package

fn deploy_full_stack() -> Result<(), Error> {
    let token_program = deploy_program(SPL_TOKEN_PROGRAM_ELF)?;
    let dex_program = deploy_program(MY_DEX_ELF)?;
    let vault_program = deploy_program(MY_VAULT_ELF)?;

    // Configure your programs to work together
    Ok(())
}
```

## Comparison: Build vs Ecosystem Package

**Traditional approach:**

```bash
# Takes 2-5 minutes, requires Solana CLI
git clone https://github.com/solana-program/token
cd token
cargo build-sbf --package spl-token
# Copy .so files around manually...
```

**Ecosystem package approach:**

```toml
# Takes 5 seconds, works anywhere
[dependencies]
elf-magic-solana-spl-token = "3.4.0"
```

## FAQ

**Q: Are ecosystem packages safe?**  
A: Yes! They contain identical binaries to what you'd build from source. All builds happen transparently in GitHub CI with full audit logs.

**Q: How do I know which version to use?**  
A: Use the version matching your target deployment. Ecosystem package versions exactly match upstream program versions.

**Q: What if I need a different version?**  
A: Open an issue requesting the specific version. We can publish additional versions as needed.

**Q: Can I use ecosystem packages with my own programs?**  
A: Absolutely! Mix and match ecosystem packages with your own elf-magic generated programs.

**Q: Do ecosystem packages work without elf-magic?**  
A: Yes! Ecosystem packages are standalone crates. You can use them in any Rust project.

**Q: How are versions kept in sync?**  
A: Automated release process tracks upstream git tags and rebuilds when new versions are released.

## Roadmap

**Available now:**

- ✅ SPL Token (Original)
- ✅ SPL Token 2022
- ✅ SPL Associated Token Account
- ✅ Solana Memo
- ✅ Solana Stake

**Coming soon:**

- System Program
- Stake Pool Program
- Address Lookup Table Program
- Metaplex programs
- Pyth Oracle programs
- Your suggestions! (Open an issue)

---

## Appendix: For Maintainers

_This section is for people who maintain ecosystem packages._

### Three Distinct Workflows

There are **three separate workflows** for ecosystem package management:

1. **[Publishing existing packages](#publishing-existing-packages)** - Release already-configured packages to crates.io
2. **[Updating packages to new versions](#updating-packages-to-new-versions)** - Sync with new upstream releases
3. **[Adding new packages](#adding-new-packages)** - Bootstrap new ecosystem packages

---

## Publishing Existing Packages

**Use case:** You have a properly configured ecosystem package and want to publish it to crates.io.

**Prerequisites:** Package is already configured with correct Cargo.toml, tests, and build.rs.

### Step 1: Validate the Package

```bash
make validate-ecosystem-package MANIFEST_PATH=ecosystem/solana-stake/Cargo.toml
```

This runs:

- Build verification
- Test execution
- Clippy validation
- Publish dry run

### Step 2: Release and Publish

```bash
./scripts/release-ecosystem-package solana-stake 1.0.0
```

This script will:

- Re-run validation
- Create git tag: `ecosystem/solana-stake/v1.0.0`
- Push tag to trigger GitHub Actions
- GitHub Actions publishes to crates.io

**That's it.** GitHub Actions handles the actual publishing.

---

## Updating Packages to New Versions

**Use case:** Upstream has released a new version and you want to update the ecosystem package.

**Example:** Updating `solana-spl-token` from `3.4.0` → `3.5.0`

### Step 1: Update to New Upstream Version

```bash
./scripts/update-ecosystem-package solana-spl-token 3.5.0
```

This script:

- Cleans submodule state
- Checks out upstream tag `v3.5.0`
- Updates package version in Cargo.toml
- Updates description

### Step 2: Regenerate and Review

```bash
cd ecosystem/solana-spl-token
cargo build
# Review generated src/lib.rs - verify expected constants
```

### Step 3: Update Tests (if needed)

```bash
# Update tests/integration.rs if new programs were added
```

### Step 4: Validate

```bash
make validate-ecosystem-package MANIFEST_PATH=ecosystem/solana-spl-token/Cargo.toml
```

### Step 5: Commit and Release

```bash
git add ecosystem/solana-spl-token/upstream ecosystem/solana-spl-token/Cargo.toml
git commit -m "Update elf-magic-solana-spl-token to v3.5.0"
./scripts/release-ecosystem-package solana-spl-token 3.5.0
```

---

## Adding New Packages

**Use case:** Creating a brand new ecosystem package for a Solana program.

**Example:** Adding support for `your-program`

### Step 1: Create Package Structure

```bash
mkdir -p ecosystem/your-program/src
touch ecosystem/your-program/src/lib.rs
```

### Step 2: Add Upstream Submodule

```bash
git submodule add https://github.com/upstream/repo ecosystem/your-program/upstream
```

### Step 3: Explore Upstream Programs

```bash
# Find Solana programs in the upstream repository
cargo metadata --no-deps --manifest-path ./ecosystem/your-program/upstream/Cargo.toml --format-version 1 | jq '.packages[] | select(.targets[0].crate_types[] == "cdylib") | {name: .name, version: .version}'
```

### Step 4: Create Cargo.toml

```toml
[package]
name = "elf-magic-your-program"
version = "1.0.0"
edition = "2021"
description = "Pre-built ELF exports for Your Program v1.0.0"
license = "MIT"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./upstream/Cargo.toml", only = ["target:your_program"] }
]

[build-dependencies]
elf-magic = { version = "0.4" }
```

### Step 5: Create Build Script

```bash
cat > ecosystem/your-program/build.rs << 'EOF'
fn main() {
    elf_magic::generate().unwrap();
}
EOF
```

### Step 6: Create Tests

```bash
cat > ecosystem/your-program/tests/integration.rs << 'EOF'
use elf_magic_your_program::*;

#[test]
fn validate_elf_constants() {
    assert!(!YOUR_PROGRAM_ELF.is_empty());
    assert_eq!(&YOUR_PROGRAM_ELF[0..4], b"\x7fELF");
}
EOF
```

### Step 7: Test Configuration

```bash
cargo build --package elf-magic-your-program
cargo test --package elf-magic-your-program
```

### Step 8: Add to Workspace and Commit

```bash
# Add to root Cargo.toml workspace.members (already done with ecosystem/* pattern)
git add ecosystem/your-program/
git commit -m "Add elf-magic-your-program ecosystem package"
```

### Step 9: Validate and Release

```bash
make validate-ecosystem-package MANIFEST_PATH=ecosystem/your-program/Cargo.toml
./scripts/release-ecosystem-package your-program 1.0.0
```

---

### GitHub Actions Workflow

All ecosystem releases use **maintainer-controlled + GitHub validation**:

- **Maintainers:** Configure packages, create tags, push
- **GitHub Actions:** Validate, publish to crates.io, create releases

**No manual publishing to crates.io.** Everything goes through GitHub Actions for consistency and security.

---

### Contributing

**Criteria for new ecosystem packages:**

- Program widely used in Solana ecosystem
- Stable tagged releases available
- Builds successfully with `cargo build-sbf`
- Well-maintained upstream repository

**Common challenges:**

- **Duplicate targets:** Use path-based discovery (`path:*/program/*`)
- **Native programs:** Some contain only client code (can't be built)

Open an issue to discuss adding new packages.
