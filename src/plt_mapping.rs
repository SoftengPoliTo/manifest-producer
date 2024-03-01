use std::collections::HashMap;
use goblin::elf::{section_header::SHT_PROGBITS, Elf, SectionHeader};

/* 
*
*   PLT Mapping: Functions for mapping the .plt and .rela.plt sections in dynamically linked ELF binaries.
*
*/

// Load the .rela.plt relocations and return a map of the PLT entry addresses and their symbol names.
pub fn load_rela_plt_relocations<'a>(elf: &'a Elf<'a>, plt_section: &'a SectionHeader, plt_entry_size: usize) -> Option<HashMap<u64, &'a str>> {
    let mut tbl = HashMap::new();
    let mut i = 0;
    for (section_index, relocations) in &elf.shdr_relocs {
        if let Some(section_header) = elf.section_headers.get(*section_index as usize) {
            if let Some(section_name) = elf.shdr_strtab.get_at(section_header.sh_name) {
                if section_name == ".rela.plt" {
                    for r in relocations {
                        if let Some(symbol) = &elf.dynsyms.get(r.r_sym as usize) {
                            if let Some(name) = elf.dynstrtab.get_at(symbol.st_name) {
                                let plt_entry_index = i + 1;
                                let result = plt_entry_address(plt_section, plt_entry_index, plt_entry_size);
                                tbl.insert(result, name);
                                i += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    Some(tbl)
}

// Calculate the PLT entry address based on the index and entry size.
fn plt_entry_address(plt_section: &SectionHeader, index: usize, plt_entry_size: usize) -> u64 {
    let offset = index * plt_entry_size;
    plt_section.sh_addr + offset as u64
}

// Find the .plt section.
pub fn find_plt_section<'a>(elf: &'a Elf<'a>) -> Option<&'a SectionHeader> {
    elf
        .section_headers
        .iter()
        .find(|sec| sec.sh_type == SHT_PROGBITS && {
            let name = elf.shdr_strtab.get_at(sec.sh_name);
            name == Some(".plt")
        })
}
