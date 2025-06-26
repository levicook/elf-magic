mod common;

use common::create_test_workspace_with_config;
use elf_magic::config::Config;

#[test]
fn test_magic_mode_default_config() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[build-dependencies]
elf-magic = { path = "../.." }
"#;

    let workspace_cargo_toml = r#"
[workspace]
members = ["programs/*"]

[workspace.package]
version = "0.1.0"
edition = "2021"
"#;

    let temp_dir = create_test_workspace_with_config(config_content, workspace_cargo_toml);

    // Test that config defaults to Magic mode when no metadata is specified
    let config = Config::load(temp_dir.path()).unwrap();

    match config {
        Config::Magic => {
            // Success - this is the expected default behavior
        }
        _ => panic!("Expected Magic mode as default"),
    }
}

#[test]
fn test_magic_mode_explicit_config() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "magic"

[build-dependencies]
elf-magic = { path = "../.." }
"#;

    let workspace_cargo_toml = r#"
[workspace]
members = ["programs/*"]

[workspace.package]
version = "0.1.0"
edition = "2021"
"#;

    let temp_dir = create_test_workspace_with_config(config_content, workspace_cargo_toml);

    let config = Config::load(temp_dir.path()).unwrap();

    match config {
        Config::Magic => {
            // Success - explicit magic mode works
        }
        _ => panic!("Expected Magic mode"),
    }
}

#[test]
fn test_invalid_mode_config() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "invalid-mode"

[build-dependencies]
elf-magic = { path = "../.." }
"#;

    let workspace_cargo_toml = r#"
[workspace]
members = ["programs/*"]
"#;

    let temp_dir = create_test_workspace_with_config(config_content, workspace_cargo_toml);

    let result = Config::load(temp_dir.path());
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid elf-magic config"));
}
