use crate::error::Error;
use std::{fmt, path::PathBuf};

/// A confirmed Solana program (has crate-type = ["cdylib"])
#[derive(Clone)]
pub struct SolanaProgram {
    pub manifest_path: PathBuf,
    pub package_name: String,
    pub target_name: String,
    pub constant_name: String,
}

/// Result of building multiple Solana programs
#[derive(Debug)]
pub struct BuildResult {
    pub successful: Vec<(SolanaProgram, PathBuf)>,
    pub failed: Vec<(SolanaProgram, Error)>,
}

impl BuildResult {
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
        }
    }

    pub fn add_success(&mut self, program: SolanaProgram, path: PathBuf) {
        self.successful.push((program, path));
    }

    pub fn add_failure(&mut self, program: SolanaProgram, error: Error) {
        self.failed.push((program, error));
    }
}

impl SolanaProgram {
    /// Convert target name to environment variable name
    pub fn env_var_name(&self) -> String {
        format!("{}_ELF_PATH", self.target_name.to_uppercase())
    }
}

impl fmt::Debug for SolanaProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SolanaProgram")
            .field("target_name", &self.target_name)
            .field("manifest_path", &self.manifest_path.display())
            .field("env_var_name", &self.env_var_name())
            .field("constant_name", &self.constant_name)
            .finish()
    }
}

impl fmt::Display for SolanaProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.target_name, self.manifest_path.display())
    }
}

/// Details about what happened in a single workspace
#[derive(Debug, Clone)]
pub struct DiscoveredPrograms {
    pub workspace_path: String,
    pub included: Vec<SolanaProgram>,
    pub excluded: Vec<SolanaProgram>,
}

/// Result of the entire generation process with rich reporting
#[derive(Debug)]
pub struct GenerationResult {
    pub discovery_mode: String, // "magic", "permissive", or "laser-eyes"
    pub discovered_programs: Vec<DiscoveredPrograms>,
}

impl fmt::Display for GenerationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_workspaces = self.discovered_programs.len();
        writeln!(
            f,
            "Mode: {} ({} workspace{} specified)",
            self.discovery_mode,
            total_workspaces,
            if total_workspaces == 1 { "" } else { "s" }
        )?;
        writeln!(f)?;

        for workspace in &self.discovered_programs {
            writeln!(f, "Workspace: {}", workspace.workspace_path)?;

            for program in &workspace.included {
                writeln!(f, "  + {}", program.target_name)?;
            }

            for excluded in &workspace.excluded {
                writeln!(f, "  - {} (denied by pattern)", excluded.target_name)?;
            }

            if workspace.included.is_empty() && workspace.excluded.is_empty() {
                writeln!(f, "  (no Solana programs found)")?;
            }

            writeln!(f)?;
        }

        let total_programs: usize = self
            .discovered_programs
            .iter()
            .map(|w| w.included.len())
            .sum();

        if total_programs > 0 {
            writeln!(
                f,
                "Generated lib.rs with {} Solana program{}",
                total_programs,
                if total_programs == 1 { "" } else { "s" }
            )?;
        } else {
            writeln!(f, "⚠️  No Solana programs found - generated empty lib.rs")?;
        }

        Ok(())
    }
}

impl GenerationResult {
    pub fn new(mode: String, workspace_results: Vec<DiscoveredPrograms>) -> Self {
        Self {
            discovery_mode: mode,
            discovered_programs: workspace_results,
        }
    }

    /// Get all included programs across all workspaces
    pub fn programs(&self) -> Vec<&SolanaProgram> {
        self.discovered_programs
            .iter()
            .flat_map(|w| &w.included)
            .collect()
    }
}

