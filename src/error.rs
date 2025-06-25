/// Main error type for elf-magic operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse config: {0}")]
    Config(String),

    #[error("Failed to discover workspace: {0}")]
    WorkspaceDiscovery(String),

    #[error("Failed to build program {program}: {error}")]
    ProgramBuild { program: String, error: String },

    #[error("Failed to generate code: {0}")]
    CodeGeneration(String),

    #[error(transparent)]
    Metadata(#[from] cargo_metadata::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_error_display() {
        let error = Error::Config("Invalid configuration".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to parse config: Invalid configuration"
        );
    }

    #[test]
    fn test_workspace_discovery_error_display() {
        let error = Error::WorkspaceDiscovery("Could not find workspace".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to discover workspace: Could not find workspace"
        );
    }

    #[test]
    fn test_program_build_error_display() {
        let error = Error::ProgramBuild {
            program: "my_program".to_string(),
            error: "Compilation failed".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Failed to build program my_program: Compilation failed"
        );
    }

    #[test]
    fn test_code_generation_error_display() {
        let error = Error::CodeGeneration("Template rendering failed".to_string());
        assert_eq!(
            error.to_string(),
            "Failed to generate code: Template rendering failed"
        );
    }

    #[test]
    fn test_metadata_error_conversion() {
        let metadata_error = cargo_metadata::Error::CargoMetadata {
            stderr: "cargo error".to_string(),
        };
        let error: Error = metadata_error.into();

        match error {
            Error::Metadata(_) => {} // Expected
            _ => panic!("Expected Metadata error variant"),
        }
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = Error::Config("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("test"));
    }
}
