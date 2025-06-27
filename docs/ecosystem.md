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

_This section is for people who want to contribute ecosystem packages or understand the release process._

### Release Process

Ecosystem packages use a **maintainer-controlled + GitHub validation** workflow:

**What maintainers do:**

- Update git submodule to upstream tag
- Update Cargo.toml version and dependencies
- Create commit and tag
- Push to trigger CI

**What GitHub CI does:**

- Validates the package (build, test, clippy)
- Publishes to crates.io (only if validation passes)
- Creates GitHub release with changelog

**Key principle: Maintainers control all file changes. CI only validates and publishes.**

### How to Release an Ecosystem Package

Example: Releasing `solana-spl-token` version `3.5.0`

1. **Sync submodule and update version**:

   ```bash
   ./scripts/sync-ecosystem-package solana-spl-token 3.5.0
   ```

   This cleans the submodule state, checks out the exact upstream tag, and updates the package version.

2. **Generate and review the exported constants**:

   ```bash
   cd ecosystem/solana-spl-token
   cargo build
   # Review generated src/lib.rs - verify expected ELF constants are exported
   ```

3. **Write validation tests** (required):

   ```rust
   // Create/update tests/integration.rs - validation will fail without tests
   use elf_magic_solana_spl_token::*;

   #[test]
   fn validate_elf_constants() {
       assert!(!SPL_TOKEN_PROGRAM_ELF.is_empty());
       assert!(!SPL_TOKEN_P_TOKEN_ELF.is_empty());
       assert_eq!(&SPL_TOKEN_PROGRAM_ELF[0..4], b"\x7fELF");
       assert_eq!(&SPL_TOKEN_P_TOKEN_ELF[0..4], b"\x7fELF");
   }
   ```

4. **Test the package locally**:

   ```bash
   # Build and test to verify basic functionality
   cargo build --package elf-magic-solana-spl-token
   cargo test --package elf-magic-solana-spl-token

   # Full validation (including publish dry run) happens in CI
   ```

5. **Commit, tag, and push**:

   ```bash
   # Stage the submodule commit hash first
   git add ecosystem/solana-spl-token/upstream

   # Stage other changes
   git add ecosystem/solana-spl-token/Cargo.toml ecosystem/solana-spl-token/tests/

   # Verify what you're committing
   git status
   git diff --cached

   git commit -m "Release elf-magic-solana-spl-token v3.5.0"
   git tag ecosystem/solana-spl-token/v3.5.0
   git push origin main ecosystem/solana-spl-token/v3.5.0
   ```

CI will validate, publish to crates.io, and create a GitHub release.

### Adding New Ecosystem Packages

1. **Create package structure:**

```bash
mkdir -p ecosystem/your-program-name/src
touch ecosystem/your-program-name/src/lib.rs
```

2. **Add git submodule:**

```bash
git submodule add https://github.com/upstream/repo ecosystem/your-program-name/upstream
```

3. **Explore the upstream programs:**

```bash
# Find Solana programs (packages with "cdylib" crate type)
cargo metadata --no-deps --manifest-path ./ecosystem/your-program-name/upstream/Cargo.toml --format-version 1 | jq '.packages[] | select(.targets[0].crate_types[] == "cdylib") | {name: .name, version: .version, crate_types: .targets[0].crate_types}'

# Note: Some repositories may have duplicate program names across different packages.
# In this case, you'll need to use path-based discovery and override constants.
```

4. **Create Cargo.toml** with elf-magic configuration (replace the target names):

```toml
[package]
name = "elf-magic-your-program-name"
version = "1.0.0"
edition = "2021"
description = "Pre-built ELF exports for Your Program v1.0.0"
license = "MIT"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    # Simple case: use target names
    { manifest_path = "./upstream/Cargo.toml", only = ["target:your_program"] },

    # Complex case: use path patterns for repositories with duplicate targets
    # { manifest_path = "./upstream/Cargo.toml", only = ["path:*/program/*", "path:*/other-program/*"] },
]

# For complex cases with duplicates, override constant names
# [package.metadata.elf-magic.constants]
# "./upstream/program/Cargo.toml" = "YOUR_MAIN_PROGRAM_ELF"
# "./upstream/other-program/Cargo.toml" = "YOUR_OTHER_PROGRAM_ELF"

[build-dependencies]
elf-magic = { version = "0.4" }
```

5. **Create supporting files:**

```bash
# Create build.rs to invoke elf-magic
cat > ecosystem/your-program-name/build.rs << 'EOF'
fn main() {
    elf_magic::generate().unwrap();
}
EOF

# Write failing tests to define expectations based on step 3 research
cat > ecosystem/your-program-name/tests/integration.rs << 'EOF'
use elf_magic_your_program_name::*;

#[test]
fn validate_elf_constants() {
    // TODO: Replace with actual expected constants from step 3
    // assert!(!YOUR_PROGRAM_ELF.is_empty());
    // assert_eq!(&YOUR_PROGRAM_ELF[0..4], b"\x7fELF");
}
EOF
```

6. **Test the package:**

```bash
# Build to verify configuration works
cargo build --package elf-magic-your-program-name

# Run tests to verify ELF constants are valid
cargo test --package elf-magic-your-program-name

# Note: Full validation (including publish dry run) happens in CI
# where git submodules are properly checked out
```

### Contributing

We welcome ecosystem packages for popular Solana programs! Open an issue or PR to discuss.

**Criteria for inclusion:**

- Program is widely used in the Solana ecosystem
- Program has stable, tagged releases
- Program builds successfully with `cargo build-sbf`
- Upstream repository is well-maintained

**Common challenges:**

- **Duplicate targets**: Some repositories contain multiple programs with the same name. Use path-based discovery (`path:*/program/*`) and override constants. See the [elf-magic configuration docs](../README.md) for details.
- **Native programs**: Some repositories contain only client code for native/system programs (like compute-budget). These can't be built with `cargo build-sbf`.
