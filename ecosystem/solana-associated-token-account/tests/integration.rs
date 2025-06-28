use elf_magic_solana_associated_token_account::*;

#[test]
fn validate_elf_constants() {
    // Test SPL_ASSOCIATED_TOKEN_ACCOUNT_ELF
    assert!(!SPL_ASSOCIATED_TOKEN_ACCOUNT_ELF.is_empty());
    assert_eq!(&SPL_ASSOCIATED_TOKEN_ACCOUNT_ELF[0..4], b"\x7fELF");
}

#[test]
fn validate_elves_function() {
    let programs = elves();
    assert_eq!(programs.len(), 1);

    // Check that the program is included
    let program_names: Vec<&str> = programs.iter().map(|(name, _)| *name).collect();
    assert!(program_names.contains(&"spl_associated_token_account"));

    // Verify ELF binary is valid
    for (name, elf_bytes) in programs {
        assert!(!elf_bytes.is_empty(), "ELF binary for {} is empty", name);
        assert_eq!(
            &elf_bytes[0..4],
            b"\x7fELF",
            "Invalid ELF magic for {}",
            name
        );
    }
}
