// #[test]
// fn test_ecosystem_package_has_expected_elves() {
//     let elves = elf_magic_solana_spl_token::elves();

//     // Should have exactly 2 programs: pinocchio_token_program and spl_token
//     assert_eq!(
//         elves.len(),
//         2,
//         "Expected exactly 2 ELF binaries in solana-spl-token ecosystem package"
//     );

//     // Verify the programs are present
//     let program_names: Vec<&str> = elves.iter().map(|(name, _)| *name).collect();
//     assert!(
//         program_names.contains(&"pinocchio_token_program"),
//         "Missing pinocchio_token_program"
//     );
//     assert!(program_names.contains(&"spl_token"), "Missing spl_token");

//     // Verify each ELF binary is non-empty
//     for (name, elf_bytes) in elves {
//         assert!(!elf_bytes.is_empty(), "ELF binary for {} is empty", name);
//         // Basic sanity check - ELF files start with magic bytes
//         assert_eq!(
//             &elf_bytes[0..4],
//             b"\x7fELF",
//             "Invalid ELF magic bytes for {}",
//             name
//         );
//     }
// }
