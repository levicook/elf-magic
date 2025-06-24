use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

/// Main error type for elf-magic operations
#[derive(Debug, thiserror::Error)]
pub enum ElfMagicError {
    #[error("Failed to discover workspace: {0}")]
    WorkspaceDiscovery(String),

    #[error("Failed to build program {program}: {error}")]
    ProgramBuild { program: String, error: String },

    #[error("Failed to generate code: {0}")]
    CodeGeneration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Metadata error: {0}")]
    Metadata(String),
}

/// Configuration for program discovery from package.metadata.elf-magic
///
/// This config specifies include/exclude glob patterns to determine
/// which workspace members should be built as Solana programs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestConfig {
    #[serde(default)]
    pub include: Vec<String>, // Glob patterns like "programs/*"
    #[serde(default)]
    pub exclude: Vec<String>, // Glob patterns like "programs/deprecated-*"
}

impl ManifestConfig {
    /// Create a config that includes everything (for testing/fallback only)
    pub fn allow_all() -> Self {
        Self {
            include: vec!["**/*".to_string()],
            exclude: vec![],
        }
    }

    /// Create a config that includes nothing (safe default)
    pub fn allow_none() -> Self {
        Self {
            include: vec![],
            exclude: vec!["**/*".to_string()],
        }
    }
}

/// Program filter using glob patterns - core domain logic for which programs to build
#[derive(Debug, Clone)]
pub struct ProgramFilter {
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
}

impl ProgramFilter {
    pub fn new(include_patterns: Vec<String>, exclude_patterns: Vec<String>) -> Self {
        Self {
            include_patterns,
            exclude_patterns,
        }
    }

    /// Create filter that includes everything
    pub fn allow_all() -> Self {
        Self {
            include_patterns: vec!["**/*".to_string()],
            exclude_patterns: vec![],
        }
    }

    /// Test if a program path should be included based on glob patterns
    pub fn should_include(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // If no include patterns, include everything by default
        let included = if self.include_patterns.is_empty() {
            true
        } else {
            // TODO: Implement glob pattern matching
            // For now, just check if path contains any include pattern (simplified)
            self.include_patterns.iter().any(|pattern| {
                // Simplified matching - replace with proper glob later
                let pattern_without_glob = pattern.replace("*", "");
                path_str.contains(&pattern_without_glob)
            })
        };

        // Check exclude patterns
        let excluded = self.exclude_patterns.iter().any(|pattern| {
            // Simplified matching - replace with proper glob later
            let pattern_without_glob = pattern.replace("*", "");
            path_str.contains(&pattern_without_glob)
        });

        included && !excluded
    }
}

impl Default for ProgramFilter {
    fn default() -> Self {
        Self::allow_all()
    }
}

impl From<&ManifestConfig> for ProgramFilter {
    fn from(config: &ManifestConfig) -> Self {
        Self::new(config.include.clone(), config.exclude.clone())
    }
}

/// A Cargo workspace containing potential Solana programs
#[derive(Debug, Clone)]
pub struct Workspace {
    pub root_path: PathBuf,
    pub members: Vec<WorkspaceMember>,
    pub config: ManifestConfig,
}

/// A workspace member (crate) that might be a Solana program
#[derive(Debug, Clone)]
pub struct WorkspaceMember {
    pub name: String,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
    pub crate_types: Vec<String>,
}

/// A confirmed Solana program (has crate-type = ["cdylib"])
#[derive(Clone)]
pub struct SolanaProgram {
    pub name: String,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
}

impl fmt::Debug for SolanaProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SolanaProgram")
            .field("name", &self.name)
            .field("path", &self.path.display())
            .field("env_var_name", &self.env_var_name())
            .field("constant_name", &self.constant_name())
            .finish()
    }
}

impl fmt::Display for SolanaProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.path.display())
    }
}

/// Result of the entire generation process
pub struct GenerationResult {
    pub programs: Vec<SolanaProgram>,
}

impl fmt::Debug for GenerationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GenerationResult")
            .field("program_count", &self.programs.len())
            .field("programs", &self.programs)
            .finish()
    }
}

impl fmt::Display for GenerationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.programs.is_empty() {
            write!(f, "Generated lib.rs (no Solana programs found)")
        } else {
            writeln!(
                f,
                "Generated lib.rs with {} Solana programs:",
                self.programs.len()
            )?;
            for program in &self.programs {
                writeln!(f, "  - {}", program)?;
            }
            Ok(())
        }
    }
}

impl GenerationResult {
    pub fn new(programs: Vec<SolanaProgram>) -> Self {
        Self { programs }
    }
}

impl Workspace {
    /// Find all Solana programs in this workspace using configured filters
    pub fn find_solana_programs(&self) -> Vec<SolanaProgram> {
        let filter = ProgramFilter::from(&self.config);
        self.find_solana_programs_with_filter(&filter)
    }

    /// Find Solana programs with a specific filter
    pub fn find_solana_programs_with_filter(&self, filter: &ProgramFilter) -> Vec<SolanaProgram> {
        self.members
            .iter()
            .filter(|member| {
                // Must be a Solana program (cdylib)
                member.crate_types.contains(&"cdylib".to_string()) &&
                // Must pass the path filter
                filter.should_include(&member.path)
            })
            .map(|member| SolanaProgram {
                name: member.name.clone(),
                path: member.path.clone(),
                manifest_path: member.manifest_path.clone(),
            })
            .collect()
    }
}

