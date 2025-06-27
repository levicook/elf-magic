use elf_magic_solana_memo::*;

#[test]
fn validate_elf_constants() {
    // Test P_MEMO_ELF
    assert!(!P_MEMO_ELF.is_empty());
    assert_eq!(&P_MEMO_ELF[0..4], b"\x7fELF");

    // Test SPL_MEMO_ELF
    assert!(!SPL_MEMO_ELF.is_empty());
    assert_eq!(&SPL_MEMO_ELF[0..4], b"\x7fELF");
}

#[test]
fn validate_elves_function() {
    let programs = elves();
    assert_eq!(programs.len(), 2);

    // Check that both programs are included
    let program_names: Vec<&str> = programs.iter().map(|(name, _)| *name).collect();
    assert!(program_names.contains(&"p_memo"));
    assert!(program_names.contains(&"spl_memo"));

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
