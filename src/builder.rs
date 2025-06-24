use crate::domain::{BuiltProgram, ElfMagicError, SolanaProgram};
use std::path::PathBuf;

/// Build a single Solana program using cargo build-sbf
///
/// Executes cargo build-sbf on the provided program and returns
/// the path to the generated .so file along with metadata.
pub fn build_single_program(program: &SolanaProgram) -> Result<BuiltProgram, ElfMagicError> {
    // TODO: Determine target directory (honor CARGO_TARGET_DIR or use temp)
    // TODO: Run cargo build-sbf with correct arguments
    // TODO: Find the generated .so file
    // TODO: Generate environment variable name from program name
    // TODO: Return BuiltProgram with all paths and metadata
    todo!("implement build_single_program")
}

/// Build multiple Solana programs in parallel
///
/// This could be optimized to build programs in parallel since
/// cargo build-sbf operations are independent.
pub fn build_programs_parallel(
    programs: &[SolanaProgram],
) -> Result<Vec<BuiltProgram>, ElfMagicError> {
    // TODO: Consider parallel execution with rayon
    // TODO: For now, sequential is fine
    programs.iter().map(build_single_program).collect()
}

/// Generate environment variable name from program name
///
/// Converts "token-manager" to "PROGRAM_TOKEN_MANAGER_SO_PATH"
pub fn program_env_var_name(program_name: &str) -> String {
    // TODO: Convert kebab-case to SCREAMING_SNAKE_CASE
    // TODO: Add PROGRAM_ prefix and _SO_PATH suffix
    todo!("implement program_env_var_name")
}

/// Generate constant name from program name
///
/// Converts "token-manager" to "TOKEN_MANAGER_ELF"
pub fn program_constant_name(program_name: &str) -> String {
    // TODO: Convert kebab-case to SCREAMING_SNAKE_CASE
    // TODO: Add _ELF suffix
    todo!("implement program_constant_name")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_env_var_name() {
        // TODO: Test name conversion
        // assert_eq!(program_env_var_name("token-manager"), "PROGRAM_TOKEN_MANAGER_SO_PATH");
        todo!("implement test")
    }

    #[test]
    fn test_program_constant_name() {
        // TODO: Test name conversion
        // assert_eq!(program_constant_name("token-manager"), "TOKEN_MANAGER_ELF");
        todo!("implement test")
    }

    #[test]
    fn test_build_single_program() {
        // TODO: Test with mock program
        todo!("implement test")
    }
}
