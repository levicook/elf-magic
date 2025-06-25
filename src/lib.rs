mod builder;
mod codegen;
mod config;
mod error;
mod programs;
mod workspace;

use std::{env, path::PathBuf};

use crate::{
    config::Config,
    error::Error,
    programs::{GenerationResult, SolanaProgram},
};

/// Generate Rust bindings for Solana programs
pub fn generate() -> Result<GenerationResult, Error> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map_err(|e| {
            let message = format!("CARGO_MANIFEST_DIR not set: {}", e);
            Error::WorkspaceDiscovery(message)
        })?;

    let cargo_target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from) // important default, DO NOT CHANGE:
        .unwrap_or_else(|_| env::temp_dir().join("elf-magic-target"));

    // Core pipeline:
    // 1. load config
    // 2. load workspaces
    // 3. discover programs across all workspaces
    // 4. build included programs across all workspaces
    // 5. generate code - one lib.rs from all included programs
    // 6. enable incremental builds for all included programs

    let config = Config::load(&cargo_manifest_dir)?;

    let workspaces = workspace::load_workspaces(&config)?;

    let discovered_programs = workspaces
        .iter()
        .map(|w| w.discover_programs())
        .collect::<Result<Vec<_>, _>>()?;

    // Extract all programs for building and code generation
    let included_programs: Vec<SolanaProgram> = discovered_programs
        .iter()
        .flat_map(|w| w.included.iter().cloned())
        .collect();

    builder::build_programs(&cargo_target_dir, &included_programs)?;

    let code = codegen::generate(&included_programs)?;
    codegen::save(&cargo_manifest_dir, &code)?;

    enable_incremental_builds(&included_programs)?;

    let mode = match &config {
        Config::Magic => "magic".to_string(),
        Config::Pedantic { .. } => "pedantic".to_string(),
    };

    Ok(GenerationResult::new(mode, discovered_programs))
}

/// Enable incremental builds for each program
fn enable_incremental_builds(programs: &[SolanaProgram]) -> Result<(), Error> {
    for program in programs {
        println!("cargo:rerun-if-changed={}", program.manifest_path.display());
    }
    Ok(())
}
