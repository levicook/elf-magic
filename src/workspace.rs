use cargo_metadata::{CrateType, Metadata, MetadataCommand};

use crate::{
    config::Config,
    error::Error,
    programs::{DiscoveredPrograms, SolanaProgram},
};

/// Load workspaces from config
pub fn load_workspaces(config: &Config) -> Result<Vec<Workspace>, Error> {
    match config {
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
                exclude_patterns: vec![],
            }])
        }
        Config::Pedantic { workspaces } => {
            let mut results = Vec::new();
            for workspace in workspaces {
                let metadata = MetadataCommand::new()
                    .manifest_path(&workspace.manifest_path)
                    .exec()?;

                let manifest_path = workspace.manifest_path.clone();
                let exclude = workspace.exclude_patterns.clone();

                results.push(Workspace {
                    metadata,
                    manifest_path,
                    exclude_patterns: exclude,
                });
            }
            Ok(results)
        }
    }
}

/// Information about an individual cargo workspace
pub struct Workspace {
    pub metadata: Metadata,
    pub manifest_path: String,
    pub exclude_patterns: Vec<String>,
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

                if should_include_program(&program, &self.exclude_patterns) {
                    included.push(program);
                } else {
                    excluded.push(program);
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

/// Check if a program should be included (not excluded by glob patterns)
fn should_include_program(program: &SolanaProgram, exclude_patterns: &[String]) -> bool {
    !exclude_patterns
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
            "Warning: Invalid exclude pattern '{}'. Use 'target:', 'package:', or 'path:' prefix.",
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
        let exclude_patterns = vec![];

        assert!(should_include_program(&program, &exclude_patterns));
    }

    #[test]
    fn test_should_include_program_target_exclusion_match() {
        let program = sample_program("test_program", "my_package");
        let exclude_patterns = vec!["target:test*".to_string()];

        assert!(!should_include_program(&program, &exclude_patterns));
    }

    #[test]
    fn test_should_include_program_target_exclusion_no_match() {
        let program = sample_program("main_program", "my_package");
        let exclude_patterns = vec!["target:test*".to_string()];

        assert!(should_include_program(&program, &exclude_patterns));
    }

    #[test]
    fn test_should_include_program_package_exclusion_match() {
        let program = sample_program("my_target", "dev_package");
        let exclude_patterns = vec!["package:dev*".to_string()];

        assert!(!should_include_program(&program, &exclude_patterns));
    }

    #[test]
    fn test_should_include_program_package_exclusion_no_match() {
        let program = sample_program("my_target", "main_package");
        let exclude_patterns = vec!["package:dev*".to_string()];

        assert!(should_include_program(&program, &exclude_patterns));
    }

    #[test]
    fn test_should_include_program_path_exclusion_match() {
        let program = SolanaProgram {
            target_name: "my_target".to_string(),
            package_name: "my_package".to_string(),
            manifest_path: PathBuf::from("/workspace/examples/test/Cargo.toml"),
        };
        let exclude_patterns = vec!["path:*/examples/*".to_string()];

        assert!(!should_include_program(&program, &exclude_patterns));
    }

    #[test]
    fn test_should_include_program_path_exclusion_no_match() {
        let program = SolanaProgram {
            target_name: "my_target".to_string(),
            package_name: "my_package".to_string(),
            manifest_path: PathBuf::from("/workspace/src/Cargo.toml"),
        };
        let exclude_patterns = vec!["path:*/examples/*".to_string()];

        assert!(should_include_program(&program, &exclude_patterns));
    }

    #[test]
    fn test_should_include_program_multiple_exclusions() {
        let program = sample_program("test_target", "dev_package");
        let exclude_patterns = vec!["target:test*".to_string(), "package:dev*".to_string()];

        // Should be excluded because it matches the target pattern
        assert!(!should_include_program(&program, &exclude_patterns));
    }

    #[test]
    fn test_should_include_program_multiple_exclusions_no_match() {
        let program = sample_program("main_target", "main_package");
        let exclude_patterns = vec!["target:test*".to_string(), "package:dev*".to_string()];

        assert!(should_include_program(&program, &exclude_patterns));
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
        // Invalid glob patterns should return false
        assert!(!matches_glob("test", "[")); // Unclosed bracket
    }
}
