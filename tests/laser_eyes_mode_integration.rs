mod common;

use common::create_test_workspace_with_config;
use elf_magic::config::Config;

#[test]
fn test_laser_eyes_mode_config_loading() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
        { manifest_path = "./workspace/Cargo.toml", only = ["target:token_manager", "target:governance"] },
        { manifest_path = "./examples/Cargo.toml", only = ["target:swap*"] }
]

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

    // Test that config loads correctly
    let config = Config::load(temp_dir.path()).unwrap();

    match config {
        Config::LaserEyes { workspaces } => {
            assert_eq!(workspaces.len(), 2);

            // First workspace
            assert_eq!(workspaces[0].manifest_path, "./workspace/Cargo.toml");
            assert_eq!(
                workspaces[0].only,
                vec!["target:token_manager", "target:governance"]
            );

            // Second workspace
            assert_eq!(workspaces[1].manifest_path, "./examples/Cargo.toml");
            assert_eq!(workspaces[1].only, vec!["target:swap*"]);
        }
        _ => panic!("Expected LaserEyes config"),
    }
}

#[test]
fn test_laser_eyes_mode_single_workspace_simple() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = ["target:my_program"] }
]

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
        Config::LaserEyes { workspaces } => {
            assert_eq!(workspaces.len(), 1);
            assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
            assert_eq!(workspaces[0].only, vec!["target:my_program"]);
        }
        _ => panic!("Expected LaserEyes config"),
    }
}

#[test]
fn test_laser_eyes_mode_pattern_matching() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = ["target:*_core", "package:my-*-program", "path:*/core/programs/*"] }
]

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
        Config::LaserEyes { workspaces } => {
            assert_eq!(workspaces.len(), 1);
            assert_eq!(workspaces[0].only.len(), 3);
            assert!(workspaces[0].only.contains(&"target:*_core".to_string()));
            assert!(workspaces[0]
                .only
                .contains(&"package:my-*-program".to_string()));
            assert!(workspaces[0]
                .only
                .contains(&"path:*/core/programs/*".to_string()));
        }
        _ => panic!("Expected LaserEyes config"),
    }
}

#[test]
fn test_laser_eyes_mode_empty_include() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = [] }
]

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
        Config::LaserEyes { workspaces } => {
            assert_eq!(workspaces.len(), 1);
            assert_eq!(workspaces[0].only.len(), 0);
        }
        _ => panic!("Expected LaserEyes config"),
    }
}

#[test]
fn test_missing_include_in_laser_eyes_mode() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml" }
]

[build-dependencies]
elf-magic = { path = "../.." }
"#;

    let workspace_cargo_toml = r#"
[workspace]
members = ["programs/*"]
"#;

    let temp_dir = create_test_workspace_with_config(config_content, workspace_cargo_toml);

    let result = Config::load(temp_dir.path());
    // This should fail because only field is required in LaserEyesWorkspaceConfig
    assert!(result.is_err());
}
