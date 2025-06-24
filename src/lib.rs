pub mod builder;
pub mod codegen;
pub mod domain;
pub mod io;

use domain::{ElfMagicError, GenerationResult};
use std::{env, path::PathBuf};

/// Main entry point for elf-magic code generation
///
/// This function orchestrates the entire process:
/// 1. Parse manifest configuration from package.metadata.elf-magic
/// 2. Discover workspace and find Solana programs
/// 3. Build each program with cargo build-sbf
/// 4. Generate clean Rust code for ELF exports
/// 5. Write the generated lib.rs file
/// 6. Set up incremental build dependencies
pub fn generate() -> Result<GenerationResult, ElfMagicError> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map_err(|e| {
            ElfMagicError::WorkspaceDiscovery(format!("CARGO_MANIFEST_DIR not set: {}", e))
        })?;

    let cargo_target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| env::temp_dir().join("elf-magic-target"));

    let manifest_config = io::parse_manifest_config(&cargo_manifest_dir)?;

    let workspace = io::discover_workspace(&cargo_manifest_dir, &manifest_config)?;

    let programs = workspace.find_solana_programs();

    builder::build_programs(&cargo_target_dir, &programs)?;

    io::write_lib_file(&cargo_manifest_dir, &programs)?;

    io::setup_incremental_builds(&programs)?;

    Ok(GenerationResult::new(programs))
}

/// This function is only available when the "testing" feature is enabled.
#[cfg(feature = "testing")]
pub fn generate_with_manifest_dir(
    cargo_manifest_dir: &str,
) -> Result<GenerationResult, ElfMagicError> {
    temp_env::with_var("CARGO_MANIFEST_DIR", Some(cargo_manifest_dir), generate)
}
