use cargo_metadata::{CrateType, Metadata, MetadataCommand};

use crate::{
    config::Config,
    error::Error,
    programs::{DiscoveredPrograms, SolanaProgram},
};

/// Load workspaces from config
pub fn load_workspaces(config: &Config) -> Result<Vec<Workspace>, Error> {
    match config {
        Config::LaserEyes { workspaces } => {
            let mut results = Vec::new();
            for workspace in workspaces {
                let metadata = MetadataCommand::new()
                    .manifest_path(&workspace.manifest_path)
                    .exec()?;

                let manifest_path = workspace.manifest_path.clone();

                results.push(Workspace {
                    metadata,
                    manifest_path,
                    filter_mode: FilterMode::Only(workspace.only.clone()),
                });
            }
            Ok(results)
        }
        Config::Magic => {
            let metadata = MetadataCommand::new().exec()?;

            let manifest_path = metadata
                .workspace_root
                .as_std_path()
                .join("Cargo.toml")
                .display()
                .to_string();

            Ok(vec![Workspace {
                metadata,
                manifest_path,
                filter_mode: FilterMode::Magic,
            }])
        }
        Config::Permissive {
            workspaces,
            global_deny,
        } => {
            let mut results = Vec::new();
            for workspace in workspaces {
                let metadata = MetadataCommand::new()
                    .manifest_path(&workspace.manifest_path)
                    .exec()?;

                let manifest_path = workspace.manifest_path.clone();

                // Merge global excludes with workspace-specific excludes
                let mut merged_denies = global_deny.clone();
                merged_denies.extend(workspace.deny.clone());

                results.push(Workspace {
                    metadata,
                    manifest_path,
                    filter_mode: FilterMode::Deny(merged_denies),
                });
            }
            Ok(results)
        }
    }
}

/// Filtering mode for programs
#[derive(Debug, Clone)]
pub enum FilterMode {
    /// Magic mode: include all programs (no filtering)
    Magic,
    /// Permissive mode: include all except those matching deny patterns
    Deny(Vec<String>),
    /// Laser-eyes mode: include programs matching only patterns
    Only(Vec<String>),
}

/// Information about an individual cargo workspace
pub struct Workspace {
    pub metadata: Metadata,
    pub manifest_path: String,
    pub filter_mode: FilterMode,
}

impl Workspace {
    /// Discover Solana programs in the workspace
    pub fn discover_programs(&self) -> Result<DiscoveredPrograms, Error> {
        let mut included = Vec::new();
        let mut excluded = Vec::new();

        for package in &self.metadata.packages {
            for target in &package.targets {
                let is_cdylib = target.crate_types.contains(&CrateType::CDyLib);
                if !is_cdylib {
                    continue;
                }

                let program = SolanaProgram {
                    package_name: package.name.to_string(),
                    target_name: target.name.to_string(),
                    manifest_path: package.manifest_path.as_std_path().to_path_buf(),
                };

                match &self.filter_mode {
                    FilterMode::Magic => {
                        // Magic mode: include all programs
                        included.push(program);
                    }
                    FilterMode::Deny(deny_patterns) => {
                        // Permissive mode: include all except those matching deny patterns
                        if should_include_program_permissive(&program, deny_patterns) {
                            included.push(program);
                        } else {
                            excluded.push(program);
                        }
                    }
                    FilterMode::Only(only_patterns) => {
                        // Laser-eyes mode: include programs matching only patterns
                        if should_only_include_program(&program, only_patterns) {
                            included.push(program);
                        } else {
                            excluded.push(program);
                        }
                    }
                }
            }
        }

        included.sort_by(|a, b| a.target_name.cmp(&b.target_name));
        excluded.sort_by(|a, b| a.target_name.cmp(&b.target_name));

        Ok(DiscoveredPrograms {
            workspace_path: self.manifest_path.clone(),
            included,
            excluded,
        })
    }
}

/// Check if a program should be included in laser-eyes mode (matches only patterns)
fn should_only_include_program(program: &SolanaProgram, only_patterns: &[String]) -> bool {
    only_patterns
        .iter()
        .any(|pattern| matches_program_pattern(program, pattern))
}

/// Check if a program should be included (not denied by glob patterns)
fn should_include_program_permissive(program: &SolanaProgram, deny_patterns: &[String]) -> bool {
    !deny_patterns
        .iter()
        .any(|pattern| matches_program_pattern(program, pattern))
}

