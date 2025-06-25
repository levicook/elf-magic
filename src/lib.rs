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

    // Deduplicate programs by manifest_path to avoid duplicate constants
    let included_programs = deduplicate_programs(included_programs);

    builder::build_programs(&included_programs)?;

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

/// Deduplicate programs by manifest_path to handle cases where multiple workspaces
/// discover the same program (e.g., shared dependencies)
fn deduplicate_programs(programs: Vec<SolanaProgram>) -> Vec<SolanaProgram> {
    use std::collections::HashMap;

    let mut seen: HashMap<PathBuf, SolanaProgram> = HashMap::new();

    for program in programs {
        // Use manifest_path as the key, keeping the first occurrence
        seen.entry(program.manifest_path.clone()).or_insert(program);
    }

    seen.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_deduplicate_programs_removes_duplicates_by_manifest_path() {
        // Same program appearing twice (realistic scenario from multiple workspaces)
        let program1 = SolanaProgram {
            package_name: "apl-token".to_string(),
            target_name: "apl_token".to_string(),
            manifest_path: PathBuf::from("/repo/token/Cargo.toml"),
        };

        let program2 = SolanaProgram {
            package_name: "apl-token".to_string(),
            target_name: "apl_token".to_string(),
            manifest_path: PathBuf::from("/repo/token/Cargo.toml"), // Same path!
        };

        let escrow_program = SolanaProgram {
            package_name: "escrow_program".to_string(),
            target_name: "escrow_program".to_string(),
            manifest_path: PathBuf::from("/repo/examples/escrow/program/Cargo.toml"),
        };

        // Input: 3 programs (with 1 duplicate)
        let input_programs = vec![escrow_program.clone(), program1, program2];
        assert_eq!(input_programs.len(), 3);

        // After deduplication: should have 2 unique programs
        let deduplicated = deduplicate_programs(input_programs);
        assert_eq!(
            deduplicated.len(),
            2,
            "Should deduplicate to 2 unique programs"
        );

        // Verify we have one of each program type
        let apl_token_count = deduplicated
            .iter()
            .filter(|p| p.target_name == "apl_token")
            .count();
        assert_eq!(
            apl_token_count, 1,
            "Should have exactly 1 apl_token after deduplication"
        );

        let escrow_count = deduplicated
            .iter()
            .filter(|p| p.target_name == "escrow_program")
            .count();
        assert_eq!(escrow_count, 1, "Should have exactly 1 escrow_program");
    }

    #[test]
    fn test_deduplicate_programs_preserves_unique_programs() {
        let program1 = SolanaProgram {
            package_name: "counter".to_string(),
            target_name: "counter_program".to_string(),
            manifest_path: PathBuf::from("/repo/examples/counter/program/Cargo.toml"),
        };

        let program2 = SolanaProgram {
            package_name: "escrow".to_string(),
            target_name: "escrow_program".to_string(),
            manifest_path: PathBuf::from("/repo/examples/escrow/program/Cargo.toml"),
        };

        // Input: 2 unique programs
        let input_programs = vec![program1, program2];
        let deduplicated = deduplicate_programs(input_programs);

        // Should preserve both programs
        assert_eq!(deduplicated.len(), 2, "Should preserve all unique programs");
    }
}
