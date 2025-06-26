# 🎯 Laser Eyes Mode

**Laser Eyes Mode** provides precision targeting for Solana program builds - only building exactly the programs you specify. Perfect for focused builds and CI optimization.

## Overview

Laser Eyes mode is all about precision:
- **Include-only filtering** - specify exactly what to build
- **Multi-workspace support** - target programs across multiple workspaces  
- **Pattern matching** - use globs for dynamic program selection
- **Zero noise** - only builds what you explicitly include

## Key Design

**Laser Eyes** = **"Include only these specific programs"**

This is the **opposite** of other modes:
- **Magic**: "Include all programs" 
- **Permissive**: "Include all programs except excludes"
- **Laser Eyes**: "Include only these programs" ✨

## Basic Configuration

```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { 
        manifest_path = "./Cargo.toml", 
        only = ["target:token_manager", "target:governance"] 
    }
]
```

## Multi-Workspace Configuration

```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { 
        manifest_path = "./Cargo.toml", 
        only = ["target:token_manager", "target:governance"] 
    },
    { 
        manifest_path = "examples/defi/Cargo.toml", 
        only = ["target:swap*"] 
    }
]
```

## Pattern Matching

```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { 
        manifest_path = "./Cargo.toml", 
        only = [
            "target:*_core",           # Target names ending with "_core"
            "package:my-*-program",    # Package names like "my-token-program"
            "path:*/core/programs/*"   # Programs in core/programs directories
        ] 
    }
]
```

## Only Patterns

Laser Eyes mode supports three pattern types:

### Target Patterns
Match against the target name (what becomes the constant):
```toml
only = ["target:token*", "target:governance", "target:*_core"]
```

### Package Patterns  
Match against the package name:
```toml
only = ["package:my-token*", "package:*-core", "package:governance-*"]
```

### Path Patterns
Match against the manifest path:
```toml
only = ["path:*/core/programs/*", "path:*/main/programs/*"]
```

### Pattern Syntax
- `*` - matches any number of characters
- `?` - matches a single character  
- Standard glob patterns supported

## Usage Examples

### Example 1: Core Programs Only
```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = ["target:token_manager", "target:governance"] }
]
```

**Result**: Only builds `token_manager` and `governance` programs, ignoring all test programs, examples, etc.

### Example 2: Pattern-Based Selection  
```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = ["target:*_core"] }
]
```

**Result**: Only builds programs with target names ending in `_core` like `token_core`, `swap_core`, etc.

### Example 3: Multi-Workspace Targeting
```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = ["target:token_manager"] },
    { manifest_path = "examples/defi/Cargo.toml", only = ["target:swap_program"] }
]
```

**Result**: Builds `token_manager` from main workspace + `swap_program` from examples workspace.

### Example 4: Empty Include (Build Nothing)
```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = [] }
]
```

**Result**: Builds no programs, generates empty `lib.rs`.

## Build Output

Laser Eyes mode shows precise targeting information:

```bash
$ cargo build
Mode: laser-eyes (2 workspaces specified)

Workspace: ./Cargo.toml
  + token_manager (matched target:token*)
  + governance (explicit match)
  - swap_program (not included)
  - test_program (not included)

Workspace: examples/defi/Cargo.toml  
  + swap_program (matched target:swap*)
  - example_program (not included)

Generated lib.rs with 3 Solana programs
```

## When to Use Laser Eyes Mode

✅ **Perfect for:**
- **Production builds** targeting specific core programs
- **CI/CD optimization** building only changed programs  
- **Development focus** working on subset of programs
- **Testing environments** focusing on particular features
- **Deployment pipelines** with program-specific stages

❌ **Not ideal for:**
- Quick prototyping where you want everything built
- Development where you're unsure what programs you need
- Simple single-program projects (magic mode is easier)

## Workspace Structure

Works with any workspace layout:

```
my-project/
├── Cargo.toml                 # Main workspace
├── elves/
│   ├── build.rs               # elf_magic::generate().unwrap();
│   └── Cargo.toml             # Laser eyes config
├── programs/
│   ├── token-manager/         # ✅ Included via target:token_manager
│   ├── governance/            # ✅ Included via target:governance  
│   ├── test-program/          # ❌ Not included
│   └── benchmark-suite/       # ❌ Not included
└── examples/
    └── defi/
        ├── Cargo.toml         # Separate workspace
        └── programs/
            ├── swap-program/  # ✅ Included via target:swap*
            └── demo-program/  # ❌ Not included
```

## Error Handling

### Missing Include Field
```toml
[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml" }  # Missing only field
]
```
**Error**: `Invalid elf-magic config: missing field 'only'`

### Invalid Patterns
Invalid patterns are warned about but don't fail the build:
```
Warning: Invalid only pattern 'invalid-pattern'. Use 'target:', 'package:', or 'path:' prefix.
```

## Benefits

1. **🎯 Precision Targeting** - Build only what you need
2. **⚡ Faster CI/CD** - Reduce build times in focused environments  
3. **🧭 Development Focus** - Work on specific programs without noise
4. **🔀 Pattern Flexibility** - Use globs for dynamic program selection
5. **🌐 Multi-Workspace** - Target programs across multiple workspaces

## Troubleshooting

### No programs matched
```bash
⚠️  No Solana programs found - generated empty lib.rs
```
**Solutions**:
- Verify only patterns are correct
- Check that target/package/path patterns match actual programs
- Use `cargo metadata` to see available programs

### Unexpected inclusions
- Review pattern matching logic
- Test patterns with simpler glob expressions first
- Verify workspace discovery is finding correct programs

---

**Next Steps:**
- Want to exclude instead of include? → [Permissive Mode](permissive.md)
- Need simpler auto-discovery? → [Magic Mode](magic.md)
- Ready to use your generated constants? → [Usage Guide](../usage.md) 