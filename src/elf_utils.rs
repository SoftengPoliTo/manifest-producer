use std::{fs::File, io::Read};

use crate::error;
use capstone::prelude::*;
use error::{Error, Result};
use goblin::elf::{Elf, SectionHeader};
use object::elf::SHT_PROGBITS;


/* 
*
*   ELF Utilities: Functions for retrieving basic information from ELF files.
*
*/

// Structure used to collect API data identified in the code.
pub struct API {
    pub name: String,
    pub start_addr: u64,
    pub end_addr: u64,
    pub syscalls: Vec<String>,
}

impl API {
    pub fn new(name: String, start_addr: u64, end_addr: u64) -> Self {
        Self {
            name,
            start_addr,
            end_addr,
            syscalls: Vec::new(),
        }
    }

    pub fn add_syscall(&mut self, syscall: String) {
        self.syscalls.push(syscall);
    }
}

// Read the contents of an ELF file.
pub fn read_elf_file(file_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(&file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// Check whether the specified ELF has been stripped of debug symbols.
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

fn has_sections(elf: &Elf, section_type: u32) -> bool {
    elf.section_headers.iter().any(|section| section.sh_type == section_type)
}

// Get the architecture type.
pub fn get_arch<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_machine {
        goblin::elf::header::EM_X86_64 =>  Ok("x86-64"),
        goblin::elf::header::EM_386 =>  Ok("x86"),
        _ =>  Err(Error::InvalidElf { source: goblin::error::Error::Malformed("Unknown Architecture".to_string())}),
    }
}

// Return the file type.
pub fn get_file_type<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_type {
        goblin::elf::header::ET_EXEC => Ok("Executable"),
        goblin::elf::header::ET_DYN => Ok("Dynamic Library"),
        goblin::elf::header::ET_CORE => Ok("File core"),
        _ => Err(Error::InvalidElf { source: goblin::error::Error::Malformed("Unknown File Type".to_string())}),
    }
}

// Check if the file is statically linked.
pub fn is_static(elf: &Elf) -> bool {
    if elf.dynamic.is_some() {
        false
    } else {
        true
    }
}

// Locate the .text section.
pub fn find_text_section<'a>(elf: &'a Elf<'a>) -> Option<&'a SectionHeader>{
    elf
        .section_headers
        .iter()
        .find(|sec| sec.sh_type == SHT_PROGBITS && {
            let name = elf.shdr_strtab.get_at(sec.sh_name);
            name == Some(".text")
        })
}

// Initialize Capstone.
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

// Retrieve the name given the address.
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
