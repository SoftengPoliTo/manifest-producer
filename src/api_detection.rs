use goblin::elf::Elf;

use crate::{elf_utils, error};
use elf_utils::API;
use error::Result;

/// Do an API lookup in the symbol table.
///
/// This function searches for APIs in the symbol table of the ELF file based on a list of API names provided.
///
/// # Arguments
///
/// * `elf` - The ELF file structure.
/// * `api_list` - A vector containing the names of the APIs to search for.
///
/// # Returns
///
/// Returns a `Result` containing a vector of `API` structures representing the APIs found.
pub fn api_search<'a>(elf: &'a Elf<'a>, api_list: &'a [&'a str]) -> Result<Vec<API>> {
    let mut api_found = Vec::new();
    for symbol in &elf.syms {
        if symbol.st_type() == goblin::elf::sym::STT_FUNC && symbol.st_shndx != 0 {
            if let Some(function_name) = get_name_sym(elf, &symbol.to_owned()) {
                if api_list.contains(&function_name) {
                    api_found.push(API::new(
                        function_name.to_string(),
                        symbol.st_value,
                        symbol.st_value + symbol.st_size,
                    ));
                }
            }
        }
    }
    Ok(api_found)
}

// This function retrieves the name of a symbol from the ELF symbol table.
fn get_name_sym<'a>(elf: &'a Elf, symbol: &'a goblin::elf::Sym) -> Option<&'a str> {
    let name_offset = symbol.st_name;
    let name_str: &'a str = elf.strtab.get_at(name_offset)?;
    Some(name_str)
}
