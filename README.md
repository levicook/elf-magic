# elf-magic ✨

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
fn main() { elf_magic::generate().unwrap(); }

cargo build  # magic ✨
``` -->

**Add to Existing Workspace**

```bash
# Create an ELF crate in your workspace
cargo new my-elves --lib

# Add to my-elves/Cargo.toml
[build-dependencies]
elf-magic = "0.2"

# Add to my-project-elves/build.rs
fn main() { elf_magic::generate().unwrap(); }

cargo build  # magic ✨
```

## What You Get

After building, your ELF crate exports generated constants for every Solana program in your workspace:

**On your first build**, you'll see rich reporting:

```bash
$ cargo build
Mode: magic (1 workspace specified)

Workspace: ./Cargo.toml
  + token_manager
  + governance

Generated lib.rs with 2 Solana programs
   Compiling token-manager v0.1.0
   [... normal cargo build-sbf output ...]
   Compiling governance v0.1.0
   [... normal cargo build-sbf output ...]
   Compiling my-elves v0.1.0
   Finished dev [unoptimized + debuginfo] target(s)
```

The `+` shows included programs, `-` shows excluded programs. If you have exclusions:

```bash
Workspace: ./Cargo.toml
  + token_manager
  + governance
  - test_program (excluded by pattern)
```

```rust
// Generated in src/lib.rs - never edit this file!
pub const TOKEN_MANAGER_ELF: &[u8] = include_bytes!(env!("TOKEN_MANAGER_ELF_PATH"));
pub const GOVERNANCE_ELF: &[u8] = include_bytes!(env!("GOVERNANCE_ELF_PATH"));

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

1. **Configuration**: Load mode (Magic or Pedantic) from `Cargo.toml`
2. **Workspace Loading**: Use `cargo metadata` to discover workspace(s)
3. **Program Discovery**: Filter for crates with `crate-type = ["cdylib"]` - those are your Solana programs
4. **Build Orchestration**: Run `cargo build-sbf` on each program automatically
5. **Code Generation**: Transform program names into clean Rust constants and generate your entire `src/lib.rs`
6. **Incremental Builds**: Set up `cargo:rerun-if-changed` so rebuilds only happen when needed

Behind the scenes:

- 🔍 **Auto-discovery**: `cargo metadata` finds all workspace members
- 🎯 **Smart filtering**: `crate-type = ["cdylib"]` identifies Solana programs
- 🔨 **Automatic building**: `cargo build-sbf` runs when source changes
- 📝 **Code generation**: Target names become `TARGET_NAME_ELF` constants
- ⚡ **Incremental**: Only rebuilds what changed
- 🧙‍♂️ **Zero config**: Works with any workspace layout out of the box

## Configuration: Magic vs Pedantic

`elf-magic` has two modes that handle different workspace patterns you'll find in the wild:

### Magic Mode (Default)

**Perfect for single-workspace repos like Anza/Agave (52 programs in main workspace)**

```bash
cargo build  # Just works, zero config ✨
```

Magic mode runs `cargo metadata` in your current workspace and builds every Solana program it finds. This is perfect for most projects where all programs live in one workspace.

### Pedantic Mode

**Essential for multi-workspace repos like Arch Network (5 programs in main + 14 example workspaces)**

```toml
# my-project-elves/Cargo.toml
[package.metadata.elf-magic]
mode = "pedantic"
workspaces = [
    { manifest_path = "./Cargo.toml" },
    { manifest_path = "examples/basic/Cargo.toml" },
    { manifest_path = "examples/advanced/Cargo.toml", exclude = ["target:test*"] }
]
```

Pedantic mode gives you explicit control over exactly which workspaces to process and which programs to exclude. Essential when you have:

- Multiple independent Cargo workspaces
- Example workspaces separate from main workspace
- Test programs you want to exclude
- Fine-grained control requirements

## Workspace Structure

Works with any layout. Here are the patterns we've tested:

**Single Workspace (Magic Mode)**

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

**Multi-Workspace (Pedantic Mode)**

```
arch-network/
├── Cargo.toml            # Main workspace (5 programs)
├── elves/
│   ├── build.rs          # elf_magic::generate().unwrap();
│   └── Cargo.toml        # Pedantic config
├── programs/             # Main programs
│   ├── orderbook/
│   └── apl-token/
└── examples/             # Separate workspaces
    ├── basic/
    │   └── Cargo.toml    # Independent workspace
    └── advanced/
        └── Cargo.toml    # Another independent workspace
```

## Advanced Usage: Exclude Patterns

Sometimes you want to exclude specific programs. Use exclude patterns with prefixes:

```toml
[package.metadata.elf-magic]
mode = "pedantic"
workspaces = [
    {
        manifest_path = "./Cargo.toml",
        exclude = [
            "target:test*",           # Exclude by target name
            "package:*deprecated*",   # Exclude by package name
            "path:*/examples/broken/*" # Exclude by manifest path
        ]
    }
]
```

**Pattern Types:**

- **`target:pattern`** - Match against the target name (from `[[bin]]` or `[lib]`)
- **`package:pattern`** - Match against the package name
- **`path:pattern`** - Match against the full manifest path

**Pattern Syntax:**

- `*` matches any characters: `test*` matches `test_program`, `testing`, etc.
- `?` matches single character: `test?` matches `test1`, `testa`, but not `test12`
- Standard glob patterns supported

**Common Patterns:**

```toml
# Exclude all test programs
exclude = ["target:test*", "target:*test*"]

# Exclude development packages
exclude = ["package:dev*", "package:*experimental*"]

# Exclude specific paths
exclude = ["path:*/examples/*", "path:*/deprecated/*"]

# Mix and match
exclude = [
    "target:test*",
    "package:dev*",
    "path:*/broken/*"
]
```

## Real-World Examples

**Anza/Agave Pattern** (52 programs, single workspace)

```bash
# Zero config needed
cargo new elves --lib
echo 'fn main() { elf_magic::generate().unwrap(); }' > elves/build.rs
cd elves && cargo build
```

**Arch Network Pattern** (5 main + 14 example workspaces)

```toml
[package.metadata.elf-magic]
mode = "pedantic"
workspaces = [
    { manifest_path = "./Cargo.toml" },
    { manifest_path = "examples/basic/program/Cargo.toml" },
    { manifest_path = "examples/cpi/program/Cargo.toml" },
    # ... 12 more example workspaces
]
```

Without pedantic mode, you'd only get the 5 programs from the main workspace. With pedantic mode, you get all 19 programs across all workspaces.

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
- Solana CLI tools (`cargo build-sbf` must work)
- Workspace with Solana programs (crates with `crate-type = ["cdylib"]`)
