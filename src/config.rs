use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Configuration for elf-magic from package.metadata.elf-magic
///
/// Clean two-mode system: Magic (default single workspace) vs Pedantic (multi-workspace)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum Config {
    #[serde(rename = "magic")]
    Magic, // No fields! Just "run cargo metadata here"

    #[serde(rename = "pedantic")]
    Pedantic { workspaces: Vec<WorkspaceConfig> },
}

impl Config {
    pub fn load(manifest_dir: &Path) -> Result<Self, Error> {
        let manifest_path = manifest_dir.join("Cargo.toml");
        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            let message = format!("Failed to read Cargo.toml: {}", e);
            Error::Config(message)
        })?;

        let toml_value: toml::Value = toml::from_str(&content).map_err(|e| {
            let message = format!("Invalid TOML in Cargo.toml: {}", e);
            Error::Config(message)
        })?;

        // Extract package.metadata.elf-magic, default to Magic mode if not present
        let metadata = toml_value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("elf-magic"));

        match metadata {
            Some(config_value) => {
                let json_value = serde_json::to_value(config_value).map_err(|e| {
                    let message = format!("Failed to convert config: {}", e);
                    Error::Config(message)
                })?;
                serde_json::from_value(json_value).map_err(|e| {
                    let message = format!("Invalid elf-magic config: {}", e);
                    Error::Config(message)
                })
            }
            None => Ok(Config::Magic),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::Magic
    }
}

/// Configuration for a single workspace in pedantic mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub manifest_path: String,
    #[serde(default)]
    #[serde(alias = "exclude")]
    pub exclude_patterns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_manifest(content: &str) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("Cargo.toml");
        fs::write(&manifest_path, content).unwrap();
        let path = temp_dir.path().to_path_buf();
        (temp_dir, path)
    }

    #[test]
    fn test_load_config_magic_mode_default() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::Magic => {}
            _ => panic!("Expected Magic mode"),
        }
    }

    #[test]
    fn test_load_config_magic_mode_explicit() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "magic"
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::Magic => {}
            _ => panic!("Expected Magic mode"),
        }
    }

    #[test]
    fn test_load_config_pedantic_mode() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "pedantic"
workspaces = [
    { manifest_path = "./Cargo.toml" },
    { manifest_path = "examples/basic/Cargo.toml", exclude_patterns = ["target:test*"] }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::Pedantic { workspaces } => {
                assert_eq!(workspaces.len(), 2);
                assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
                assert_eq!(workspaces[0].exclude_patterns.len(), 0);
                assert_eq!(workspaces[1].manifest_path, "examples/basic/Cargo.toml");
                assert_eq!(workspaces[1].exclude_patterns, vec!["target:test*"]);
            }
            _ => panic!("Expected Pedantic mode"),
        }
    }

    #[test]
    fn test_load_config_pedantic_mode_with_exclude_alias() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "pedantic"
workspaces = [
    { manifest_path = "./Cargo.toml", exclude = ["target:test*", "package:dev*"] }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::Pedantic { workspaces } => {
                assert_eq!(workspaces.len(), 1);
                assert_eq!(
                    workspaces[0].exclude_patterns,
                    vec!["target:test*", "package:dev*"]
                );
            }
            _ => panic!("Expected Pedantic mode"),
        }
    }

    #[test]
    fn test_load_config_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("missing");

        let result = Config::load(&non_existent);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read Cargo.toml"));
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let invalid_toml = r#"
[package
name = "invalid"
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(invalid_toml);
        let result = Config::load(&manifest_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid TOML"));
    }

    #[test]
    fn test_load_config_invalid_elf_magic_config() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "invalid-mode"
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let result = Config::load(&manifest_dir);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid elf-magic config"));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        match config {
            Config::Magic => {}
            _ => panic!("Default should be Magic mode"),
        }
    }

    #[test]
    fn test_workspace_config_defaults() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "pedantic"
workspaces = [
    { manifest_path = "./Cargo.toml" }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::Pedantic { workspaces } => {
                assert_eq!(workspaces[0].exclude_patterns.len(), 0); // Should default to empty
            }
            _ => panic!("Expected Pedantic mode"),
        }
    }
}