/// Deduplicate programs by manifest_path to handle cases where multiple workspaces
/// discover the same program (e.g., shared dependencies)
pub fn deduplicate_programs(programs: Vec<SolanaProgram>) -> Vec<SolanaProgram> {
    use std::collections::HashMap;

    let mut seen: HashMap<PathBuf, SolanaProgram> = HashMap::new();

    for program in programs {
        // Use manifest_path as the key, keeping the first occurrence
        seen.entry(program.manifest_path.clone()).or_insert(program);
    }

    let mut deduplicated: Vec<SolanaProgram> = seen.into_values().collect();

    // Sort by target_name to ensure consistent alphabetical ordering
    deduplicated.sort_by(|a, b| a.target_name.cmp(&b.target_name));

    deduplicated
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_program() -> SolanaProgram {
        SolanaProgram {
            package_name: "my-package".to_string(),
            target_name: "my_target".to_string(),
            manifest_path: PathBuf::from("/path/to/Cargo.toml"),
            constant_name: "MY_TARGET_ELF".to_string(),
        }
    }

    #[test]
    fn test_env_var_name() {
        let program = sample_program();
        assert_eq!(program.env_var_name(), "MY_TARGET_ELF_PATH");
    }

    #[test]
    fn test_env_var_name_with_hyphens() {
        let program = SolanaProgram {
            package_name: "my-package".to_string(),
            target_name: "my_target_program".to_string(),
            manifest_path: PathBuf::from("/path/to/Cargo.toml"),
            constant_name: "MY_TARGET_PROGRAM_ELF".to_string(),
        };
        assert_eq!(program.env_var_name(), "MY_TARGET_PROGRAM_ELF_PATH");
    }

    #[test]
    fn test_deduplicate_programs_removes_duplicates_by_manifest_path() {
        // Same program appearing twice (realistic scenario from multiple workspaces)
        let apl_token_duplicate1 = SolanaProgram {
            package_name: "apl-token".to_string(),
            target_name: "apl_token".to_string(),
            manifest_path: PathBuf::from("/repo/token/Cargo.toml"),
            constant_name: "APL_TOKEN_ELF".to_string(),
        };

        let apl_token_duplicate2 = SolanaProgram {
            package_name: "apl-token".to_string(),
            target_name: "apl_token".to_string(),
            manifest_path: PathBuf::from("/repo/token/Cargo.toml"), // Same path!
            constant_name: "APL_TOKEN_ELF".to_string(),
        };

        let escrow_program = SolanaProgram {
            package_name: "escrow_program".to_string(),
            target_name: "escrow_program".to_string(),
            manifest_path: PathBuf::from("/repo/examples/escrow/program/Cargo.toml"),
            constant_name: "ESCROW_PROGRAM_ELF".to_string(),
        };

        // Input: 3 programs (with 1 duplicate)
        let input_programs = vec![
            escrow_program.clone(),
            apl_token_duplicate1,
            apl_token_duplicate2,
        ];
        assert_eq!(input_programs.len(), 3);

        // After deduplication: should have 2 unique programs
        let deduplicated = deduplicate_programs(input_programs);
        assert_eq!(
            deduplicated.len(),
            2,
            "Should deduplicate to 2 unique programs"
        );

        // Verify we have one of each program type
        let apl_token_count = deduplicated
            .iter()
            .filter(|p| p.target_name == "apl_token")
            .count();
        assert_eq!(
            apl_token_count, 1,
            "Should have exactly 1 apl_token after deduplication"
        );

        let escrow_count = deduplicated
            .iter()
            .filter(|p| p.target_name == "escrow_program")
            .count();
        assert_eq!(escrow_count, 1, "Should have exactly 1 escrow_program");
    }

    #[test]
    fn test_deduplicate_programs_preserves_unique_programs() {
        let program1 = SolanaProgram {
            package_name: "package1".to_string(),
            target_name: "target1".to_string(),
            manifest_path: PathBuf::from("/workspace/program1/Cargo.toml"),
            constant_name: "TARGET1_ELF".to_string(),
        };

        let program2 = SolanaProgram {
            package_name: "package2".to_string(),
            target_name: "target2".to_string(),
            manifest_path: PathBuf::from("/workspace/program2/Cargo.toml"),
            constant_name: "TARGET2_ELF".to_string(),
        };

        // Input: 2 unique programs
        let input_programs = vec![program1, program2];
        let deduplicated = deduplicate_programs(input_programs);

        // Should preserve both programs
        assert_eq!(deduplicated.len(), 2, "Should preserve all unique programs");
    }

    #[test]
    fn test_generation_result_display_magic_mode() {
        let program1 = SolanaProgram {
            package_name: "pkg1".to_string(),
            target_name: "target1".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
            constant_name: "MY_TARGET_ELF".to_string(),
        };

        let program2 = SolanaProgram {
            package_name: "pkg2".to_string(),
            target_name: "target2".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
            constant_name: "MY_TARGET_ELF".to_string(),
        };

        let discovered = DiscoveredPrograms {
            workspace_path: "./Cargo.toml".to_string(),
            included: vec![program1, program2],
            excluded: vec![],
        };

        let result = GenerationResult::new("magic".to_string(), vec![discovered]);
        let display = format!("{}", result);

        assert!(display.contains("Mode: magic (1 workspace specified)"));
        assert!(display.contains("Workspace: ./Cargo.toml"));
        assert!(display.contains("  + target1"));
        assert!(display.contains("  + target2"));
        assert!(display.contains("Generated lib.rs with 2 Solana programs"));
    }

    #[test]
    fn test_generation_result_display_with_exclusions() {
        let included_program = SolanaProgram {
            package_name: "included".to_string(),
            target_name: "good_target".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
            constant_name: "MY_TARGET_ELF".to_string(),
        };

        let excluded_program = SolanaProgram {
            package_name: "excluded".to_string(),
            target_name: "bad_target".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
            constant_name: "MY_TARGET_ELF".to_string(),
        };

        let discovered = DiscoveredPrograms {
            workspace_path: "./Cargo.toml".to_string(),
            included: vec![included_program],
            excluded: vec![excluded_program],
        };

        let result = GenerationResult::new("permissive".to_string(), vec![discovered]);
        let display = format!("{}", result);

        assert!(display.contains("Mode: permissive"));
        assert!(display.contains("  + good_target"));
        assert!(display.contains("  - bad_target (denied by pattern)"));
        assert!(display.contains("Generated lib.rs with 1 Solana program"));
    }

    #[test]
    fn test_generation_result_display_no_programs() {
        let discovered = DiscoveredPrograms {
            workspace_path: "./empty/Cargo.toml".to_string(),
            included: vec![],
            excluded: vec![],
        };

        let result = GenerationResult::new("magic".to_string(), vec![discovered]);
        let display = format!("{}", result);

        assert!(display.contains("(no Solana programs found)"));
        assert!(display.contains("⚠️  No Solana programs found - generated empty lib.rs"));
    }

    #[test]
    fn test_generation_result_programs_method() {
        let program1 = SolanaProgram {
            package_name: "pkg1".to_string(),
            target_name: "target1".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
            constant_name: "MY_TARGET_ELF".to_string(),
        };

        let program2 = SolanaProgram {
            package_name: "pkg2".to_string(),
            target_name: "target2".to_string(),
            manifest_path: PathBuf::from("/workspace2/Cargo.toml"),
            constant_name: "MY_TARGET_ELF".to_string(),
        };

        let discovered1 = DiscoveredPrograms {
            workspace_path: "./Cargo.toml".to_string(),
            included: vec![program1],
            excluded: vec![],
        };

        let discovered2 = DiscoveredPrograms {
            workspace_path: "./workspace2/Cargo.toml".to_string(),
            included: vec![program2],
            excluded: vec![],
        };

        let result =
            GenerationResult::new("permissive".to_string(), vec![discovered1, discovered2]);
        let programs = result.programs();

        assert_eq!(programs.len(), 2);
        assert_eq!(programs[0].target_name, "target1");
        assert_eq!(programs[1].target_name, "target2");
    }
}
