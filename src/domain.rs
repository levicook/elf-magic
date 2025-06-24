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
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub include: Vec<String>, // Glob patterns like "programs/*"
    pub exclude: Vec<String>, // Glob patterns like "programs/deprecated-*"
}

impl DiscoveryConfig {
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

impl From<&DiscoveryConfig> for ProgramFilter {
    fn from(config: &DiscoveryConfig) -> Self {
        Self::new(config.include.clone(), config.exclude.clone())
    }
}

/// A Cargo workspace containing potential Solana programs
#[derive(Debug, Clone)]
pub struct Workspace {
    pub root_path: PathBuf,
    pub members: Vec<WorkspaceMember>,
    pub config: DiscoveryConfig,
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
#[derive(Debug, Clone)]
pub struct SolanaProgram {
    pub name: String,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
}

/// A successfully built Solana program with ELF output
#[derive(Debug, Clone)]
pub struct BuiltProgram {
    pub program: SolanaProgram,
    pub elf_path: PathBuf,
    pub env_var_name: String, // e.g., "PROGRAM_TOKEN_MANAGER_ELF_MAGIC_PATH"
}

/// Generated constant definition for the lib.rs file
#[derive(Debug, Clone)]
pub struct ConstantDefinition {
    pub name: String,    // e.g., "TOKEN_MANAGER_ELF"
    pub env_var: String, // e.g., "PROGRAM_TOKEN_MANAGER_ELF_MAGIC_PATH"
}

/// Generated code structure
#[derive(Debug, Clone)]
pub struct GeneratedCode {
    pub constants: Vec<ConstantDefinition>,
    pub all_programs_fn: String,
}

/// Result of the entire generation process
#[derive(Debug)]
pub struct GenerationResult {
    pub built_programs: Vec<BuiltProgram>,
    pub generated_code: GeneratedCode,
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

    /// Build all provided Solana programs using cargo build-sbf
    pub fn build_programs(
        &self,
        programs: &[SolanaProgram],
    ) -> Result<Vec<BuiltProgram>, ElfMagicError> {
        // TODO: For each program, call cargo build-sbf
        // TODO: Determine output .so file path
        // TODO: Generate environment variable name
        // TODO: Return BuiltProgram with all metadata
        todo!("implement build_programs")
    }
}

impl GenerationResult {
    pub fn new(built_programs: Vec<BuiltProgram>, generated_code: GeneratedCode) -> Self {
        Self {
            built_programs,
            generated_code,
        }
    }

    pub fn empty() -> Self {
        Self {
            built_programs: Vec::new(),
            generated_code: GeneratedCode {
                constants: Vec::new(),
                all_programs_fn:
                    "pub fn all_programs() -> Vec<(&'static str, &'static [u8])> {\n    vec![]\n}"
                        .to_string(),
            },
        }
    }
}

impl GeneratedCode {
    pub fn new(constants: Vec<ConstantDefinition>) -> Self {
        let all_programs_fn = if constants.is_empty() {
            "pub fn all_programs() -> Vec<(&'static str, &'static [u8])> {\n    vec![]\n}"
                .to_string()
        } else {
            // TODO: Generate the all_programs function body
            todo!("implement all_programs function generation")
        };

        Self {
            constants,
            all_programs_fn,
        }
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

        let config = DiscoveryConfig {
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
    fn test_discovery_config_conversion() {
        let config = DiscoveryConfig {
            include: vec!["programs/*".to_string()],
            exclude: vec!["deprecated-*".to_string()],
        };

        let filter = ProgramFilter::from(&config);

        assert!(filter.should_include(Path::new("programs/good")));
        assert!(!filter.should_include(Path::new("deprecated-bad")));
    }
}
