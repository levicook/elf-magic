# 🪄 Magic Mode

**Magic Mode** is the default mode for `elf-magic` - it automatically discovers and builds all Solana programs in your workspace with zero configuration.

## Overview

Magic mode embodies the "it just works" philosophy:
- **Zero config required** - works out of the box
- **Auto-discovery** - finds all Solana programs automatically  
- **Single workspace** - perfect for most projects
- **Default behavior** - no `[package.metadata.elf-magic]` needed

## Configuration

### Default (No Config)
```toml
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[build-dependencies]
elf-magic = "0.3"
```

### Explicit Magic Mode
```toml
[package]
name = "my-elves" 
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "magic"

[build-dependencies]
elf-magic = "0.3"
```

## How It Works

1. **Workspace Discovery**: Runs `cargo metadata` in current directory
2. **Program Detection**: Finds all crates with `crate-type = ["cdylib"]`
3. **Automatic Building**: Runs `cargo build-sbf` on each program
4. **Code Generation**: Creates constants for all successfully built programs

## Generated Output

For a workspace with `token_manager` and `governance` programs:

```rust
// Auto-generated in src/lib.rs
pub const TOKEN_MANAGER_ELF: &[u8] = include_bytes!(env!("TOKEN_MANAGER_ELF_PATH"));
pub const GOVERNANCE_ELF: &[u8] = include_bytes!(env!("GOVERNANCE_ELF_PATH"));

pub fn elves() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("token_manager", TOKEN_MANAGER_ELF),
        ("governance", GOVERNANCE_ELF),
    ]
}
```

## When to Use Magic Mode

✅ **Perfect for:**
- Single workspace repositories
- Development and prototyping
- Projects where you want all programs built
- Getting started quickly
- Most Solana projects

❌ **Not ideal for:**
- Multi-workspace repositories
- When you need to exclude specific programs
- Complex build scenarios requiring fine control
- Production builds where you only want specific programs

## Workspace Structure

Magic mode works with any standard Rust workspace:

```
my-project/
├── Cargo.toml              # Workspace root
├── my-elves/               # Your ELF crate
│   ├── build.rs            # elf_magic::generate().unwrap();
│   ├── Cargo.toml          # Magic mode config (or none)
│   └── src/lib.rs          # Auto-generated
└── programs/
    ├── token-manager/      # Solana program
    │   ├── Cargo.toml      # crate-type = ["cdylib"]
    │   └── src/lib.rs
    └── governance/         # Another Solana program
        ├── Cargo.toml      # crate-type = ["cdylib"]
        └── src/lib.rs
```

## Build Output

Magic mode provides rich console output:

```bash
$ cargo build
Mode: magic (1 workspace specified)

Workspace: ./Cargo.toml
  + token_manager
  + governance

Generated lib.rs with 2 Solana programs
   Compiling token-manager v0.1.0
   [... cargo build-sbf output ...]
   Compiling governance v0.1.0  
   [... cargo build-sbf output ...]
   Compiling my-elves v0.1.0
   Finished dev [unoptimized + debuginfo] target(s)
```

## Limitations

- **Single workspace only** - can't span multiple `Cargo.toml` workspaces
- **No exclusions** - builds every Solana program found
- **No fine control** - all-or-nothing approach

For more control, consider [Permissive Mode](permissive.md) or [Laser Eyes Mode](laser-eyes.md).

## Troubleshooting

### No programs found
```bash
⚠️  No Solana programs found - generated empty lib.rs
```
**Solution**: Ensure your programs have `crate-type = ["cdylib"]` in their `Cargo.toml`

### Build failures
If some programs fail to build, they'll be excluded from the generated code with helpful error messages in the build status comments.

---

**Next Steps:**
- Need to exclude specific programs? → [Permissive Mode](permissive.md)
- Want to target only specific programs? → [Laser Eyes Mode](laser-eyes.md)
- Ready to use your generated constants? → [Usage Guide](../usage.md) 