# 🏗️ Architecture

**How elf-magic works under the hood**

elf-magic uses standard Rust build patterns to generate ELF constants at compile time. This document explains the complete build process and how your code is generated.

## Overview

elf-magic follows the same pattern used by tools like `bindgen`, `prost`, and other code generators:

1. **Build script runs** during `cargo build`
2. **Programs are discovered and built** via `cargo build-sbf`
3. **Code is generated** to `$OUT_DIR/generated.rs`
4. **Environment variable is set** pointing to generated file
5. **Your `src/lib.rs` includes** the generated code

This keeps your source tree clean and avoids race conditions with tools like Rust Analyzer.

## Build Flow

### 1. Build Script Execution

When you run `cargo build`, Rust executes your `build.rs`:

```rust
// build.rs
fn main() {
    elf_magic::build().unwrap();
}
```

### 2. Program Discovery

elf-magic uses `cargo metadata` to discover all Solana programs in your workspace(s):

```bash
# Internally runs:
cargo metadata --format-version 1
```

This finds all crates with `crate-type = ["cdylib"]` - the marker for Solana programs.

### 3. Program Building

For each discovered program, elf-magic builds the `.so` file:

```bash
# For each program, runs:
cargo build-sbf --package program-name
```

Built programs are cached and only rebuilt when source files change (incremental builds).

### 4. Code Generation

Generated code is written to `$OUT_DIR/generated.rs`:

```rust
// $OUT_DIR/generated.rs (generated at build time)
pub const TOKEN_MANAGER_ELF: &[u8] = include_bytes!(env!("TOKEN_MANAGER_ELF_PATH"));
pub const GOVERNANCE_ELF: &[u8] = include_bytes!(env!("GOVERNANCE_ELF_PATH"));

pub fn elves() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("token_manager", TOKEN_MANAGER_ELF),
        ("governance", GOVERNANCE_ELF),
    ]
}
```

### 5. Environment Variables

elf-magic sets these environment variables for the main compilation:

- `ELF_MAGIC_GENERATED_PATH` - Path to generated code file
- `TOKEN_MANAGER_ELF_PATH` - Path to each program's `.so` file
- `GOVERNANCE_ELF_PATH` - etc.

### 6. Your Source Code

Your hand-written `src/lib.rs` includes the generated code:

```rust
// src/lib.rs (hand-written)
//! ELF binaries for my Solana programs.

include!(env!("ELF_MAGIC_GENERATED_PATH"));
```

## File Layout

```
your-workspace/
├── target/
│   └── deploy/                    # Built .so files
│       ├── token_manager.so       # Built by cargo build-sbf
│       └── governance.so
├── my-elves/                      # Your ELF crate
│   ├── build.rs                   # elf_magic::build()
│   ├── Cargo.toml
│   ├── src/lib.rs                 # Hand-written
│   └── target/
│       └── debug/
│           └── build/
│               └── my-elves-*/
│                   └── out/
│                       └── generated.rs    # Generated code
└── programs/
    ├── token-manager/
    └── governance/
```

## Why This Pattern?

### Standard Rust Convention

This follows the same pattern as major Rust codegen tools:

- **bindgen** generates bindings to `$OUT_DIR`
- **prost** generates protobuf code to `$OUT_DIR`  
- **sqlx** generates query metadata to `$OUT_DIR`

### Clean Source Tree

- No generated files in `src/`
- No `.gitignore` entries needed
- No conflicts with Rust Analyzer

### Incremental Builds

elf-magic tracks:

- Source file changes in program directories
- Cargo.toml changes
- Build script changes

Only rebuilds what actually changed.

### Race Condition Free

Previous versions that wrote to `src/lib.rs` had race conditions:

- Rust Analyzer would scan half-written files
- Multiple builds could conflict
- Required `--allow-dirty` for publishing

The `$OUT_DIR` pattern eliminates these issues.

## Configuration Processing

### Mode Detection

elf-magic reads your package metadata to determine the mode:

```toml
[package.metadata.elf-magic]
mode = "magic"  # or "permissive" or "laser-eyes"
```

No metadata = Magic Mode (default).

### Workspace Processing

Depending on mode:

- **Magic Mode** - Single workspace, auto-discovery
- **Permissive Mode** - Multi-workspace with exclusions
- **Laser Eyes Mode** - Explicit target lists

### Pattern Matching

For modes with exclusions/inclusions, elf-magic applies glob patterns:

```toml
deny = ["target:test*", "package:*-demo"]
```

Patterns are matched against target names, package names, and paths.

## Error Handling

### Build Failures

If a program fails to build, it's excluded from generated code:

```rust
// Program 'broken-program' failed to build - excluded from generated code
pub const WORKING_PROGRAM_ELF: &[u8] = include_bytes!(env!("WORKING_PROGRAM_ELF_PATH"));

pub fn elves() -> Vec<(&'static str, &'static [u8])> {
    vec![
        ("working_program", WORKING_PROGRAM_ELF),
        // 'broken_program' not included
    ]
}
```

Build errors are shown in console output but don't fail the overall build.

### Missing Dependencies

If `cargo build-sbf` is not available, elf-magic provides helpful error messages pointing to Solana CLI installation instructions.

## Environment Variables

You can control elf-magic behavior with environment variables:

- `ELF_MAGIC_VERBOSE=1` - Enable verbose logging
- `ELF_MAGIC_CACHE_DIR` - Override cache directory
- `ELF_MAGIC_NO_CACHE=1` - Disable incremental builds

## Debugging

### Inspect Generated Code

View the generated code:

```bash
# Find the generated file
find target -name "generated.rs" -path "*/out/*"

# View contents
cat target/debug/build/my-elves-*/out/generated.rs
```

### Build Logs

Enable verbose output:

```bash
ELF_MAGIC_VERBOSE=1 cargo build
```

This shows:

- Program discovery details
- Build commands executed
- Cache hit/miss information
- Generated file paths

### Force Rebuild

Clear cache to force full rebuild:

```bash
cargo clean
cargo build
```

## Performance

### Incremental Builds

First build:
- Discovers programs: ~100ms
- Builds N programs: ~30s per program
- Generates code: ~10ms

Subsequent builds (no changes):
- Cache validation: ~50ms
- Code generation: ~10ms

### Caching Strategy

elf-magic caches:

- Program metadata
- Built `.so` files
- File modification times

Cache keys include source file hashes, so changes are automatically detected.

## Comparison to Other Tools

### vs Manual Building

**Manual approach:**
```bash
cargo build-sbf --package token-manager
cargo build-sbf --package governance
# Manually copy .so files and write includes...
```

**elf-magic approach:**
```bash
cargo build  # Handles everything automatically
```

### vs Build Scripts

**Custom build script:**
```rust
// Lots of manual workspace discovery
// Manual cargo build-sbf invocation
// Manual file tracking for incremental builds
// Manual code generation
```

**elf-magic:**
```rust
fn main() { elf_magic::build().unwrap(); }
```

## Future Architecture

Planned improvements:

- **Parallel builds** - Build multiple programs simultaneously
- **Cross-compilation** - Support different target platforms
- **Custom builders** - Plugin system for non-Solana programs
- **Binary optimization** - Automatic UPX compression, etc. 