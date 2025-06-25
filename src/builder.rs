use crate::domain::{ElfMagicError, SolanaProgram};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Build multiple Solana programs
///
/// Returns the paths to the generated .so files in the same order as input programs.
pub fn build_programs(
    cargo_target_dir: &Path,
    programs: &[SolanaProgram],
) -> Result<Vec<PathBuf>, ElfMagicError> {
    programs
        .iter()
        .map(|program| build_program(cargo_target_dir, program))
        .collect()
}

/// Build a single Solana program using cargo build-sbf
///
/// Executes cargo build-sbf on the provided program and returns
/// the path to the generated .so file.
pub fn build_program(
    cargo_target_dir: &Path,
    program: &SolanaProgram,
) -> Result<PathBuf, ElfMagicError> {
    // Create elf-magic subdirectory for our Solana program builds
    let sbf_out_dir = cargo_target_dir.join("elf-magic-bin");

    // Expected output path for the .so file
    let program_so_path = sbf_out_dir.join(format!("{}.so", program.name));

    // Remove existing .so file to ensure clean build
    if program_so_path.exists() {
        fs::remove_file(&program_so_path).map_err(|e| ElfMagicError::ProgramBuild {
            program: program.name.clone(),
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
        .env(
            "CARGO_TARGET_DIR", // note cargo-build-sbf doesn't honor CARGO_TARGET_DIR well, but we should set it anyway
            cargo_target_dir.to_string_lossy().into_owned(),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| ElfMagicError::ProgramBuild {
            program: program.name.clone(),
            error: format!(
                "Failed to execute cargo build-sbf: {}\nMake sure solana CLI tools are installed",
                e
            ),
        })?;

    if !status.success() {
        return Err(ElfMagicError::ProgramBuild {
            program: program.name.clone(),
            error: format!("cargo build-sbf failed with exit code: {:?}", status.code()),
        });
    }

    // Verify the .so file was created
    if !program_so_path.exists() {
        return Err(ElfMagicError::ProgramBuild {
            program: program.name.clone(),
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
