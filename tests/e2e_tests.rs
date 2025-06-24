#![cfg(feature = "testing")]

use elf_magic;
use std::env;
use std::fs;
use tempfile::TempDir;
use toml;

/// Creates a complete mock Solana workspace with real buildable programs
fn create_e2e_workspace() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp workspace");
    let workspace_root = temp_dir.path();

    // Create root Cargo.toml with workspace
    let root_cargo_toml = r#"
[workspace]
members = ["token-program", "governance-program", "shared-lib"]
resolver = "2"

[workspace.dependencies]
solana-program = "~1.18.0"
"#;
    fs::write(workspace_root.join("Cargo.toml"), root_cargo_toml)
        .expect("Failed to write root Cargo.toml");

    // Create token-program (Solana program with cdylib)
    let token_dir = workspace_root.join("token-program");
    fs::create_dir_all(token_dir.join("src")).expect("Failed to create token-program dir");

    let token_cargo_toml = r#"
[package]
name = "token-program"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
solana-program = { workspace = true }
"#;
    fs::write(token_dir.join("Cargo.toml"), token_cargo_toml)
        .expect("Failed to write token-program Cargo.toml");

    let token_lib_rs = r#"
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Token program executed!");
    Ok(())
}
"#;
    fs::write(token_dir.join("src/lib.rs"), token_lib_rs)
        .expect("Failed to write token-program lib.rs");

    // Create governance-program (Solana program with cdylib)
    let governance_dir = workspace_root.join("governance-program");
    fs::create_dir_all(governance_dir.join("src"))
        .expect("Failed to create governance-program dir");

    let governance_cargo_toml = r#"
[package]
name = "governance-program"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
solana-program = { workspace = true }
"#;
    fs::write(governance_dir.join("Cargo.toml"), governance_cargo_toml)
        .expect("Failed to write governance-program Cargo.toml");

    let governance_lib_rs = r#"
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Governance program executed!");
    Ok(())
}
"#;
    fs::write(governance_dir.join("src/lib.rs"), governance_lib_rs)
        .expect("Failed to write governance-program lib.rs");

    // Create shared-lib (regular library, should be ignored)
    let shared_dir = workspace_root.join("shared-lib");
    fs::create_dir_all(shared_dir.join("src")).expect("Failed to create shared-lib dir");

    let shared_cargo_toml = r#"
[package]
name = "shared-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["lib"]

[dependencies]
solana-program = { workspace = true }
"#;
    fs::write(shared_dir.join("Cargo.toml"), shared_cargo_toml)
        .expect("Failed to write shared-lib Cargo.toml");

    let shared_lib_rs = r#"
pub fn shared_function() -> u32 {
    42
}
"#;
    fs::write(shared_dir.join("src/lib.rs"), shared_lib_rs)
        .expect("Failed to write shared-lib lib.rs");

    temp_dir
}

/// Creates an elf-magic crate within the workspace
fn create_elf_crate_in_workspace(workspace_root: &std::path::Path, elf_magic_config: Option<&str>) {
    // Get the path to the current elf-magic workspace for the dependency
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let elf_magic_path = current_dir.display();
    let elf_dir = workspace_root.join("my-elves");
    fs::create_dir_all(elf_dir.join("src")).expect("Failed to create elf crate dir");

    // Add my-elves to workspace members
    let mut root_cargo = fs::read_to_string(workspace_root.join("Cargo.toml"))
        .expect("Failed to read root Cargo.toml");

    // Parse the current TOML to extract existing members
    let mut cargo_toml: toml::Value =
        toml::from_str(&root_cargo).expect("Failed to parse root Cargo.toml");

    if let Some(workspace) = cargo_toml.get_mut("workspace") {
        if let Some(members) = workspace.get_mut("members") {
            if let Some(members_array) = members.as_array_mut() {
                // Add "my-elves" if not already present
                let my_elves_value = toml::Value::String("my-elves".to_string());
                if !members_array.contains(&my_elves_value) {
                    members_array.push(my_elves_value);
                }
            }
        }
    }

    // Write the updated TOML back
    root_cargo = toml::to_string(&cargo_toml).expect("Failed to serialize updated Cargo.toml");

    fs::write(workspace_root.join("Cargo.toml"), root_cargo)
        .expect("Failed to update root Cargo.toml");

    let mut elf_cargo_toml = format!(
        r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[build-dependencies]
elf-magic = {{ path = "{}" }}
"#,
        elf_magic_path
    );

    // Add elf-magic configuration if provided
    if let Some(config) = elf_magic_config {
        elf_cargo_toml.push_str("\n");
        elf_cargo_toml.push_str(config);
    }

    fs::write(elf_dir.join("Cargo.toml"), elf_cargo_toml)
        .expect("Failed to write elf crate Cargo.toml");

    let build_rs = r#"
fn main() {
    elf_magic::generate().expect("elf-magic generation failed");
}
"#;
    fs::write(elf_dir.join("build.rs"), build_rs).expect("Failed to write build.rs");

    // Create a minimal lib.rs that will be overwritten
    let lib_rs = r#"
// This will be overwritten by elf-magic
"#;
    fs::write(elf_dir.join("src/lib.rs"), lib_rs).expect("Failed to write initial lib.rs");
}

