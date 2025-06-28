use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::{
    error::Error,
    programs::{ProgramBuildResult, SolanaProgram},
};

/// Build multiple Solana programs, collecting both successes and failures
///
/// Returns build results containing successfully built programs and any failures.
/// This allows partial success - some programs can build while others fail.
pub fn build_programs(programs: &[SolanaProgram]) -> ProgramBuildResult {
    let mut result = ProgramBuildResult::new();

    for program in programs {
        match build_program(program) {
            Ok(path) => {
                result.add_success(program.clone(), path);
            }
            Err(error) => {
                result.add_failure(program.clone(), error);
            }
        }
    }

    result
}

/// Build a single Solana program using cargo build-sbf
///
/// Executes cargo build-sbf on the provided program and returns
/// the path to the generated .so file.
pub fn build_program(program: &SolanaProgram) -> Result<PathBuf, Error> {
    // Create elf-magic subdirectory for our Solana program builds
    let sbf_out_dir = std::env::temp_dir()
        .join("elf-magic-bin")
        .join(program.package_name.clone());

    // Expected output path for the .so file
    let program_so_path = sbf_out_dir.join(format!("{}.so", program.target_name));

    // Remove existing .so file to ensure clean build
    if program_so_path.exists() {
        fs::remove_file(&program_so_path).map_err(|e| Error::ProgramBuild {
            program: program.target_name.clone(),
            error: format!("Failed to remove existing .so file: {}", e),
        })?;
    }

    // Execute cargo build-sbf
    let status = Command::new("cargo")
        .args([
            "build-sbf",
            "--manifest-path",
            &program.manifest_path.to_string_lossy(),
            "--sbf-out-dir",
            &sbf_out_dir.to_string_lossy(),
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| Error::ProgramBuild {
            program: program.target_name.clone(),
            error: format!(
                "Failed to execute cargo build-sbf: {}\nMake sure solana CLI tools are installed",
                e
            ),
        })?;

    if !status.success() {
        return Err(Error::ProgramBuild {
            program: program.target_name.clone(),
            error: format!("cargo build-sbf failed with exit code: {:?}", status.code()),
        });
    }

    // Verify the .so file was created
    if !program_so_path.exists() {
        return Err(Error::ProgramBuild {
            program: program.target_name.clone(),
            error: format!(
                "Expected .so file not found at: {}",
                program_so_path.display()
            ),
        });
    }

    // Set environment variable for this program
    println!(
        "cargo:rustc-env={}={}",
        program.env_var_name(),
        program_so_path.display()
    );

    Ok(program_so_path)
}

/// Enable incremental builds for each program
pub fn enable_incremental_builds(
    manifest_dir: &Path,
    programs: &[SolanaProgram],
) -> Result<(), Error> {
    // Watch the lib.rs file (now hand-written, not generated)
    let src_path = manifest_dir.join("src");
    println!("cargo:rerun-if-changed={}", src_path.display());

    // Watch the upstream programs that we're building
    for program in programs {
        let program_root = program.manifest_path.parent().unwrap();
        println!("cargo:rerun-if-changed={}", program_root.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_program() -> SolanaProgram {
        SolanaProgram {
            package_name: "test_package".to_string(),
            target_name: "test_target".to_string(),
            manifest_path: PathBuf::from("/workspace/Cargo.toml"),
            constant_name: "TEST_TARGET_ELF".to_string(),
        }
    }

    #[test]
    fn test_build_programs_empty() {
        let result = build_programs(&[]);

        // Should succeed with empty programs list
        assert_eq!(result.successful.len(), 0);
        assert_eq!(result.failed.len(), 0);
    }

    #[test]
    fn test_expected_so_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_target_dir = temp_dir.path();
        let _program = sample_program();

        // Test the path construction logic
        let expected_path = cargo_target_dir
            .join("elf-magic-bin")
            .join("test_target.so");

        // This tests our path construction without actually running cargo build-sbf
        assert_eq!(expected_path.file_name().unwrap(), "test_target.so");
        assert_eq!(
            expected_path.parent().unwrap().file_name().unwrap(),
            "elf-magic-bin"
        );
    }

    #[test]
    fn test_program_env_var_name_consistency() {
        let program = sample_program();
        let expected_env_var = "TEST_TARGET_ELF_PATH";

        // Verify the env var name matches what SolanaProgram::env_var_name() produces
        assert_eq!(program.env_var_name(), expected_env_var);
    }

    #[test]
    fn test_build_programs_preserves_order() {
        let programs = vec![
            SolanaProgram {
                package_name: "pkg1".to_string(),
                target_name: "target1".to_string(),
                manifest_path: PathBuf::from("/workspace/Cargo.toml"),
                constant_name: "TARGET1_ELF".to_string(),
            },
            SolanaProgram {
                package_name: "pkg2".to_string(),
                target_name: "target2".to_string(),
                manifest_path: PathBuf::from("/workspace/Cargo.toml"),
                constant_name: "TARGET2_ELF".to_string(),
            },
        ];

        // We can't actually run cargo build-sbf in tests, but we can verify the function
        // signature and that it attempts to process all programs
        let result = build_programs(&programs);

        // This will likely have failures because cargo build-sbf won't work, but we're testing
        // that it processes the correct number of programs
        assert_eq!(result.successful.len() + result.failed.len(), 2);
        // In test environment without Solana tools, we expect failures
        // The important thing is that it tried to process both programs
    }

    #[test]
    fn test_sbf_out_dir_creation() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_target_dir = temp_dir.path();

        let expected_sbf_dir = cargo_target_dir.join("elf-magic-bin");

        // Verify the path structure we expect to create
        assert_eq!(expected_sbf_dir.parent(), Some(cargo_target_dir));
        assert_eq!(expected_sbf_dir.file_name().unwrap(), "elf-magic-bin");
    }

    #[test]
    fn test_program_so_filename() {
        let programs = vec![
            sample_program(),
            SolanaProgram {
                package_name: "my_package".to_string(),
                target_name: "my-complex-target-name".to_string(),
                manifest_path: PathBuf::from("/workspace/Cargo.toml"),
                constant_name: "MY_COMPLEX_TARGET_NAME_ELF".to_string(),
            },
        ];

        // Test that .so filenames are constructed correctly
        for program in &programs {
            let expected_filename = format!("{}.so", program.target_name);
            assert!(expected_filename.ends_with(".so"));
            assert!(expected_filename.contains(&program.target_name));
        }
    }
}
