use elf_magic_solana_token_2022::*;

#[test]
fn validate_elf_constants() {
    // Test SPL_TOKEN_2022_ELF - main Token 2022 program
    assert!(!SPL_TOKEN_2022_ELF.is_empty());
    assert_eq!(&SPL_TOKEN_2022_ELF[0..4], b"\x7fELF");

    // Test SPL_ELGAMAL_REGISTRY_ELF - ElGamal Registry program
    assert!(!SPL_ELGAMAL_REGISTRY_ELF.is_empty());
    assert_eq!(&SPL_ELGAMAL_REGISTRY_ELF[0..4], b"\x7fELF");
}

#[test]
fn validate_elves_function() {
    let programs = elves();
    assert_eq!(programs.len(), 2);

    // Check that both programs are included
    let program_names: Vec<&str> = programs.iter().map(|(name, _)| *name).collect();
    assert!(program_names.contains(&"spl_token_2022"));
    assert!(program_names.contains(&"spl_elgamal_registry"));

    // Verify all ELF binaries are valid
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
