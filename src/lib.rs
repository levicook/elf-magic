pub mod builder;
pub mod codegen;
pub mod domain;
pub mod io;

use domain::{ElfMagicError, GenerationResult};
use std::env;

/// Main entry point for elf-magic code generation
///
/// This function orchestrates the entire process:
/// 1. Parse discovery configuration from package.metadata.elf-magic
/// 2. Discover workspace and find Solana programs
/// 3. Build each program with cargo build-sbf
/// 4. Generate clean Rust code for ELF exports
/// 5. Write the generated lib.rs file
/// 6. Set up incremental build dependencies
pub fn generate() -> Result<GenerationResult, ElfMagicError> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").map_err(|e| {
        ElfMagicError::WorkspaceDiscovery(format!("CARGO_MANIFEST_DIR not set: {}", e))
    })?;

    let discovery_config = io::parse_discovery_config(&cargo_manifest_dir)?;
    let workspace = io::discover_workspace(&discovery_config)?;
    let programs = workspace.find_solana_programs();

    if programs.is_empty() {
        return Ok(GenerationResult::empty());
    }

    let built_programs = workspace.build_programs(&programs)?;
    let generated_code = codegen::generate_lib_code(&built_programs);

    io::write_lib_file(&generated_code)?;
    io::setup_incremental_builds(&programs)?;

    Ok(GenerationResult::new(built_programs, generated_code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_empty_workspace() {
        // TODO: Test with workspace containing no Solana programs
        // Should return empty GenerationResult
    }

    #[test]
    fn test_generate_with_programs() {
        // TODO: Test full generation flow with mock programs
    }
}
