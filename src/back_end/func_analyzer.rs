use crate::back_end::error::Result;

use cpp_demangle::{DemangleOptions, Symbol};
use rustc_demangle::demangle;

use goblin::elf::Elf;

/// Represents a function within an ELF binary.
///
/// # Fields
/// * `name` - The name of the function.
/// * `start_address` - The starting address of the function in the binary.
/// * `end_address` - The ending address of the function in the binary.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FUNC {
    pub name: String,
    pub start_address: u64,
    pub end_address: u64,
}
impl FUNC {
    /// Creates a new `FUNC` instance.
    ///
    /// # Arguments
    /// * `name` - The name of the function.
    /// * `start_address` - The starting address of the function.
    /// * `end_address` - The ending address of the function.
    pub fn new(name: String, start_address: u64, end_address: u64) -> Self {
        Self {
            name,
            start_address,
            end_address,
        }
    }
}

/// Represents a node for call tree.
///
/// # Fields
/// * `name` - The name of the root function.
/// * `invocation_count` - The number of times the function is invoked.
/// * `nodes` - A list of child functions that this function calls.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CallTree {
    pub name: String,
    pub invocation_count: usize,
    pub nodes: Vec<String>,
}
impl CallTree {
    /// Creates a new `CallTree` instance.
    ///
    /// # Arguments
    /// * `name` - The name of the root function.
    pub fn new(name: String) -> Self {
        Self {
            name,
            invocation_count: 0,
            nodes: vec![],
        }
    }
    /// Adds a new child node to the call tree.
    ///
    /// # Arguments
    /// * `node` - The name of the function to add as a child.
    pub fn add_node(&mut self, node: String) {
        self.nodes.push(node);
    }
}

/// Detects and retrieves functions from an ELF binary.
///
/// # Arguments
/// * `elf` - A reference to the ELF object.
/// * `language` - The programming language used (e.g., Rust or C++).
///
/// # Returns
/// Returns a vector of `FUNC` objects containing the name, start, and end addresses of the detected functions.
///
/// # Errors
/// Returns an error if the function name cannot be demangled.
pub fn functions_detection<'a>(elf: &'a Elf<'a>, language: &str) -> Result<Vec<FUNC>> {
    let mut func_found = Vec::new();
    for symbol in &elf.syms {
        if symbol.st_type() == goblin::elf::sym::STT_FUNC && symbol.st_shndx != 0 {
            if let Some(func_name) = get_name_symbol(elf, &symbol.to_owned()) {
                let demangled_name = demangle_function_name(func_name, language)?;
                func_found.push(FUNC::new(
                    demangled_name.clone(),
                    symbol.st_value,
                    symbol.st_value + symbol.st_size,
                ))
            }
        }
    }
    Ok(func_found)
}

/// Demangles a function name based on the programming language.
///
/// # Arguments
/// * `mangled_name` - The mangled (encoded) name of the function.
/// * `language` - The programming language (Rust or C++).
///
/// # Returns
/// Returns the demangled function name as a `String`.
///
/// # Errors
/// Returns an error if the demangling fails.
pub fn demangle_function_name(mangled_name: &str, language: &str) -> Result<String> {
    match language {
        "Rust" | "rust" => {
            let demangled_name = demangle(mangled_name).to_string();
            Ok(demangled_name)
        }
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

/// Retrieves the symbol name from the ELF binary.
fn get_name_symbol<'a>(elf: &'a Elf<'a>, symbol: &'a goblin::elf::Sym) -> Option<&'a str> {
    let name_offset = symbol.st_name;
    let name_str: &str = elf.strtab.get_at(name_offset)?;
    Some(name_str)
}

/// Searches for a specific function by name from a list of functions.
pub fn get_function(functions: &[FUNC], name: &str) -> Option<FUNC> {
    // TODO: implement the get_function(..) method considering retrieving a possible function from the .plt section or the GOT
    functions.iter().find(|func| func.name == name).cloned()
}