#[test]
fn test_e2e_full_pipeline() {
    // Create a complete mock Solana workspace
    let workspace = create_e2e_workspace();

    // Create an elf-magic crate with default configuration (find everything)
    create_elf_crate_in_workspace(workspace.path(), None);

    // Run the full elf-magic pipeline using white-box API
    let elf_manifest_dir = workspace
        .path()
        .join("my-elves")
        .to_string_lossy()
        .to_string();
    let result = elf_magic::generate_with_manifest_dir(&elf_manifest_dir);

    // Verify the generation succeeded
    assert!(
        result.is_ok(),
        "elf-magic generation should succeed: {:?}",
        result.err()
    );
    let generation_result = result.unwrap();

    // Debug: Print what programs were found
    println!("Found {} programs:", generation_result.programs.len());
    for program in &generation_result.programs {
        println!("  - {} at {}", program.name, program.path.display());
    }

    // Should find 2 Solana programs (token_program, governance_program)
    assert_eq!(generation_result.programs.len(), 2);
    let program_names: Vec<&String> = generation_result.programs.iter().map(|p| &p.name).collect();
    assert!(program_names.contains(&&"governance_program".to_string()));
    assert!(program_names.contains(&&"token_program".to_string()));

    // Verify the generated lib.rs file exists and has correct content
    let lib_path = workspace.path().join("my-elves/src/lib.rs");
    assert!(lib_path.exists(), "Generated lib.rs should exist");

    let lib_content = fs::read_to_string(&lib_path).expect("Failed to read generated lib.rs");

    // Verify key constants are present
    assert!(
        lib_content.contains("GOVERNANCE_PROGRAM_ELF"),
        "Should contain governance program constant"
    );
    assert!(
        lib_content.contains("TOKEN_PROGRAM_ELF"),
        "Should contain token program constant"
    );
    assert!(
        lib_content.contains("pub fn elves()"),
        "Should contain elves function"
    );
    assert!(
        lib_content.contains("auto-generated by elf-magic"),
        "Should contain generation comment"
    );

    // Verify environment variables are set for the programs
    for program in &generation_result.programs {
        let env_var = program.env_var_name();
        assert!(
            lib_content.contains(&env_var),
            "Should reference env var {}",
            env_var
        );
    }

    // TODO: In a real E2E test, we'd also verify that:
    // - The .so files actually exist in the target directory
    // - The include_bytes! calls work correctly
    // - The cargo:rerun-if-changed directives are properly set
    // This requires actually running cargo build-sbf which needs Solana SDK installed
}

