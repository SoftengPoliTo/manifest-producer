use std::path::Path;

use goblin::elf::Elf;
use manifest_producer_backend::error::Result;
use manifest_producer_backend::inspect::read_elf;
use manifest_producer_checker::checks::{
    bad_magic_report, display_cli_results, json_results, malformed_report, validate_elf_file,
};

pub fn perform_checks(elf_path: &str, file_path: &str) -> Result<()> {
    let file_data = read_elf(elf_path)?;
    let file_size = file_data.len() as u64;

    let report = match Elf::parse(&file_data) {
        Ok(elf) => validate_elf_file(&elf, Path::new(file_path), file_size),
        Err(goblin::error::Error::BadMagic(_)) => bad_magic_report(Path::new(file_path)),
        Err(goblin::error::Error::Malformed(e)) => {
            let description = e.to_string();
            malformed_report(Path::new(file_path), &description)
        }
        Err(goblin::error::Error::Scroll(e)) => {
            let description = e.to_string();
            let kind = if description.contains("bad offset") {
                "Invalid Program/Section Header offset in ELF file"
            } else {
                "Scroll error while parsing ELF file"
            };
            malformed_report(Path::new(file_path), kind)
        }
        Err(e) => {
            let description = e.to_string();
            malformed_report(Path::new(file_path), &description)
        }
    };

    display_cli_results(&report);
    json_results(&report);
    Ok(())
}
