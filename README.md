# elf-magic ✨

> It just works, don't ask how.

Actually, fine - here's how it works: You get a build.rs one-linger that generates compile-time ELF exports for every Solana program in your workspace.

Stop wrestling with Solana program builds. `elf-magic` automatically discovers all your programs, builds them, and generates clean Rust code so your ELF bytes are always available as constants.

## Quick Start

**Option 1: Start Fresh**

```bash
cargo install cargo-generate
cargo generate levicook/elf-magic my-awesome-project
cd my-awesome-project
cargo build  # Everything just works ✨
```

**Option 2: Add to Existing Workspace**

```bash
# Create an ELF crate in your workspace
cargo new my-project-elves --lib

# Add to my-project-elves/Cargo.toml
[dependencies]
elf-magic = "0.1"

# Add to my-project-elves/build.rs
fn main() { elf_magic::generate(); }

# Build and profit
cargo build
```

## What You Get

After building, your ELF crate automatically exports constants for every Solana program:

**On your first build**, you'll see:

```bash
$ cargo build
   Compiling example-program v0.1.0
   [... cargo build-sbf output ...]
   Compiling my-awesome-project-elves v0.1.0
   Finished dev [unoptimized + debuginfo] target(s)
```

```rust
// Generated in src/lib.rs - never edit this file!
pub const TOKEN_MANAGER_ELF: &[u8] = include_bytes!(env!("PROGRAM_TOKEN_MANAGER_SO_PATH"));
pub const GOVERNANCE_ELF: &[u8] = include_bytes!(env!("PROGRAM_GOVERNANCE_SO_PATH"));

pub fn all_programs() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("token_manager", TOKEN_MANAGER_ELF),
        ("governance", GOVERNANCE_ELF),
    ]
}
```

Use your programs anywhere:

```rust
use my_project_elves::{TOKEN_MANAGER_ELF, GOVERNANCE_ELF};

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
- 🧙‍♂️ **Zero config**: Works with any workspace layout

## Workspace Structure

Works with any layout, but here's what the template gives you:

```
my-awesome-project/
├── Cargo.toml                    # Workspace root
├── my-awesome-project-elves/     # Generated ELF exports
│   ├── build.rs                  # One-liner magic ✨
│   └── src/lib.rs               # Auto-generated, don't edit
└── programs/
    ├── token-manager/           # Your Solana programs
    ├── governance/
    └── whatever-else/
```

## Why elf-magic?

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

## Requirements

- Rust toolchain
- Solana CLI tools (`cargo install-sbf` must work)
- Workspace with Solana programs (crates with `crate-type = ["cdylib"]`)