impl SolanaProgram {
    /// Generate environment variable name for this program
    /// e.g., "token_manager" → "PROGRAM_TOKEN_MANAGER_ELF_MAGIC_PATH"
    pub fn env_var_name(&self) -> String {
        format!("PROGRAM_{}_ELF_MAGIC_PATH", self.name.to_uppercase())
    }

    /// Generate constant name for this program
    /// e.g., "token_manager" → "TOKEN_MANAGER_ELF"
    pub fn constant_name(&self) -> String {
        format!("{}_ELF", self.name.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_member(name: &str, path: &str, crate_types: Vec<&str>) -> WorkspaceMember {
        WorkspaceMember {
            name: name.to_string(),
            path: PathBuf::from(path),
            manifest_path: PathBuf::from(format!("{}/Cargo.toml", path)),
            crate_types: crate_types.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_program_filter_simple() {
        let filter = ProgramFilter::new(
            vec!["programs/*".to_string()],
            vec!["programs/deprecated-*".to_string()],
        );

        // Should include programs that match pattern
        assert!(filter.should_include(Path::new("programs/token-manager")));
        assert!(filter.should_include(Path::new("programs/governance")));

        // Should exclude deprecated programs
        assert!(!filter.should_include(Path::new("programs/deprecated-old")));

        // Should exclude non-program paths
        assert!(!filter.should_include(Path::new("examples/demo")));
        assert!(!filter.should_include(Path::new("src/lib.rs")));
    }

    #[test]
    fn test_program_filter_empty_includes_all() {
        let filter = ProgramFilter::new(vec![], vec!["deprecated-*".to_string()]);

        // Empty include patterns should include everything
        assert!(filter.should_include(Path::new("programs/token-manager")));
        assert!(filter.should_include(Path::new("examples/demo")));

        // But still respect exclude patterns
        assert!(!filter.should_include(Path::new("deprecated-old")));
    }

    #[test]
    fn test_workspace_find_solana_programs() {
        let members = vec![
            create_test_member("token-manager", "programs/token-manager", vec!["cdylib"]),
            create_test_member("governance", "programs/governance", vec!["cdylib"]),
            create_test_member("deprecated", "programs/deprecated-old", vec!["cdylib"]),
            create_test_member("test-utils", "test-utils", vec!["lib"]),
        ];

        let config = ManifestConfig {
            include: vec!["programs/*".to_string()],
            exclude: vec!["programs/deprecated-*".to_string()],
        };

        let workspace = Workspace {
            root_path: PathBuf::from("/workspace"),
            members,
            config,
        };

        let programs = workspace.find_solana_programs();

        // Should find 2 programs (excluding deprecated and non-cdylib)
        assert_eq!(programs.len(), 2);
        assert!(programs.iter().any(|p| p.name == "token-manager"));
        assert!(programs.iter().any(|p| p.name == "governance"));

        // Should not include deprecated or non-cdylib crates
        assert!(!programs.iter().any(|p| p.name == "deprecated"));
        assert!(!programs.iter().any(|p| p.name == "test-utils"));
    }

    #[test]
    fn test_manifest_config_conversion() {
        let config = ManifestConfig {
            include: vec!["programs/*".to_string()],
            exclude: vec!["deprecated-*".to_string()],
        };

        let filter = ProgramFilter::from(&config);

        assert!(filter.should_include(Path::new("programs/good")));
        assert!(!filter.should_include(Path::new("deprecated-bad")));
    }

    #[test]
    fn test_solana_program_naming() {
        let program = SolanaProgram {
            name: "token_manager".to_string(),
            path: PathBuf::from("programs/token-manager"),
            manifest_path: PathBuf::from("programs/token-manager/Cargo.toml"),
        };

        assert_eq!(
            program.env_var_name(),
            "PROGRAM_TOKEN_MANAGER_ELF_MAGIC_PATH"
        );
        assert_eq!(program.constant_name(), "TOKEN_MANAGER_ELF");
    }

    #[test]
    fn test_solana_program_debug_display() {
        let program = SolanaProgram {
            name: "token-manager".to_string(),
            path: PathBuf::from("programs/token-manager"),
            manifest_path: PathBuf::from("programs/token-manager/Cargo.toml"),
        };

        // Test Debug - should be clean and structured with computed fields
        let debug_output = format!("{:?}", program);
        assert!(debug_output.contains("SolanaProgram"));
        assert!(debug_output.contains("token-manager"));
        assert!(debug_output.contains("programs/token-manager"));
        assert!(debug_output.contains("env_var_name"));
        assert!(debug_output.contains("PROGRAM_TOKEN_MANAGER_ELF_MAGIC_PATH"));
        assert!(debug_output.contains("constant_name"));
        assert!(debug_output.contains("TOKEN_MANAGER_ELF"));

        // Test Display - should be user-friendly
        let display_output = format!("{}", program);
        assert_eq!(display_output, "token-manager (programs/token-manager)");
    }

    #[test]
    fn test_generation_result_debug_display() {
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

        let result = GenerationResult::new(programs);

        // Test Debug - should show count and details
        let debug_output = format!("{:?}", result);
        assert!(debug_output.contains("GenerationResult"));
        assert!(debug_output.contains("program_count: 2"));

        // Test Display - should be user-friendly summary
        let display_output = format!("{}", result);
        assert!(display_output.contains("Generated lib.rs with 2 Solana programs:"));
        assert!(display_output.contains("- token-manager (programs/token-manager)"));
        assert!(display_output.contains("- governance (programs/governance)"));

        // Test empty case
        let empty_result = GenerationResult::new(vec![]);
        let empty_display = format!("{}", empty_result);
        assert_eq!(empty_display, "Generated lib.rs (no Solana programs found)");
    }
}
