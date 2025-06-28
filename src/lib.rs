mod builder;
mod codegen;
pub mod config;
mod error;
mod programs;
mod workspace;

use std::{env, path::PathBuf};

use crate::{
    config::Config,
    error::Error,
    programs::{deduplicate_programs, BuildResults, SolanaProgram},
};

#[deprecated(note = "use build() instead")]
pub fn generate() -> Result<BuildResults, Error> {
    build()
}

/// Generate Rust bindings for Solana programs
pub fn build() -> Result<BuildResults, Error> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map_err(|e| Error::WorkspaceDiscovery(format!("CARGO_MANIFEST_DIR not set: {}", e)))?;

    // Clean pipeline: load → discover → build → generate → save
    let config = Config::load(&cargo_manifest_dir)?;
    let workspaces = workspace::load_workspaces(&cargo_manifest_dir, &config)?;
    let discovered_programs = workspaces
        .iter()
        .map(|w| w.discover_programs())
        .collect::<Result<Vec<_>, _>>()?;

    // Extract and deduplicate included programs
    let included_programs: Vec<SolanaProgram> = discovered_programs
        .iter()
        .flat_map(|w| w.included.iter().cloned())
        .collect();
    let included_programs = deduplicate_programs(included_programs);

    // Build, generate, and save
    let build_result = builder::build_programs(&included_programs);
    let code = codegen::generate(&build_result)?;
    codegen::save(&cargo_manifest_dir, &code)?;

    builder::enable_incremental_builds(&included_programs)?;

    Ok(BuildResults::new(
        config.mode_name().to_string(),
        discovered_programs,
    ))
}
