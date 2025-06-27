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

**Coming soon:**

- SPL Associated Token program
- Metaplex programs
- Pyth Oracle programs
- Your suggestions! (Open an issue)

---

## Appendix: For Maintainers

_This section is for people who want to contribute ecosystem packages or understand the release process._

### Release Process

Ecosystem packages use a **local preparation + GitHub automation** workflow:

```bash
# 1. Prepare release locally (lightweight)
make prepare-ecosystem-release ECOSYSTEM=solana-spl-token VERSION=3.5.0

# 2. Push to trigger GitHub workflow (heavy lifting)
git push origin main ecosystem/solana-spl-token/v3.5.0
```

**Local preparation:**

- Updates git submodule to upstream tag
- Synchronizes Cargo.toml version
- Creates atomic commit + tag

**GitHub automation:**

- ELF generation in clean environment
- Comprehensive validation (tests, clippy)
- Transparent crates.io publication
- GitHub release with changelog

### Adding New Ecosystem Packages

1. **Create package structure:**

```bash
mkdir -p ecosystem/your-program-name
```

2. **Add git submodule:**

```bash
git submodule add https://github.com/upstream/repo ecosystem/your-program-name/upstream
```

3. **Create Cargo.toml** with elf-magic configuration:

```toml
[package]
name = "elf-magic-your-program-name"
version = "1.0.0"
edition = "2021"
description = "Pre-built ELF exports for Your Program v1.0.0"
license = "MIT"

[dependencies]
elf-magic = { path = "../.." }

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./upstream/Cargo.toml", only = ["target:your_program"] },
]

[build-dependencies]
elf-magic = { path = "../.." }
```

4. **Test the release process:**

```bash
make prepare-ecosystem-release ECOSYSTEM=your-program-name VERSION=1.0.0
git push origin main ecosystem/your-program-name/v1.0.0
```

5. **Add to workspace** in root `Cargo.toml`:

```toml
[workspace]
members = [
    ".",
    "ecosystem/solana-spl-token",
    "ecosystem/your-program-name",  # Add here
]
```

### Contributing

We welcome ecosystem packages for popular Solana programs! Open an issue or PR to discuss.

**Criteria for inclusion:**

- Program is widely used in the Solana ecosystem
- Program has stable, tagged releases
- Program builds successfully with `cargo build-sbf`
- Upstream repository is well-maintained
