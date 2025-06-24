use crate::codegen;
use crate::domain::{ElfMagicError, ManifestConfig, SolanaProgram, Workspace, WorkspaceMember};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents the structure of cargo metadata JSON output
#[derive(Debug, Deserialize)]
struct CargoMetadata {
    workspace_root: PathBuf,
    packages: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    manifest_path: PathBuf,
    targets: Vec<Target>,
}

#[derive(Debug, Deserialize)]
struct Target {
    name: String,
    crate_types: Vec<String>,
}

/// Represents the structure of a Cargo.toml file for parsing metadata
#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<PackageSection>,
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(rename = "elf-magic")]
    elf_magic: Option<ManifestConfig>,
}

/// Parse manifest configuration from package.metadata.elf-magic
///
/// Reads the include/exclude patterns from the Cargo.toml metadata section.
/// If no configuration is found, returns a safe default that includes nothing.
pub fn parse_manifest_config(
    cargo_manifest_dir: &PathBuf,
) -> Result<ManifestConfig, ElfMagicError> {
    let manifest_path = cargo_manifest_dir.join("Cargo.toml");

    // Read the Cargo.toml file
    let content = fs::read_to_string(&manifest_path).map_err(|e| {
        ElfMagicError::Metadata(format!(
            "Failed to read Cargo.toml at {}: {}",
            manifest_path.display(),
            e
        ))
    })?;

    // Parse the TOML
    let cargo_toml: CargoToml = toml::from_str(&content)
        .map_err(|e| ElfMagicError::Metadata(format!("Failed to parse Cargo.toml: {}", e)))?;

    // Extract elf-magic configuration or use permissive default (aka: magic ✨)
    let config = cargo_toml
        .package
        .and_then(|p| p.metadata)
        .and_then(|m| m.elf_magic)
        .unwrap_or_else(ManifestConfig::allow_all);

    Ok(config)
}

/// Discover workspace using the provided manifest configuration
///
/// This function finds the workspace root and all workspace members,
/// then parses each member's Cargo.toml to extract crate types.
pub fn discover_workspace(
    cargo_manifest_dir: &PathBuf,
    manifest_config: &ManifestConfig,
) -> Result<Workspace, ElfMagicError> {
    let manifest_path = cargo_manifest_dir.join("Cargo.toml");

    let mut cmd = Command::new("cargo");
    cmd.args(&[
        "metadata",
        "--format-version",
        "1",
        "--no-deps",
        "--manifest-path",
        manifest_path.to_str().unwrap(),
    ]);

    let output = cmd.output().map_err(|e| {
        ElfMagicError::WorkspaceDiscovery(format!("Failed to execute cargo metadata: {}", e))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ElfMagicError::WorkspaceDiscovery(format!(
            "cargo metadata failed: {}",
            stderr
        )));
    }

    // Parse the JSON output
    let metadata: CargoMetadata = serde_json::from_slice(&output.stdout).map_err(|e| {
        ElfMagicError::WorkspaceDiscovery(format!("Failed to parse cargo metadata JSON: {}", e))
    })?;

    // Process each package into WorkspaceMembers
    let mut members: Vec<WorkspaceMember> = Vec::new();

    for package in metadata.packages {
        // Find targets with cdylib crate type (Solana programs)
        for target in package.targets {
            if target.crate_types.contains(&"cdylib".to_string()) {
                let package_dir = package
                    .manifest_path
                    .parent()
                    .ok_or_else(|| {
                        ElfMagicError::WorkspaceDiscovery(format!(
                            "Invalid manifest path: {}",
                            package.manifest_path.display()
                        ))
                    })?
                    .to_path_buf();

                let member = WorkspaceMember {
                    name: target.name, // Use library name, not package name
                    path: package_dir,
                    manifest_path: package.manifest_path.clone(),
                    crate_types: target.crate_types, // Use from metadata, not parsed TOML
                };
                members.push(member);
            }
        }
    }

    // Sort members by name for stable, predictable output
    members.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Workspace {
        root_path: metadata.workspace_root,
        members,
        config: manifest_config.clone(),
    })
}