#[test]
fn test_e2e_with_filtering() {
    // Create a complete mock Solana workspace
    let workspace = create_e2e_workspace();

    // Create an elf-magic crate with filtering (exclude governance-program)
    let elf_magic_config = r#"
[package.metadata.elf-magic]
include = ["**/*"]
exclude = ["governance-program"]
"#;
    create_elf_crate_in_workspace(workspace.path(), Some(elf_magic_config));

    // Run the full elf-magic pipeline using white-box API
    let elf_manifest_dir = workspace
        .path()
        .join("my-elves")
        .to_string_lossy()
        .to_string();
    let result = elf_magic::generate_with_manifest_dir(&elf_manifest_dir);

    // Verify the generation succeeded
    assert!(
        result.is_ok(),
        "elf-magic generation should succeed: {:?}",
        result.err()
    );
    let generation_result = result.unwrap();

    // Should find only 1 Solana program (token_program, governance_program excluded)
    assert_eq!(generation_result.programs.len(), 1);
    assert_eq!(generation_result.programs[0].name, "token_program");

    // Verify the generated lib.rs file has correct content
    let lib_path = workspace.path().join("my-elves/src/lib.rs");
    let lib_content = fs::read_to_string(&lib_path).expect("Failed to read generated lib.rs");

    // Should contain token program but not governance program
    assert!(
        lib_content.contains("TOKEN_PROGRAM_ELF"),
        "Should contain token program constant"
    );

    assert!(
        !lib_content.contains("GOVERNANCE_PROGRAM_ELF"),
        "Should NOT contain governance program constant"
    );
}

#[test]
fn test_e2e_empty_workspace() {
    // Create a workspace with no Solana programs
    let temp_dir = tempfile::tempdir().expect("Failed to create temp workspace");
    let workspace_root = temp_dir.path();

    // Create root Cargo.toml with workspace containing only regular libs
    let root_cargo_toml = r#"
[workspace]
members = ["regular-lib"]
resolver = "2"
"#;
    fs::write(workspace_root.join("Cargo.toml"), root_cargo_toml)
        .expect("Failed to write root Cargo.toml");

    // Create regular-lib (not a Solana program)
    let lib_dir = workspace_root.join("regular-lib");
    fs::create_dir_all(lib_dir.join("src")).expect("Failed to create regular-lib dir");

    let lib_cargo_toml = r#"
[package]
name = "regular-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["lib"]
"#;
    fs::write(lib_dir.join("Cargo.toml"), lib_cargo_toml)
        .expect("Failed to write regular-lib Cargo.toml");

    fs::write(lib_dir.join("src/lib.rs"), "pub fn hello() {}")
        .expect("Failed to write regular-lib lib.rs");

    // Create an elf-magic crate
    create_elf_crate_in_workspace(workspace_root, None);

    // Run the full elf-magic pipeline using white-box API
    let elf_manifest_dir = workspace_root
        .join("my-elves")
        .to_string_lossy()
        .to_string();
    let result = elf_magic::generate_with_manifest_dir(&elf_manifest_dir);

    // Verify the generation succeeded
    assert!(
        result.is_ok(),
        "elf-magic generation should succeed even with no programs"
    );
    let generation_result = result.unwrap();

    // Should find 0 Solana programs
    assert_eq!(generation_result.programs.len(), 0);

    // Verify the generated lib.rs file exists but is empty
    let lib_path = workspace_root.join("my-elves/src/lib.rs");
    let lib_content = fs::read_to_string(&lib_path).expect("Failed to read generated lib.rs");

    // Should contain empty elves function and generation comment
    assert!(
        lib_content.contains("pub fn elves()"),
        "Should contain elves function"
    );
    assert!(
        lib_content.contains("vec![]"),
        "Should contain empty vector"
    );
    assert!(
        lib_content.contains("auto-generated by elf-magic"),
        "Should contain generation comment"
    );
    assert!(
        !lib_content.contains("_ELF"),
        "Should not contain any ELF constants"
    );
}

#[test]
fn test_e2e_invalid_workspace() {
    // Test behavior when workspace discovery fails
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Don't create any Cargo.toml files - this should cause workspace discovery to fail

    // Run elf-magic pipeline using white-box API - should fail gracefully
    let temp_manifest_dir = temp_dir.path().to_string_lossy().to_string();
    let result = elf_magic::generate_with_manifest_dir(&temp_manifest_dir);

    // Should fail with a clear error
    assert!(
        result.is_err(),
        "elf-magic should fail gracefully on invalid workspace"
    );
    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(
        error_msg.contains("Cargo.toml")
            || error_msg.contains("workspace")
            || error_msg.contains("metadata"),
        "Error should mention workspace/cargo issues: {}",
        error_msg
    );
}
