use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Configuration for elf-magic from package.metadata.elf-magic
///
/// Clean three-mode system: Magic (default single workspace) vs Permissive (multi-workspace with excludes) vs Laser Eyes (multi-workspace with includes)
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum Config {
    #[serde(rename = "magic")]
    Magic, // No fields! Just "run cargo metadata here"

    #[serde(rename = "laser-eyes")]
    LaserEyes {
        workspaces: Vec<LaserEyesWorkspaceConfig>,
        #[serde(default)]
        constants: HashMap<String, String>,
        #[serde(default)]
        targets: HashMap<String, String>,
    },

    #[serde(rename = "permissive")]
    Permissive {
        workspaces: Vec<PermissiveWorkspaceConfig>,
        #[serde(default)]
        global_deny: Vec<String>,
        #[serde(default)]
        constants: HashMap<String, String>,
        #[serde(default)]
        targets: HashMap<String, String>,
    },
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

    /// Get the mode name as a string
    pub fn mode_name(&self) -> &'static str {
        match self {
            Config::Magic => "magic",
            Config::LaserEyes { .. } => "laser-eyes",
            Config::Permissive { .. } => "permissive",
        }
    }

    /// Get the constants map for this config
    pub fn constants(&self) -> HashMap<String, String> {
        match self {
            Config::Magic => HashMap::new(),
            Config::LaserEyes { constants, .. } => constants.clone(),
            Config::Permissive { constants, .. } => constants.clone(),
        }
    }

    /// Get the targets map for this config
    pub fn targets(&self) -> HashMap<String, String> {
        match self {
            Config::Magic => HashMap::new(),
            Config::LaserEyes { targets, .. } => targets.clone(),
            Config::Permissive { targets, .. } => targets.clone(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::Magic
    }
}

/// Configuration for a single workspace in laser-eyes mode
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LaserEyesWorkspaceConfig {
    pub manifest_path: String,
    pub only: Vec<String>,
}

/// Configuration for a single workspace in permissive mode
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PermissiveWorkspaceConfig {
    pub manifest_path: String,
    #[serde(default)]
    #[serde(alias = "exclude")]
    pub deny: Vec<String>,
}

/// Resolve constant override paths to absolute paths based on config file location
pub fn resolve_constants_paths(
    constants: &HashMap<String, String>,
    config_file_dir: &Path,
) -> HashMap<PathBuf, String> {
    let mut resolved = HashMap::new();

    for (relative_path, constant_name) in constants {
        let absolute_path = config_file_dir.join(relative_path);
        resolved.insert(absolute_path, constant_name.clone());
    }

    resolved
}

/// Resolve target override paths to absolute paths based on config file location
pub fn resolve_targets_paths(
    targets: &HashMap<String, String>,
    config_file_dir: &Path,
) -> HashMap<PathBuf, String> {
    let mut resolved = HashMap::new();

    for (relative_path, target_name) in targets {
        let absolute_path = config_file_dir.join(relative_path);
        resolved.insert(absolute_path, target_name.clone());
    }

    resolved
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_manifest(content: &str) -> (TempDir, PathBuf) {
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
    fn test_load_config_permissive_mode() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "permissive"
workspaces = [
    { manifest_path = "./Cargo.toml" },
    { manifest_path = "examples/basic/Cargo.toml", deny = ["target:test*"] }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::Permissive {
                workspaces,
                global_deny,
                constants,
                targets,
            } => {
                assert_eq!(workspaces.len(), 2);
                assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
                assert_eq!(workspaces[0].deny.len(), 0);
                assert_eq!(workspaces[1].manifest_path, "examples/basic/Cargo.toml");
                assert_eq!(workspaces[1].deny, vec!["target:test*"]);
                assert_eq!(global_deny.len(), 0); // No global excludes in this test
                assert!(constants.is_empty());
                assert!(targets.is_empty());
            }
            _ => panic!("Expected Permissive mode"),
        }
    }

    #[test]
    fn test_load_config_permissive_mode_with_exclude_alias() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "permissive"
workspaces = [
    { manifest_path = "./Cargo.toml", exclude = ["target:test*"] }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

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
                assert_eq!(global_deny.len(), 0);
                assert!(constants.is_empty());
                assert!(targets.is_empty());
            }
            _ => panic!("Expected Permissive mode"),
        }
    }

    #[test]
    fn test_load_config_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_dir = temp_dir.path().join("non_existent");

        let result = Config::load(&non_existent_dir);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read Cargo.toml"));
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let manifest_content = r#"
[package
name = "test-package" -- invalid TOML syntax
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
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
        let error_message = result.unwrap_err().to_string();
        assert!(error_message.contains("Invalid elf-magic config"));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        match config {
            Config::Magic => {}
            _ => panic!("Expected Magic mode as default"),
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
mode = "permissive"
workspaces = [
    { manifest_path = "./Cargo.toml" }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::Permissive {
                workspaces,
                global_deny,
                constants,
                targets,
            } => {
                assert_eq!(workspaces[0].deny.len(), 0); // Should default to empty
                assert_eq!(global_deny.len(), 0); // Should default to empty
                assert!(constants.is_empty());
                assert!(targets.is_empty());
            }
            _ => panic!("Expected Permissive mode"),
        }
    }

    #[test]
    fn test_load_config_with_global_exclude() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "permissive"
global_deny = ["package:apl-token", "package:apl-associated-token-account"]
workspaces = [
    { manifest_path = "./Cargo.toml" },
    { manifest_path = "examples/escrow/Cargo.toml", deny = ["target:test*"] }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::Permissive {
                workspaces,
                global_deny,
                constants,
                targets,
            } => {
                assert_eq!(workspaces.len(), 2);
                assert_eq!(
                    global_deny,
                    vec!["package:apl-token", "package:apl-associated-token-account"]
                );

                // First workspace has no local excludes
                assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
                assert_eq!(workspaces[0].deny.len(), 0);

                // Second workspace has local excludes
                assert_eq!(workspaces[1].manifest_path, "examples/escrow/Cargo.toml");
                assert_eq!(workspaces[1].deny, vec!["target:test*"]);

                assert!(constants.is_empty());
                assert!(targets.is_empty());
            }
            _ => panic!("Expected Permissive mode"),
        }
    }

    #[test]
    fn test_load_config_laser_eyes_mode() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = ["target:token_manager", "target:governance"] },
    { manifest_path = "examples/defi/Cargo.toml", only = ["target:swap*", "package:my-*-program"] }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::LaserEyes {
                workspaces,
                constants,
                targets,
            } => {
                assert_eq!(workspaces.len(), 2);

                // First workspace
                assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
                assert_eq!(
                    workspaces[0].only,
                    vec!["target:token_manager", "target:governance"]
                );

                // Second workspace
                assert_eq!(workspaces[1].manifest_path, "examples/defi/Cargo.toml");
                assert_eq!(
                    workspaces[1].only,
                    vec!["target:swap*", "package:my-*-program"]
                );

                assert!(constants.is_empty());
                assert!(targets.is_empty());
            }
            _ => panic!("Expected LaserEyes mode"),
        }
    }

    #[test]
    fn test_load_config_laser_eyes_mode_single_workspace() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = ["target:my_program"] }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::LaserEyes {
                workspaces,
                constants,
                targets,
            } => {
                assert_eq!(workspaces.len(), 1);
                assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
                assert_eq!(workspaces[0].only, vec!["target:my_program"]);
                assert!(constants.is_empty());
                assert!(targets.is_empty());
            }
            _ => panic!("Expected LaserEyes mode"),
        }
    }

    #[test]
    fn test_load_config_laser_eyes_mode_empty_include() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./Cargo.toml", only = [] }
]
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::LaserEyes {
                workspaces,
                constants,
                targets,
            } => {
                assert_eq!(workspaces.len(), 1);
                assert_eq!(workspaces[0].manifest_path, "./Cargo.toml");
                assert_eq!(workspaces[0].only.len(), 0);
                assert!(constants.is_empty());
                assert!(targets.is_empty());
            }
            _ => panic!("Expected LaserEyes mode"),
        }
    }

    #[test]
    fn test_load_config_laser_eyes_mode_with_constants_and_targets() {
        let manifest_content = r#"
[package]
name = "test-package"
version = "0.1.0"
edition = "2021"

[package.metadata.elf-magic]
mode = "laser-eyes"
workspaces = [
    { manifest_path = "./upstream/Cargo.toml", only = ["path:*/program/*", "path:*/p-token/*"] }
]
constants = { "upstream/programs/token/program" = "SPL_TOKEN_PROGRAM_ELF", "upstream/programs/token/p-token" = "SPL_TOKEN_P_TOKEN_ELF" }
targets = { "upstream/programs/token/p-token" = "potato" }
"#;

        let (_temp_dir, manifest_dir) = create_temp_manifest(manifest_content);
        let config = Config::load(&manifest_dir).unwrap();

        match config {
            Config::LaserEyes {
                workspaces,
                constants,
                targets,
            } => {
                assert_eq!(workspaces.len(), 1);
                assert_eq!(workspaces[0].manifest_path, "./upstream/Cargo.toml");
                assert_eq!(
                    workspaces[0].only,
                    vec!["path:*/program/*", "path:*/p-token/*"]
                );

                // Check constants
                assert_eq!(constants.len(), 2);
                assert_eq!(
                    constants.get("upstream/programs/token/program"),
                    Some(&"SPL_TOKEN_PROGRAM_ELF".to_string())
                );
                assert_eq!(
                    constants.get("upstream/programs/token/p-token"),
                    Some(&"SPL_TOKEN_P_TOKEN_ELF".to_string())
                );

                // Check targets
                assert_eq!(targets.len(), 1);
                assert_eq!(
                    targets.get("upstream/programs/token/p-token"),
                    Some(&"potato".to_string())
                );
            }
            _ => panic!("Expected LaserEyes mode"),
        }
    }
}
