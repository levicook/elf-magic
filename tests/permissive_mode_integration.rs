mod common;

use common::create_test_workspace_with_config;
use elf_magic::config::Config;

#[test]
fn test_permissive_mode_basic_config() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "permissive"
workspaces = [
    { manifest_path = "./Cargo.toml" }
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
        Config::Permissive {
            workspaces,
            global_deny,
            constants,
            targets,
        } => {
            assert_eq!(workspaces.len(), 1);
            assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
            assert_eq!(workspaces[0].deny.len(), 0);
            assert_eq!(global_deny.len(), 0);
            assert!(constants.is_empty());
            assert!(targets.is_empty());
        }
        _ => panic!("Expected Permissive config"),
    }
}

#[test]
fn test_permissive_mode_with_exclusions() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "permissive"
workspaces = [
    { manifest_path = "./Cargo.toml", deny = ["target:test*", "package:dev*"] }
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
        Config::Permissive {
            workspaces,
            global_deny,
            constants,
            targets,
        } => {
            assert_eq!(workspaces.len(), 1);
            assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
            assert_eq!(workspaces[0].deny, vec!["target:test*", "package:dev*"]);
            assert_eq!(global_deny.len(), 0);
            assert!(constants.is_empty());
            assert!(targets.is_empty());
        }
        _ => panic!("Expected Permissive config"),
    }
}

#[test]
fn test_permissive_mode_with_global_exclude() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "permissive"
global_deny = ["package:apl-token"]
workspaces = [
    { manifest_path = "./Cargo.toml", deny = ["target:test*"] }
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
        Config::Permissive {
            workspaces,
            global_deny,
            constants,
            targets,
        } => {
            assert_eq!(workspaces.len(), 1);
            assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
            assert_eq!(workspaces[0].deny, vec!["target:test*"]);
            assert_eq!(global_deny, vec!["package:apl-token"]);
            assert!(constants.is_empty());
            assert!(targets.is_empty());
        }
        _ => panic!("Expected Permissive config"),
    }
}

#[test]
fn test_permissive_mode_multi_workspace_complex() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "permissive"
global_deny = ["package:*-test", "target:bench*"]
workspaces = [
    { manifest_path = "./Cargo.toml", deny = ["target:dev*"] },
    { manifest_path = "examples/Cargo.toml" },
    { manifest_path = "tests/Cargo.toml", deny = ["path:*/integration/*"] }
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
        Config::Permissive {
            workspaces,
            global_deny,
            constants,
            targets,
        } => {
            assert_eq!(workspaces.len(), 3);

            // Global deny patterns
            assert_eq!(global_deny, vec!["package:*-test", "target:bench*"]);

            // First workspace with local deny patterns
            assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
            assert_eq!(workspaces[0].deny, vec!["target:dev*"]);

            // Second workspace with no local deny patterns
            assert_eq!(workspaces[1].manifest_path, "examples/Cargo.toml");
            assert_eq!(workspaces[1].deny.len(), 0);

            // Third workspace with different local deny patterns
            assert_eq!(workspaces[2].manifest_path, "tests/Cargo.toml");
            assert_eq!(workspaces[2].deny, vec!["path:*/integration/*"]);

            assert!(constants.is_empty());
            assert!(targets.is_empty());
        }
        _ => panic!("Expected Permissive config"),
    }
}

#[test]
fn test_missing_workspaces_in_permissive_mode() {
    let config_content = r#"
[package]
name = "my-elves"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "permissive"

[build-dependencies]
elf-magic = { path = "../.." }
"#;

    let workspace_cargo_toml = r#"
[workspace]
members = ["programs/*"]
"#;

    let temp_dir = create_test_workspace_with_config(config_content, workspace_cargo_toml);

    let result = Config::load(temp_dir.path());
    // This should fail because workspaces field is required in Permissive mode
    assert!(result.is_err());
}
