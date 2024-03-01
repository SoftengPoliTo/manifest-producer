use crate::{error, elf_utils};
use elf_utils::API;
use error::Result;
use goblin::elf::Elf;

/* 
*
*   API Detection: Identifies the functions declared by the developer.
*
*/

// Do an API lookup in the symbol table.
pub fn api_search<'a>(elf: &'a Elf<'a>, api_list: &'a Vec<&'a str>) -> Result<Vec<API>> {
    let mut api_found = Vec::new();
    for symbol in &elf.syms {
        if symbol.st_type() == goblin::elf::sym::STT_FUNC && symbol.st_shndx != 0 {
            if let Some(function_name) = get_name_sym(&elf, &symbol.to_owned()) {
                if api_list.contains(&function_name) {
                    api_found.push(API::new(function_name.to_string(), symbol.st_value, symbol.st_value+symbol.st_size));
                }
            }
        }
    }
    Ok(api_found)
}

// Retrieve the name given the symbol.
fn get_name_sym<'a>(elf: &'a Elf, symbol: &'a goblin::elf::Sym) -> Option<&'a str> {
    let name_offset = symbol.st_name as usize;
    let name_str: &'a str = &elf.strtab.get_at(name_offset)?;
    Some(name_str)
}
