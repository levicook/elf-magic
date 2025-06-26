use std::fs;
use tempfile::TempDir;

pub fn create_test_workspace_with_config(
    config_content: &str,
    cargo_toml_content: &str,
) -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    // Create the elf-magic consumer Cargo.toml
    let consumer_manifest = temp_dir.path().join("Cargo.toml");
    fs::write(&consumer_manifest, config_content).unwrap();

    // Create a mock workspace Cargo.toml for testing
    let workspace_dir = temp_dir.path().join("workspace");
    fs::create_dir_all(&workspace_dir).unwrap();
    let workspace_manifest = workspace_dir.join("Cargo.toml");
    fs::write(&workspace_manifest, cargo_toml_content).unwrap();

    temp_dir
}