fn matches_program_pattern(program: &SolanaProgram, pattern: &str) -> bool {
    if let Some(target_pattern) = pattern.strip_prefix("target:") {
        matches_glob(&program.target_name, target_pattern)
    } else if let Some(package_pattern) = pattern.strip_prefix("package:") {
        matches_glob(&program.package_name, package_pattern)
    } else if let Some(path_pattern) = pattern.strip_prefix("path:") {
        matches_glob(&program.manifest_path.to_string_lossy(), path_pattern)
    } else {
        // No fallback - invalid pattern
        eprintln!(
            "Warning: Invalid deny pattern '{}'. Use 'target:', 'package:', or 'path:' prefix.",
            pattern
        );
        false
    }
}

fn matches_glob(text: &str, pattern: &str) -> bool {
    glob::Pattern::new(pattern)
        .map(|p| p.matches(text))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_program(target_name: &str, package_name: &str) -> SolanaProgram {
        SolanaProgram {
            target_name: target_name.to_string(),
            package_name: package_name.to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
        }
    }

    #[test]
    fn test_should_include_program_no_exclusions() {
        let program = sample_program("my_target", "my_package");
        let deny_patterns = vec![];

        assert!(should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_should_include_program_target_exclusion_match() {
        let program = sample_program("test_program", "my_package");
        let deny_patterns = vec!["target:test*".to_string()];

        assert!(!should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_should_include_program_target_exclusion_no_match() {
        let program = sample_program("main_program", "my_package");
        let deny_patterns = vec!["target:test*".to_string()];

        assert!(should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_should_include_program_package_exclusion_match() {
        let program = sample_program("my_target", "dev_package");
        let deny_patterns = vec!["package:dev*".to_string()];

        assert!(!should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_should_include_program_package_exclusion_no_match() {
        let program = sample_program("my_target", "main_package");
        let deny_patterns = vec!["package:dev*".to_string()];

        assert!(should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_should_include_program_path_exclusion_match() {
        let program = SolanaProgram {
            target_name: "my_target".to_string(),
            package_name: "my_package".to_string(),
            manifest_path: PathBuf::from("/workspace/examples/test/Cargo.toml"),
        };
        let deny_patterns = vec!["path:*/examples/*".to_string()];

        assert!(!should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_should_include_program_path_exclusion_no_match() {
        let program = SolanaProgram {
            target_name: "my_target".to_string(),
            package_name: "my_package".to_string(),
            manifest_path: PathBuf::from("/workspace/src/Cargo.toml"),
        };
        let deny_patterns = vec!["path:*/examples/*".to_string()];

        assert!(should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_should_include_program_multiple_exclusions() {
        let program = sample_program("test_target", "dev_package");
        let deny_patterns = vec!["target:test*".to_string(), "package:dev*".to_string()];

        // Should be denied because it matches the target pattern
        assert!(!should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_should_include_program_multiple_exclusions_no_match() {
        let program = sample_program("main_target", "main_package");
        let deny_patterns = vec!["target:test*".to_string(), "package:dev*".to_string()];

        assert!(should_include_program_permissive(&program, &deny_patterns));
    }

    #[test]
    fn test_matches_program_pattern_target() {
        let program = sample_program("test_program", "my_package");

        assert!(matches_program_pattern(&program, "target:test*"));
        assert!(matches_program_pattern(&program, "target:test_program"));
        assert!(!matches_program_pattern(&program, "target:main*"));
    }

    #[test]
    fn test_matches_program_pattern_package() {
        let program = sample_program("my_target", "dev_package");

        assert!(matches_program_pattern(&program, "package:dev*"));
        assert!(matches_program_pattern(&program, "package:dev_package"));
        assert!(!matches_program_pattern(&program, "package:main*"));
    }

    #[test]
    fn test_matches_program_pattern_path() {
        let program = SolanaProgram {
            target_name: "my_target".to_string(),
            package_name: "my_package".to_string(),
            manifest_path: PathBuf::from("/workspace/examples/basic/Cargo.toml"),
        };

        assert!(matches_program_pattern(&program, "path:*/examples/*"));
        assert!(matches_program_pattern(&program, "path:*/basic/*"));
        assert!(!matches_program_pattern(&program, "path:*/src/*"));
    }

    #[test]
    fn test_matches_program_pattern_invalid_prefix() {
        let program = sample_program("test_program", "my_package");

        // Invalid patterns should return false and print warning
        assert!(!matches_program_pattern(&program, "invalid:test*"));
        assert!(!matches_program_pattern(&program, "test*")); // No prefix
        assert!(!matches_program_pattern(&program, "random_pattern"));
    }

    #[test]
    fn test_matches_glob_basic() {
        assert!(matches_glob("test_program", "test*"));
        assert!(matches_glob("test_program", "*program"));
        assert!(matches_glob("test_program", "test_program"));
        assert!(!matches_glob("test_program", "main*"));
    }

    #[test]
    fn test_matches_glob_question_mark() {
        assert!(matches_glob("test", "tes?"));
        assert!(matches_glob("test", "t?st"));
        assert!(!matches_glob("test", "tes??"));
    }

    #[test]
    fn test_matches_glob_complex_patterns() {
        assert!(matches_glob("my-test-program", "*test*"));
        assert!(matches_glob("program_v1", "program_*"));
        assert!(matches_glob("examples/basic/program", "examples/*/program"));
        assert!(!matches_glob(
            "examples/basic/program",
            "examples/advanced/*"
        ));
    }

    #[test]
    fn test_matches_glob_invalid_pattern() {
        // Invalid glob pattern should not match anything
        assert!(!matches_glob("test", "[invalid"));
    }

    #[test]
    fn test_global_exclude_merging() {
        let program1 = sample_program("my_target", "apl-token");
        let program2 = sample_program("test_target", "my_package");
        let program3 = sample_program("my_target", "dev-package");

        // Test merging: global excludes + workspace excludes
        let global_denies = vec!["package:apl-*".to_string()];
        let workspace_denies = vec!["target:test*".to_string()];
        let mut merged_denies = global_denies.clone();
        merged_denies.extend(workspace_denies);

        // program1 should be excluded by global exclude (package:apl-*)
        assert!(!should_include_program_permissive(
            &program1,
            &merged_denies
        ));

        // program2 should be excluded by workspace exclude (target:test*)
        assert!(!should_include_program_permissive(
            &program2,
            &merged_denies
        ));

        // program3 should be included (matches neither pattern)
        assert!(should_include_program_permissive(&program3, &merged_denies));
    }

    #[test]
    fn test_should_only_include_program_target_match() {
        let program = sample_program("token_manager", "my_package");
        let only_patterns = vec!["target:token*".to_string()];

        assert!(should_only_include_program(&program, &only_patterns));
    }

    #[test]
    fn test_should_only_include_program_target_no_match() {
        let program = sample_program("governance", "my_package");
        let only_patterns = vec!["target:token*".to_string()];

        assert!(!should_only_include_program(&program, &only_patterns));
    }

    #[test]
    fn test_should_only_include_program_package_match() {
        let program = sample_program("my_target", "token_program");
        let only_patterns = vec!["package:token*".to_string()];

        assert!(should_only_include_program(&program, &only_patterns));
    }

    #[test]
    fn test_should_only_include_program_package_no_match() {
        let program = sample_program("my_target", "governance_program");
        let only_patterns = vec!["package:token*".to_string()];

        assert!(!should_only_include_program(&program, &only_patterns));
    }

    #[test]
    fn test_should_only_include_program_multiple_patterns() {
        let program1 = sample_program("token_manager", "my_package");
        let program2 = sample_program("governance", "my_package");
        let program3 = sample_program("other_program", "my_package");

        let only_patterns = vec!["target:token*".to_string(), "target:governance".to_string()];

        // Should match both token* and governance patterns
        assert!(should_only_include_program(&program1, &only_patterns));
        assert!(should_only_include_program(&program2, &only_patterns));

        // Should not match
        assert!(!should_only_include_program(&program3, &only_patterns));
    }

    #[test]
    fn test_should_only_include_program_empty_patterns() {
        let program = sample_program("any_program", "any_package");
        let only_patterns = vec![];

        // Empty patterns should include nothing
        assert!(!should_only_include_program(&program, &only_patterns));
    }

    #[test]
    fn test_should_only_include_program_path_match() {
        let program = SolanaProgram {
            target_name: "my_target".to_string(),
            package_name: "my_package".to_string(),
            manifest_path: PathBuf::from("/workspace/programs/core/Cargo.toml"),
        };
        let only_patterns = vec!["path:*/programs/core/*".to_string()];

        assert!(should_only_include_program(&program, &only_patterns));
    }

    #[test]
    fn test_filter_mode_magic() {
        let filter_mode = FilterMode::Magic;

        match filter_mode {
            FilterMode::Magic => {
                // Magic mode should include all programs - test passes by not panicking
            }
            _ => panic!("Expected Magic filter mode"),
        }
    }

    #[test]
    fn test_filter_mode_exclude() {
        let filter_mode = FilterMode::Deny(vec!["target:test*".to_string()]);

        match filter_mode {
            FilterMode::Deny(patterns) => {
                assert_eq!(patterns, vec!["target:test*"]);
            }
            _ => panic!("Expected Exclude filter mode"),
        }
    }

    #[test]
    fn test_filter_mode_include() {
        let filter_mode = FilterMode::Only(vec!["target:token*".to_string()]);

        match filter_mode {
            FilterMode::Only(patterns) => {
                assert_eq!(patterns, vec!["target:token*"]);
            }
            _ => panic!("Expected Include filter mode"),
        }
    }
}
