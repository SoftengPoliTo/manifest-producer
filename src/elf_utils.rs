use std::{fs::File, io::Read};

use crate::error;
use capstone::prelude::*;
use error::{Error, Result};
use goblin::elf::{Elf, SectionHeader};
use object::elf::SHT_PROGBITS;

/// Structure used to collect API data identified in the code.
pub struct API {
    /// The name of the API.
    pub name: String,
    /// The starting address of the API.
    pub start_addr: u64,
    /// The ending address of the API.
    pub end_addr: u64,
    /// The list of system calls associated with the API.
    pub syscalls: Vec<String>,
}

impl API {
    /// Creates a new API instance with the specified name, start address, and end address.
    pub fn new(name: String, start_addr: u64, end_addr: u64) -> Self {
        Self {
            name,
            start_addr,
            end_addr,
            syscalls: Vec::new(),
        }
    }
    /// Adds a system call to the list of system calls associated with the API.
    pub fn add_syscall(&mut self, syscall: String) {
        self.syscalls.push(syscall);
    }
}

/// Read the contents of an ELF file.
///
/// # Arguments
///
/// * `file_path` - The path to the ELF file.
///
/// # Returns
///
/// Returns a `Result` containing the vector of bytes read from the ELF file.
pub fn read_elf_file(file_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(&file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Check whether the specified ELF file has been stripped of debug symbols.
pub fn is_stripped(elf: &Elf) -> bool {
    match elf.header.e_ident[goblin::elf::header::EI_CLASS] {
        goblin::elf::header::ELFCLASS64
        | goblin::elf::header::ELFCLASS32 => {
            !has_sections(&elf, goblin::elf::section_header::SHT_SYMTAB)
                || !has_sections(&elf, goblin::elf::section_header::SHT_STRTAB)
        }
        _ => true,
    }
}
// Get the architecture type of the ELF file.
fn has_sections(elf: &Elf, section_type: u32) -> bool {
    elf.section_headers.iter().any(|section| section.sh_type == section_type)
}

/// Get the architecture type.
pub fn get_arch<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_machine {
        goblin::elf::header::EM_X86_64 =>  Ok("x86-64"),
        goblin::elf::header::EM_386 =>  Ok("x86"),
        goblin::elf::header::EM_XTENSA => Ok("Xtensa"),
        _ =>  Err(Error::InvalidElf { source: goblin::error::Error::Malformed("Unknown Architecture".to_string())}),
    }
}

/// Return the type of the ELF file.
pub fn get_file_type<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_type {
        goblin::elf::header::ET_EXEC => Ok("Executable"),
        goblin::elf::header::ET_DYN => Ok("Dynamic Library"),
        goblin::elf::header::ET_CORE => Ok("File core"),
        _ => Err(Error::InvalidElf { source: goblin::error::Error::Malformed("Unknown File Type".to_string())}),
    }
}

/// Check if the ELF file is statically linked.
pub fn is_static(elf: &Elf) -> bool {
    if elf.dynamic.is_some() {
        false
    } else {
        true
    }
}

/// Locate the `.text` section in the ELF file.
pub fn find_text_section<'a>(elf: &'a Elf<'a>) -> Option<&'a SectionHeader>{
    elf
        .section_headers
        .iter()
        .find(|sec| sec.sh_type == SHT_PROGBITS && {
            let name = elf.shdr_strtab.get_at(sec.sh_name);
            name == Some(".text")
        })
}

/// Initialize Capstone disassembly engine.
pub fn cs_init() -> Result<Capstone> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build();
    cs.map_err(|err| {
        Error::Capstone(format!("Failed to create Capstone instance: {}", err))
    })
}

/// Retrieve the name associated with the given address in the ELF file.
///
/// # Arguments
///
/// * `elf` - A reference to the ELF structure representing the binary file.
/// * `address` - The address for which to retrieve the name.
///
/// # Returns
///
/// Returns an optional reference to the name associated with the given address.
pub fn get_name_addr<'a>(elf: &'a Elf<'a>, address: u64) -> Option<&'a str> {
    let symtab = &elf.syms;
    let dyntab = &elf.dynsyms;
    if let Some(sym) = symtab.iter().find(|sym| sym.st_value == address) {
        if let Some(name) = elf.strtab.get_at(sym.st_name) {
            return Some(name);
        }
    } 
    else if let Some(dsym) = dyntab.iter().find(|dsym| dsym.st_value == address) {
        if let Some(name) = elf.dynstrtab.get_at(dsym.st_name) {
            return Some(name);
        }
    }
    else if let Some(text_section) = find_text_section(elf) {
        if let Some(name) = elf.strtab.get_at(text_section.sh_name) {
            return Some(name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_elf_file() {
        // Assicurati di avere un percorso valido a un file ELF per eseguire il test
        let file_path = "./elf_file/fake-firmware-rust-dynamic";
        let result = read_elf_file(file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cs_init() {
        let result = cs_init();
        assert!(result.is_ok(), "Failed to initialize Capstone: {:?}", result.err());
    }
}

