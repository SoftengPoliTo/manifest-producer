use std::collections::HashMap;

use crate::{error::Result, FunctionNode};
use cpp_demangle::{DemangleOptions, Symbol};
use goblin::{self, elf::Elf};
use rustc_demangle::demangle;

pub fn function_detection<'a>(
    elf: &'a Elf<'a>,
    language: &str,
) -> Result<HashMap<String, FunctionNode>> {
    let mut func_found = HashMap::new();

    for symbol in &elf.syms {
        if symbol.st_type() == goblin::elf::sym::STT_FUNC && symbol.st_shndx != 0 {
            if let Some(func_name) = get_name_symbol(elf, &symbol) {
                let demangled_name = demangle_function_name(func_name, language)?;
                func_found.insert(
                    demangled_name.clone(),
                    FunctionNode::new(
                        demangled_name,
                        symbol.st_value,
                        symbol.st_value + symbol.st_size,
                    ),
                );
            }
        }
    }

    Ok(func_found)
}

pub(crate) fn demangle_function_name(mangled_name: &str, language: &str) -> Result<String> {
    match language {
        "Rust" | "rust" => Ok(demangle(mangled_name).to_string()),
        "C_plus_plus_14" | "C++" => {
            if mangled_name.starts_with("_Z") {
                let options = DemangleOptions::default();
                let demangled_name = Symbol::new(mangled_name)?.demangle(&options)?;
                Ok(demangled_name)
            } else {
                Ok(mangled_name.to_string())
            }
        }
        _ => Ok(mangled_name.to_string()),
    }
}

fn get_name_symbol<'a>(elf: &'a Elf<'a>, symbol: &'a goblin::elf::Sym) -> Option<&'a str> {
    elf.strtab.get_at(symbol.st_name)
}