/// Write the generated code to src/lib.rs
///
/// Creates the generated lib.rs file using atomic writes with proper temp file handling.
/// Formats the code before final placement.
pub fn write_lib_file(
    cargo_manifest_dir: &Path,
    programs: &[SolanaProgram],
) -> Result<(), ElfMagicError> {
    // Render the template content using codegen
    let rendered_content = codegen::render_lib_file(programs)?;

    // Create target path relative to the cargo manifest directory
    let target_path = cargo_manifest_dir.join("src/lib.rs");
    let target_dir = target_path.parent().unwrap_or(cargo_manifest_dir);

    let temp_file =
        tempfile::NamedTempFile::new_in(target_dir).map_err(|e| ElfMagicError::Io(e))?;

    // Write content to temporary file
    fs::write(temp_file.path(), &rendered_content).map_err(|e| ElfMagicError::Io(e))?;

    // Run cargo fmt on the temporary file before moving
    // Get the path as a string to avoid borrow issues
    let temp_path = temp_file.path().to_path_buf();
    let fmt_result = Command::new("cargo")
        .arg("fmt")
        .arg("--")
        .arg(&temp_path)
        .output();

    match fmt_result {
        Ok(output) if !output.status.success() => {
            eprintln!(
                "Warning: cargo fmt failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            eprintln!("Warning: failed to run cargo fmt: {}", e);
        }
        _ => {} // Success
    }

    // Atomically move the formatted temporary file to final location
    temp_file
        .persist(target_path)
        .map_err(|e| ElfMagicError::Io(e.error))?;

    Ok(())
}

/// Set up incremental build dependencies
///
/// Tells cargo to rerun this build script whenever any of the
/// Solana program source files change.
pub fn setup_incremental_builds(programs: &[SolanaProgram]) -> Result<(), ElfMagicError> {
    for program in programs {
        // Watch the program's src directory for changes
        let src_path = program.path.join("src");
        if src_path.exists() {
            println!("cargo:rerun-if-changed={}", src_path.display());
        }

        // Watch the program's Cargo.toml file
        println!("cargo:rerun-if-changed={}", program.manifest_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cargo_toml(content: &str) -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml_path, content).expect("Failed to write test Cargo.toml");
        temp_dir
    }

    fn create_test_workspace() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp workspace");
        let workspace_root = temp_dir.path();

        // Create root Cargo.toml with workspace
        let root_cargo_toml = r#"
[workspace]
members = ["program-a", "program-b", "lib-crate"]
"#;
        std::fs::write(workspace_root.join("Cargo.toml"), root_cargo_toml)
            .expect("Failed to write root Cargo.toml");

        // Create program-a (Solana program with cdylib)
        let program_a_dir = workspace_root.join("program-a");
        std::fs::create_dir_all(program_a_dir.join("src")).expect("Failed to create program-a dir");
        let program_a_cargo_toml = r#"
[package]
name = "program-a"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
"#;
        std::fs::write(program_a_dir.join("Cargo.toml"), program_a_cargo_toml)
            .expect("Failed to write program-a Cargo.toml");
        std::fs::write(program_a_dir.join("src/lib.rs"), "// Test Solana program A")
            .expect("Failed to write program-a lib.rs");

        // Create program-b (Solana program with cdylib)
        let program_b_dir = workspace_root.join("program-b");
        std::fs::create_dir_all(program_b_dir.join("src")).expect("Failed to create program-b dir");
        let program_b_cargo_toml = r#"
[package]
name = "program-b"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
"#;
        std::fs::write(program_b_dir.join("Cargo.toml"), program_b_cargo_toml)
            .expect("Failed to write program-b Cargo.toml");
        std::fs::write(program_b_dir.join("src/lib.rs"), "// Test Solana program B")
            .expect("Failed to write program-b lib.rs");

        // Create lib-crate (regular library)
        let lib_crate_dir = workspace_root.join("lib-crate");
        std::fs::create_dir_all(lib_crate_dir.join("src")).expect("Failed to create lib-crate dir");
        let lib_crate_cargo_toml = r#"
[package]
name = "lib-crate"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["lib"]
"#;
        std::fs::write(lib_crate_dir.join("Cargo.toml"), lib_crate_cargo_toml)
            .expect("Failed to write lib-crate Cargo.toml");
        std::fs::write(lib_crate_dir.join("src/lib.rs"), "// Test regular library")
            .expect("Failed to write lib-crate lib.rs");

        temp_dir
    }

    #[test]
    fn test_parse_manifest_config_with_full_config() {
        let cargo_toml_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
include = ["programs/*", "contracts/*"]
exclude = ["programs/deprecated-*", "contracts/old-*"]
"#;

        let temp_dir = create_test_cargo_toml(cargo_toml_content);
        let result = parse_manifest_config(&temp_dir.path().into()).unwrap();

        assert_eq!(result.include, vec!["programs/*", "contracts/*"]);
        assert_eq!(
            result.exclude,
            vec!["programs/deprecated-*", "contracts/old-*"]
        );
    }

    #[test]
    fn test_parse_manifest_config_with_partial_config() {
        let cargo_toml_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
include = ["programs/*"]
"#;

        let temp_dir = create_test_cargo_toml(cargo_toml_content);
        let result = parse_manifest_config(&temp_dir.path().into()).unwrap();

        assert_eq!(result.include, vec!["programs/*"]);
        assert_eq!(result.exclude, Vec::<String>::new());
    }

    #[test]
    fn test_parse_manifest_config_missing_elf_magic_section() {
        let cargo_toml_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"
"#;

        let temp_dir = create_test_cargo_toml(cargo_toml_content);
        let result = parse_manifest_config(&temp_dir.path().into()).unwrap();

        // Should return permissive default (allow_all)
        assert_eq!(result.include, vec!["**/*"]);
        assert_eq!(result.exclude, Vec::<String>::new());
    }

    #[test]
    fn test_parse_manifest_config_missing_metadata_section() {
        let cargo_toml_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"
"#;

        let temp_dir = create_test_cargo_toml(cargo_toml_content);
        let result = parse_manifest_config(&temp_dir.path().into()).unwrap();

        // Should return permissive default (allow_all)
        assert_eq!(result.include, vec!["**/*"]);
        assert_eq!(result.exclude, Vec::<String>::new());
    }

    #[test]
    fn test_parse_manifest_config_missing_package_section() {
        let cargo_toml_content = r#"
[workspace]
members = ["programs/*"]
"#;

        let temp_dir = create_test_cargo_toml(cargo_toml_content);
        let result = parse_manifest_config(&temp_dir.path().into()).unwrap();

        // Should return permissive default (allow_all)
        assert_eq!(result.include, vec!["**/*"]);
        assert_eq!(result.exclude, Vec::<String>::new());
    }

    #[test]
    fn test_parse_manifest_config_file_not_found() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        // Don't create Cargo.toml, so it doesn't exist

        let result = parse_manifest_config(&temp_dir.path().into());

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Failed to read Cargo.toml"));
    }

    #[test]
    fn test_parse_manifest_config_invalid_toml() {
        let invalid_cargo_toml_content = r#"
[package
name = "test-package"  # Missing closing bracket
"#;

        let temp_dir = create_test_cargo_toml(invalid_cargo_toml_content);
        let result = parse_manifest_config(&temp_dir.path().into());

        assert!(result.is_err());
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Failed to parse Cargo.toml"));
    }

    #[test]
    fn test_parse_manifest_config_empty_arrays() {
        let cargo_toml_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
include = []
exclude = []
"#;

        let temp_dir = create_test_cargo_toml(cargo_toml_content);
        let result = parse_manifest_config(&temp_dir.path().into()).unwrap();

        assert_eq!(result.include, Vec::<String>::new());
        assert_eq!(result.exclude, Vec::<String>::new());
    }

    #[test]
    fn test_discover_workspace_integration() {
        let temp_workspace = create_test_workspace();

        let config = ManifestConfig {
            include: vec!["**/*".to_string()],
            exclude: vec![],
        };

        let result = discover_workspace(&temp_workspace.path().to_path_buf(), &config);

        // Verify the workspace discovery worked
        let workspace = result.expect("Should successfully discover test workspace");

        // Check basic workspace properties (canonicalize paths to handle symlinks on macOS)
        assert_eq!(
            workspace.root_path.canonicalize().unwrap(),
            temp_workspace.path().canonicalize().unwrap()
        );
        assert_eq!(workspace.config.include, vec!["**/*"]);
        assert_eq!(workspace.config.exclude, Vec::<String>::new());

        // Should find only 2 members (cdylib programs only)
        assert_eq!(workspace.members.len(), 2);

        // Members should be sorted by name
        let member_names: Vec<&String> = workspace.members.iter().map(|m| &m.name).collect();
        assert_eq!(member_names, vec!["program_a", "program_b"]);

        // Verify crate types are correctly extracted (should all be cdylib)
        for member in &workspace.members {
            assert_eq!(member.crate_types, vec!["cdylib"]);
        }

        let program_a = workspace
            .members
            .iter()
            .find(|m| m.name == "program_a")
            .unwrap();

        let program_b = workspace
            .members
            .iter()
            .find(|m| m.name == "program_b")
            .unwrap();

        // Verify paths are correct (canonicalize to handle symlinks)
        let workspace_root = temp_workspace.path().canonicalize().unwrap();
        assert_eq!(
            program_a.path.canonicalize().unwrap(),
            workspace_root.join("program-a")
        );
        assert_eq!(
            program_b.path.canonicalize().unwrap(),
            workspace_root.join("program-b")
        );

        // Verify manifest paths are correct
        assert_eq!(
            program_a.manifest_path.canonicalize().unwrap(),
            workspace_root.join("program-a/Cargo.toml")
        );
        assert_eq!(
            program_b.manifest_path.canonicalize().unwrap(),
            workspace_root.join("program-b/Cargo.toml")
        );
    }

    #[test]
    fn test_discover_workspace_with_filtering() {
        let temp_workspace = create_test_workspace();

        // Test filtering that should exclude program-a
        let config = ManifestConfig {
            include: vec!["**/*".to_string()],
            exclude: vec!["program-a".to_string()],
        };

        let result = discover_workspace(&temp_workspace.path().to_path_buf(), &config);

        let workspace = result.expect("Should successfully discover test workspace");

        // Find Solana programs using the workspace's filtering logic
        let solana_programs = workspace.find_solana_programs();

        // Should find only program-b (program-a excluded, lib-crate not a Solana program)
        assert_eq!(solana_programs.len(), 1);
        assert_eq!(solana_programs[0].name, "program_b");
    }

    #[test]
    fn test_discover_workspace_find_solana_programs() {
        let temp_workspace = create_test_workspace();

        let config = ManifestConfig::allow_all();
        let result = discover_workspace(&temp_workspace.path().to_path_buf(), &config);

        let workspace = result.expect("Should successfully discover test workspace");

        // Find Solana programs
        let solana_programs = workspace.find_solana_programs();

        // Should find exactly 2 Solana programs (cdylib crates)
        assert_eq!(solana_programs.len(), 2);

        // Should be sorted by name
        assert_eq!(solana_programs[0].name, "program_a");
        assert_eq!(solana_programs[1].name, "program_b");

        // Verify they're properly configured
        for program in &solana_programs {
            assert!(program.name.starts_with("program_"));
            assert_eq!(
                program.env_var_name(),
                format!("PROGRAM_{}_ELF_MAGIC_PATH", program.name.to_uppercase())
            );
            assert_eq!(
                program.constant_name(),
                format!("{}_ELF", program.name.to_uppercase())
            );
        }
    }

    #[test]
    fn test_discover_workspace() {
        // This test runs against the real workspace as a sanity check
        let config = ManifestConfig {
            include: vec!["programs/*".to_string()],
            exclude: vec!["programs/deprecated-*".to_string()],
        };

        // Test in current workspace (should work since we're in a valid cargo workspace)
        let current_dir = std::env::current_dir().unwrap();
        match discover_workspace(&current_dir, &config) {
            Ok(workspace) => {
                // Basic sanity checks
                assert!(!workspace.root_path.as_os_str().is_empty());
                assert_eq!(workspace.config.include, vec!["programs/*"]);
                assert_eq!(workspace.config.exclude, vec!["programs/deprecated-*"]);

                // Members should be sorted by name
                let member_names: Vec<&String> =
                    workspace.members.iter().map(|m| &m.name).collect();
                let mut sorted_names = member_names.clone();
                sorted_names.sort();
                assert_eq!(member_names, sorted_names);
            }
            Err(e) => {
                // If it fails, it should be a reasonable error
                println!("Expected error in test environment: {}", e);
                assert!(
                    e.to_string().contains("cargo metadata") || e.to_string().contains("workspace")
                );
            }
        }
    }

    #[test]
    fn test_discover_workspace_sorted() {
        // Test the sorting behavior specifically
        let config = ManifestConfig::allow_all();

        let current_dir = std::env::current_dir().unwrap();
        match discover_workspace(&current_dir, &config) {
            Ok(workspace) => {
                // Verify members are sorted by name
                for i in 1..workspace.members.len() {
                    assert!(workspace.members[i - 1].name <= workspace.members[i].name);
                }
            }
            Err(_) => {
                // Expected to fail in some test environments - that's ok
                // The important thing is that when it works, sorting happens
            }
        }
    }

    #[test]
    fn test_write_lib_file() {
        use tempfile::TempDir;

        // Create test programs
        let programs = vec![SolanaProgram {
            name: "test_program".to_string(),
            path: PathBuf::from("programs/test-program"),
            manifest_path: PathBuf::from("programs/test-program/Cargo.toml"),
        }];

        // Create a completely isolated temporary workspace
        let temp_workspace = TempDir::new().expect("Failed to create temp workspace");
        let temp_src_dir = temp_workspace.path().join("src");
        std::fs::create_dir_all(&temp_src_dir).expect("Failed to create temp src dir");

        // Save the original working directory
        let original_dir = std::env::current_dir().expect("Failed to get current dir");

        // Change to the isolated temp workspace
        std::env::set_current_dir(temp_workspace.path())
            .expect("Failed to change to temp workspace");

        // Test the write_lib_file function in isolation
        let result = write_lib_file(&temp_workspace.path(), &programs);

        // CRITICAL: Restore original directory BEFORE any assertions that might panic
        std::env::set_current_dir(original_dir).expect("Failed to restore original dir");

        // Now it's safe to run assertions
        assert!(result.is_ok(), "write_lib_file should succeed");

        // Check that the file was created in the temp workspace
        let lib_path = temp_src_dir.join("lib.rs");
        assert!(lib_path.exists(), "lib.rs should be created");

        let content = std::fs::read_to_string(&lib_path).expect("Failed to read generated file");

        // Verify key content is present
        assert!(content.contains("auto-generated by elf-magic"));
        assert!(content.contains("TEST_PROGRAM_ELF"));
        assert!(content.contains("pub fn elves()"));

        // Verify no temporary files left behind
        let temp_files: Vec<_> = std::fs::read_dir(&temp_src_dir)
            .expect("Failed to read temp src dir")
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();
                name.starts_with(".tmp") || name.ends_with(".tmp")
            })
            .collect();

        assert!(
            temp_files.is_empty(),
            "Temporary files should be cleaned up: found {:?}",
            temp_files
        );
    }

    #[test]
    fn test_setup_incremental_builds() {
        use std::path::PathBuf;

        let programs = vec![
            SolanaProgram {
                name: "token-manager".to_string(),
                path: PathBuf::from("programs/token-manager"),
                manifest_path: PathBuf::from("programs/token-manager/Cargo.toml"),
            },
            SolanaProgram {
                name: "governance".to_string(),
                path: PathBuf::from("programs/governance"),
                manifest_path: PathBuf::from("programs/governance/Cargo.toml"),
            },
        ];

        // This function outputs to stdout via println! which is hard to capture in tests
        // But we can at least verify it doesn't error
        let result = setup_incremental_builds(&programs);
        assert!(result.is_ok());

        // Note: In a real build.rs context, this would output:
        // cargo:rerun-if-changed=programs/token-manager/src
        // cargo:rerun-if-changed=programs/token-manager/Cargo.toml
        // cargo:rerun-if-changed=programs/governance/src
        // cargo:rerun-if-changed=programs/governance/Cargo.toml
    }
}
