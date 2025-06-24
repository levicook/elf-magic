use crate::domain::{
    DiscoveryConfig, ElfMagicError, GeneratedCode, SolanaProgram, Workspace, WorkspaceMember,
};
use std::path::{Path, PathBuf};

/// Parse discovery configuration from package.metadata.elf-magic
///
/// Reads the include/exclude patterns from the Cargo.toml metadata section.
pub fn parse_discovery_config(cargo_manifest_dir: &str) -> Result<DiscoveryConfig, ElfMagicError> {
    // TODO: Parse cargo_manifest_dir/Cargo.toml
    // TODO: Extract [package.metadata.elf-magic] section
    // TODO: Parse include and exclude arrays
    // TODO: Return DiscoveryConfig with patterns
    todo!("implement parse_discovery_config")
}

/// Discover workspace using the provided discovery configuration
///
/// This function finds the workspace root and all workspace members,
/// then parses each member's Cargo.toml to extract crate types.
pub fn discover_workspace(discovery_config: &DiscoveryConfig) -> Result<Workspace, ElfMagicError> {
    // TODO: Run `cargo metadata --format-version 1`
    // TODO: Parse JSON output to find workspace root and members
    // TODO: For each member, parse Cargo.toml for crate-type field
    // TODO: IMPORTANT: Sort workspace members by name for stable, predictable output
    // TODO: Return Workspace with members and the provided config
    todo!("implement discover_workspace")
}

/// Write the generated code to src/lib.rs
///
/// Creates the generated lib.rs file with warning comments and
/// all the ELF constant definitions.
pub fn write_lib_file(code: &GeneratedCode) -> Result<(), ElfMagicError> {
    // TODO: Generate file header with "do not edit" warning
    // TODO: Write all constant definitions
    // TODO: Write the all_programs() function
    // TODO: IMPORTANT: Write to temporary file first, then atomically move to src/lib.rs
    //       This prevents partial writes and corruption during generation
    // TODO: IMPORTANT: Run `cargo fmt` on the generated file to prevent accidental
    //       modifications by user's toolchain and ensure consistent formatting
    todo!("implement write_lib_file")
}

/// Set up incremental build dependencies
///
/// Tells cargo to rerun this build script whenever any of the
/// Solana program source files change.
pub fn setup_incremental_builds(programs: &[SolanaProgram]) -> Result<(), ElfMagicError> {
    // TODO: For each program, println!("cargo:rerun-if-changed={}", src_path)
    // TODO: Also watch Cargo.toml files
    // TODO: Set environment variables for each program's .so path
    todo!("implement setup_incremental_builds")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discovery_config() {
        // TODO: Test parsing package.metadata.elf-magic config
        todo!("implement test")
    }

    #[test]
    fn test_discover_workspace() {
        // TODO: Test workspace discovery
        let config = DiscoveryConfig {
            include: vec!["programs/*".to_string()],
            exclude: vec!["programs/deprecated-*".to_string()],
        };

        // let workspace = discover_workspace(&config).unwrap();
        todo!("implement test")
    }

    #[test]
    fn test_discover_workspace_sorted() {
        // TODO: Test that workspace members are sorted by name
        todo!("implement test")
    }

    #[test]
    fn test_write_lib_file() {
        // TODO: Test generating lib file
        todo!("implement test")
    }

    #[test]
    fn test_write_lib_file_atomic() {
        // TODO: Test that file writing is atomic (no partial writes)
        todo!("implement test")
    }

    #[test]
    fn test_setup_incremental_builds() {
        // TODO: Test cargo directive generation
        todo!("implement test")
    }
}
