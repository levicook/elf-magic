# 🎛️ Permissive Mode

**Permissive Mode** gives you explicit control over workspace discovery and program exclusions. Perfect for multi-workspace repositories and complex build scenarios.

## Overview

Permissive mode provides fine-grained control:
- **Multi-workspace support** - specify multiple `Cargo.toml` workspaces
- **Flexible denials** - deny programs by pattern
- **Global + local denials** - apply denials across all workspaces or per-workspace
- **Explicit configuration** - everything is specified, nothing is assumed

## Basic Configuration

```toml
[package.metadata.elf-magic]
mode = "permissive"
workspaces = [
    { manifest_path = "./Cargo.toml" }
]
```

## Advanced Configuration

### With Exclusions

```toml
[package.metadata.elf-magic]
mode = "permissive"
workspaces = [
    { manifest_path = "./Cargo.toml", deny = ["target:test*", "package:dev*"] },
    { manifest_path = "examples/basic/Cargo.toml", deny = ["target:*_example"] }
]
```

### With Global Exclusions

```toml
[package.metadata.elf-magic]
mode = "permissive"
global_deny = ["package:apl-token", "target:*_test"] 
workspaces = [
    { manifest_path = "./Cargo.toml", deny = ["target:dev*"] },
    { manifest_path = "examples/Cargo.toml" }
]
```

## Exclusion Patterns

Permissive mode supports three pattern types:

### Target Patterns
Match against the target name (what becomes the constant):
```toml
deny = ["target:test*", "target:*_example", "target:benchmark_*"]
```

### Package Patterns  
Match against the package name:
```toml
deny = ["package:dev*", "package:*-test", "package:example-*"]
```

### Path Patterns
Match against the manifest path:
```toml
deny = ["path:*/examples/*", "path:*/tests/*", "path:*/benchmarks/*"]
```

### Pattern Syntax
- `*` - matches any number of characters
- `?` - matches a single character
- Standard glob patterns supported

## Global vs Local Exclusions

### Global Exclusions
Applied to **all workspaces**:
```toml
global_deny = ["package:*-test", "target:bench*"]
```

### Local Exclusions  
Applied to **specific workspace**:
```toml
{ manifest_path = "./Cargo.toml", deny = ["target:dev*"] }
```

### Merging Behavior
Global and local exclusions are **merged** for each workspace:
- Global: `["package:*-test"]`
- Local: `["target:dev*"]` 
- **Effective**: `["package:*-test", "target:dev*"]`

## Field Aliases

Both field names are supported for flexibility:

```toml
# These are equivalent:
{ manifest_path = "./Cargo.toml", deny = ["target:test*"] }
{ manifest_path = "./Cargo.toml", exclude = ["target:test*"] }
```

## Multi-Workspace Example

Perfect for projects like Arch Network (main workspace + examples):

```toml
[package.metadata.elf-magic]
mode = "permissive"
global_deny = ["package:*-test", "target:bench*"]
workspaces = [
    { manifest_path = "./Cargo.toml", deny = ["target:example*"] },
    { manifest_path = "examples/basic/Cargo.toml", deny = ["target:*_demo"] },
    { manifest_path = "examples/advanced/Cargo.toml" },
    { manifest_path = "tests/integration/Cargo.toml", deny = ["path:*/integration/*"] }
]
```

## Build Output

Permissive mode shows detailed exclusion information:

```bash
$ cargo build
Mode: permissive (3 workspaces specified)

Workspace: ./Cargo.toml
  + token_manager
  + governance
  - test_program (denied by pattern)
- benchmark_suite (denied by pattern)

Workspace: examples/basic/Cargo.toml
  + swap_example
  - basic_demo (denied by pattern)

Generated lib.rs with 3 Solana programs
```

## When to Use Permissive Mode

✅ **Perfect for:**
- Multi-workspace repositories  
- Projects with test/example programs to exclude
- Complex monorepos (like Anza/Agave with 50+ programs)
- Production builds needing specific exclusions
- When you need explicit control over what gets built

❌ **Overkill for:**
- Simple single-workspace projects
- Quick prototyping
- When you want all programs included

## Workspace Structure

Typical multi-workspace setup:

```
arch-network/
├── Cargo.toml                    # Main workspace
├── elves/
│   ├── build.rs                  # elf_magic::generate().unwrap();
│   └── Cargo.toml                # Permissive config
├── programs/                     # Main programs
│   ├── orderbook/
│   ├── apl-token/               # Excluded via global_deny
│   └── governance/
└── examples/                     # Separate workspaces
    ├── basic/
    │   └── Cargo.toml            # Independent workspace
    └── advanced/
        └── Cargo.toml            # Another independent workspace
```

## Error Handling

### Missing Workspaces Field
```toml
[package.metadata.elf-magic]
mode = "permissive"
# Missing workspaces field - will error
```
**Error**: `Invalid elf-magic config: missing field 'workspaces'`

### Invalid Patterns
Invalid patterns are warned about but don't fail the build:
```
Warning: Invalid deny pattern 'invalid-pattern'. Use 'target:', 'package:', or 'path:' prefix.
```

## Troubleshooting

### Workspace not found
**Error**: `Failed to obtain package metadata`  
**Solution**: Verify `manifest_path` points to valid `Cargo.toml`

### Unexpected exclusions
Use verbose patterns and check global vs local exclusions are correctly configured.

### No programs included
If all programs are excluded, you'll get an empty `lib.rs` with warning.

---

**Next Steps:**
- Need even more precision? → [Laser Eyes Mode](laser-eyes.md)
- Want simpler auto-discovery? → [Magic Mode](magic.md)  
- Ready to use your generated constants? → [Usage Guide](../usage.md) 