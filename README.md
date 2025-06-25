# elf-magic ✨

> **⚠️ UNDER CONSTRUCTION ⚠️**  
> **This crate is in active development and is not well tested yet.**  
> **APIs may change without notice. Use at your own risk.**

> It just works, don't ask how.

Actually, fine - here's how it works: You get a build.rs one-liner that generates compile-time ELF exports for every Solana program in your workspace.

Stop wrestling with Solana program builds. `elf-magic` automatically discovers all your programs, builds them, and generates clean Rust code so your ELF bytes are always available as constants.

## Quick Start

<!--  FUTURE GOAL:
**Option 1: Start Fresh**

```bash
cargo install cargo-generate
cd my-workspace/
cargo generate levicook/elf-magic my-elves
cd my-elves
cargo build  # magic ✨
```

**Option 2: Add to Existing Workspace**

```bash
# Create an ELF crate in your workspace
cargo new my-elves --lib

# Add to my-elves/Cargo.toml
[build-dependencies]
elf-magic = "0.1"

# Add to my-project-elves/build.rs
fn main() { elf_magic::generate(); }

cargo build  # magic ✨
``` -->

**Add to Existing Workspace**

```bash
# Create an ELF crate in your workspace
cargo new my-elves --lib

# Add to my-elves/Cargo.toml
[build-dependencies]
elf-magic = { git = "https://github.com/levicook/elf-magic.git" }

# Add to my-project-elves/build.rs
fn main() { elf_magic::generate(); }

cargo build  # magic ✨
```

## What You Get

After building, your ELF crate exports generated constants for every Solana program in your workspace:

**On your first build**, you'll see:

```bash
$ cargo build
   Compiling token-manager v0.1.0
   [... normal cargo build-sbf output ...]
   Compiling governance v0.1.0
   [... normal cargo build-sbf output ...]
   Compiling my-elves v0.1.0
   Finished dev [unoptimized + debuginfo] target(s)
```

```rust
// Generated in src/lib.rs - never edit this file!
pub const TOKEN_MANAGER_ELF: &[u8] = include_bytes!(env!("TOKEN_MANAGER_ELF_MAGIC_PATH"));
pub const GOVERNANCE_ELF: &[u8] = include_bytes!(env!("GOVERNANCE_ELF_MAGIC_PATH"));

pub fn elves() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("token_manager", TOKEN_MANAGER_ELF),
        ("governance", GOVERNANCE_ELF),
    ]
}
```

Use your programs anywhere:

```rust
use my_elves::{TOKEN_MANAGER_ELF, GOVERNANCE_ELF};

// Deploy, test, embed - whatever you need
let program_id = deploy_program(TOKEN_MANAGER_ELF)?;
```

## How It Works

The one-liner sounds too good to be true, but here's the magic:

1. **Program Discovery**: `cargo metadata` tells us about every crate in your workspace
2. **Solana Detection**: We filter for crates with `crate-type = ["cdylib"]` - those are your Solana programs
3. **Build Orchestration**: Run `cargo build-sbf` on each program automatically
4. **Code Generation**: Transform program names into clean Rust constants and generate your entire `src/lib.rs`
5. **Incremental Builds**: Set up `cargo:rerun-if-changed` so rebuilds only happen when needed

Behind the scenes:

- 🔍 **Auto-discovery**: `cargo metadata` finds all workspace members
- 🎯 **Smart filtering**: `crate-type = ["cdylib"]` identifies Solana programs
- 🔨 **Automatic building**: `cargo build-sbf` runs when source changes
- 📝 **Code generation**: Program names become `PROGRAM_NAME_ELF` constants
- ⚡ **Incremental**: Only rebuilds what changed
- 🧙‍♂️ **Config Optional**: Works with any workspace layout

## Workspace Structure

Works with any layout, but here's what the template gives you:

```
my-workspace/
├── Cargo.toml            # Workspace root
├── my-elves/             # Generated ELF exports
│   ├── build.rs          # One-liner magic ✨
│   └── src/lib.rs        # Auto-generated, don't edit
└── programs/
    ├── token-manager/    # Your Solana programs
    ├── governance/
    └── whatever-else/
```

## Advanced Usage

Need more control over which programs get built? Configure your ELF crate's `Cargo.toml`:

```toml
# my-project-elves/Cargo.toml
[package.metadata.elf-magic]
include = ["programs/*", "examples/simple-*"]
exclude = ["programs/deprecated-*", "examples/broken-*"]
```

**Common patterns:**

```toml
# Separate production and examples
[package.metadata.elf-magic]
include = ["programs/*"]
exclude = ["examples/*", "tests/*"]

# Only specific programs
[package.metadata.elf-magic]
include = ["programs/token-manager", "programs/governance"]

# Everything except broken ones
[package.metadata.elf-magic]
exclude = ["programs/experimental-*"]
```

Without any config, `elf-magic` discovers and builds everything - perfect for getting started. Add config only when you need it.

## Why elf-magic? The tl;dr

**Before:**

- Run `cargo build-sbf` manually for each program
- Hard-code filesystem paths to .so files in your code
- Get runtime panics when files aren't where you expect
- Context-switch between `cargo` and Solana-specific tooling
- Remember which programs need rebuilding and when
- Hunt down missing program files across environments

**After:**

```bash
cargo build  # Just standard Cargo, always
```

Your ELF bytes are always available as clean, typed constants. No more bespoke build commands. No more tracking file paths. Just `cargo build` and everything works.

The magic happens behind the scenes - `elf-magic` runs `cargo build-sbf` when needed, but you never have to think about it.

## Why elf-magic? The manifesto

### The Real Problem: Missing Engineering Practices

Solana development suffers from a **tooling gap** that makes essential software engineering practices unnecessarily difficult:

**Testing is broken.** Most projects can't easily unit test their program interactions because ELF bytes aren't available at compile time. Developers resort to:

- Hard-coded file paths that break in CI
- Runtime discovery that fails unpredictably
- Skipping integration tests entirely

**Benchmarking is impossible.** You can't benchmark program deployment or interaction patterns when your toolchain can't reliably find program binaries.

**Auditing is compromised.** Security reviews need to verify the exact program bytes being deployed, but most projects have fragile, bespoke build processes that obscure this.

### The Developer Experience Tax

Every Solana project pays this tax:

- **Context switching** between `cargo` and Solana-specific commands
- **Runtime panics** when files aren't where expected
- **Environment-specific builds** that work locally but fail in CI
- **Fragile deployment scripts** that break when paths change

### elf-magic Fixes This

**Clear Rust dependencies.** Your program binaries become compile-time constants with normal Rust visibility and dependency management.

**Standard toolchain.** Just `cargo build`, `cargo test`, `cargo bench`. No special commands, no custom scripts.

**Reliable CI/CD.** Deterministic builds that work the same everywhere.

**Better testing.** Write unit tests that actually test your program interactions:

```rust
#[test]
fn test_token_manager_deployment() {
    let program_id = deploy_program(my_elves::TOKEN_MANAGER_ELF)?;
    let mint = create_mint(&program_id)?;
    assert!(mint.is_initialized());
}
```

**Professional auditing.** Auditors can verify exact program bytes with confidence.

elf-magic doesn't just automate builds - it enables the software engineering practices that make Solana projects reliable, testable, and maintainable.

## Requirements

- Rust toolchain
- Solana CLI tools (`cargo install-sbf` must work)
- Workspace with Solana programs (crates with `crate-type = ["cdylib"]`)
