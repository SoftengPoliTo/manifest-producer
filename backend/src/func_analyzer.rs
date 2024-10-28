use common::{
    cpp_demangle::{DemangleOptions, Symbol},
    error::Result,
    goblin::{self, elf::Elf},
    rustc_demangle::demangle,
    FUNC,
};

pub fn extract_functions<'a>(elf: &'a Elf<'a>, language: &str) -> Result<Vec<FUNC>> {
    let mut func_found = Vec::new();
    for symbol in &elf.syms {
        if symbol.st_type() == goblin::elf::sym::STT_FUNC && symbol.st_shndx != 0 {
            if let Some(func_name) = get_name_symbol(elf, &symbol) {
                let demangled_name = demangle_function_name(func_name, language)?;
                func_found.push(FUNC::new(
                    demangled_name,
                    symbol.st_value,
                    symbol.st_value + symbol.st_size,
                ));
            }
        }
    }
    Ok(func_found)
}

pub fn demangle_function_name(mangled_name: &str, language: &str) -> Result<String> {
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

pub fn get_function(functions: &[FUNC], name: &str) -> Option<FUNC> {
    functions.iter().find(|func| func.name == name).cloned()
}

fn get_name_symbol<'a>(elf: &'a Elf<'a>, symbol: &'a goblin::elf::Sym) -> Option<&'a str> {
    elf.strtab.get_at(symbol.st_name)
}
