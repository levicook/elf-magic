# elf-magic ✨

> Automatic compile-time ELF exports for Solana programs. One-liner integration, zero config, just works.

Stop wrestling with Solana program builds. `elf-magic` automatically discovers all your programs, builds them, and generates clean Rust code so your ELF bytes are always available as constants.

## Quick Start

```bash
# Create an ELF crate in your workspace
cargo new my-elves --lib
```

Add to `my-elves/Cargo.toml`:

```toml
[build-dependencies]
elf-magic = "0.5"
```

Add to `my-elves/build.rs`:

```rust
fn main() { elf_magic::build().unwrap(); }
```

```bash
cargo build  # magic ✨
```

## What You Get

After building, your ELF crate exports constants for every Solana program in your workspace:

**Your hand-written `src/lib.rs`:**

```rust
//! ELF binaries for my Solana programs.

include!(env!("ELF_MAGIC_GENERATED_PATH"));
```

**Generated at build time (in `$OUT_DIR`):**

```rust
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

## Three Modes for Every Workflow

### 🪄 [Magic Mode](docs/modes/magic.md) (Default)

**Zero config, just works**

```bash
cargo build  # Discovers and builds all programs automatically
```

Perfect for: Single workspaces, development, getting started

### 🎛️ [Permissive Mode](docs/modes/permissive.md)

**Multi-workspace with exclusions**

```toml
[package.metadata.elf-magic]
mode = "permissive"
global_deny = ["package:*-test"]
workspaces = [
    { manifest_path = "./Cargo.toml", deny = ["target:dev*"] },
    { manifest_path = "examples/Cargo.toml" }
]
```

Perfect for: Complex repos, excluding test programs, multi-workspace projects

### 🎯 [Laser Eyes Mode](docs/modes/laser-eyes.md)

**Precision targeting**

```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = ["target:token_manager", "target:governance"] }
]
```

Perfect for: Production builds, CI optimization, focused development

## Rich Build Reporting

First build shows what's happening:

```bash
$ cargo build
Mode: magic (1 workspace specified)

Workspace: ./Cargo.toml
  + token_manager
  + governance

Generated constants with 2 Solana programs
   Compiling token-manager v0.1.0
   Compiling governance v0.1.0
   Compiling my-elves v0.1.0
   Finished dev [unoptimized + debuginfo] target(s)
```

The `+` shows included programs, `-` shows excluded programs.

## How It Works

The magic behind the one-liner:

1. **🔍 Auto-discovery**: `cargo metadata` finds all workspace members
2. **🎯 Smart filtering**: `crate-type = ["cdylib"]` identifies Solana programs
3. **🔨 Automatic building**: `cargo build-sbf` runs when source changes
4. **📝 Code generation**: Target names become `TARGET_NAME_ELF` constants
5. **⚡ Incremental**: Only rebuilds what changed

## Installation

Add to your ELF crate's `Cargo.toml`:

```toml
[build-dependencies]
elf-magic = "0.5"
```

## Documentation

- **🪄 [Magic Mode](docs/modes/magic.md)** - Zero config auto-discovery
- **🎛️ [Permissive Mode](docs/modes/permissive.md)** - Multi-workspace with exclusions
- **🎯 [Laser Eyes Mode](docs/modes/laser-eyes.md)** - Precision targeting
- **📖 [Usage Guide](docs/usage.md)** - Using your generated constants
- **🏗️ [Architecture](docs/architecture.md)** - How it works under the hood

## Examples

Works with any workspace layout:

**Single Workspace** (Magic Mode)

```
my-workspace/
├── Cargo.toml            # Workspace root
├── my-elves/             # Generated ELF exports
│   ├── build.rs          # One-liner magic ✨
│   └── src/lib.rs        # Hand-written wrapper
└── programs/
    ├── token-manager/    # Your Solana programs
    └── governance/
```

**Multi-Workspace** (Permissive/Laser Eyes Mode)

```
arch-network/
├── Cargo.toml            # Main workspace (5 programs)
├── elves/                # ELF exports with advanced config
└── examples/             # Separate workspaces
    ├── basic/Cargo.toml  # Independent workspace
    └── advanced/Cargo.toml
```

## Requirements

- Rust 2021 edition
- [Solana CLI tools](https://docs.solana.com/cli/install-solana-cli-tools) for `cargo build-sbf`

## License

MIT
