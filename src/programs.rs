use std::fmt;
use std::path::PathBuf;

/// A confirmed Solana program (has crate-type = ["cdylib"])
#[derive(Clone)]
pub struct SolanaProgram {
    pub manifest_path: PathBuf,
    pub package_name: String,
    pub target_name: String,
}

impl SolanaProgram {
    /// Convert target name to environment variable name
    pub fn env_var_name(&self) -> String {
        format!("{}_ELF_PATH", self.target_name.to_uppercase())
    }

    /// Convert target name to constant name for generated code
    pub fn constant_name(&self) -> String {
        format!("{}_ELF", self.target_name.to_uppercase())
    }
}

impl fmt::Debug for SolanaProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SolanaProgram")
            .field("target_name", &self.target_name)
            .field("manifest_path", &self.manifest_path.display())
            .field("env_var_name", &self.env_var_name())
            .field("constant_name", &self.constant_name())
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
    pub discovery_mode: String, // "magic" or "pedantic"
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
                writeln!(f, "  - {} (excluded by pattern)", excluded.target_name)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_program() -> SolanaProgram {
        SolanaProgram {
            package_name: "my-package".to_string(),
            target_name: "my_target".to_string(),
            manifest_path: PathBuf::from("/path/to/Cargo.toml"),
        }
    }

    #[test]
    fn test_env_var_name() {
        let program = sample_program();
        assert_eq!(program.env_var_name(), "MY_TARGET_ELF_PATH");
    }

    #[test]
    fn test_constant_name() {
        let program = sample_program();
        assert_eq!(program.constant_name(), "MY_TARGET_ELF");
    }

    #[test]
    fn test_env_var_name_with_hyphens() {
        let program = SolanaProgram {
            package_name: "my-package".to_string(),
            target_name: "my_target_program".to_string(),
            manifest_path: PathBuf::from("/path/to/Cargo.toml"),
        };
        assert_eq!(program.env_var_name(), "MY_TARGET_PROGRAM_ELF_PATH");
    }

    #[test]
    fn test_constant_name_with_hyphens() {
        let program = SolanaProgram {
            package_name: "my-package".to_string(),
            target_name: "my_target_program".to_string(),
            manifest_path: PathBuf::from("/path/to/Cargo.toml"),
        };
        assert_eq!(program.constant_name(), "MY_TARGET_PROGRAM_ELF");
    }

    #[test]
    fn test_generation_result_display_magic_mode() {
        let program1 = SolanaProgram {
            package_name: "pkg1".to_string(),
            target_name: "target1".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
        };

        let program2 = SolanaProgram {
            package_name: "pkg2".to_string(),
            target_name: "target2".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
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
        };

        let excluded_program = SolanaProgram {
            package_name: "excluded".to_string(),
            target_name: "bad_target".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
        };

        let discovered = DiscoveredPrograms {
            workspace_path: "./Cargo.toml".to_string(),
            included: vec![included_program],
            excluded: vec![excluded_program],
        };

        let result = GenerationResult::new("pedantic".to_string(), vec![discovered]);
        let display = format!("{}", result);

        assert!(display.contains("Mode: pedantic"));
        assert!(display.contains("  + good_target"));
        assert!(display.contains("  - bad_target (excluded by pattern)"));
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
        };

        let program2 = SolanaProgram {
            package_name: "pkg2".to_string(),
            target_name: "target2".to_string(),
            manifest_path: PathBuf::from("/workspace2/Cargo.toml"),
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

        let result = GenerationResult::new("pedantic".to_string(), vec![discovered1, discovered2]);
        let programs = result.programs();

        assert_eq!(programs.len(), 2);
        assert_eq!(programs[0].target_name, "target1");
        assert_eq!(programs[1].target_name, "target2");
    }
}
