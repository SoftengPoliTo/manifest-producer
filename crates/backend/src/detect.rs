use std::collections::HashMap;

use crate::{error::Result, FunctionNode};
use cpp_demangle::{DemangleOptions, Symbol};
use goblin::{self, elf::Elf};
use rustc_demangle::demangle;

/// Detects functions in an ELF binary from its symbol table.
///
/// # Overview
///
/// This function scans the ELF symbol table, identifies functions, and processes them into
/// [`FunctionNode`] structures with details like start and end addresses. Function names are demangled if necessary.
///
/// # Arguments
///
/// - `elf`: A reference to an [`Elf`] structure containing the binary's symbol table.
/// - `language`: The programming language for function name demangling.
///
/// # Returns
///
/// - A `Result` containing a `HashMap<String, FunctionNode>` with function names as keys.
///
/// # Errors
///
/// - Returns errors if symbol name demangling fails.
///
/// # Feature Flags
///
/// - `progress_bar`: If enabled, displays a spinner indicating the function detection.
pub fn function_detection<'a>(
    elf: &'a Elf<'a>,
    language: &str,
    output_path: &'a str,
) -> Result<HashMap<String, FunctionNode>> {
    let mut func_found = HashMap::new();

    #[cfg(feature = "progress_bar")]
    let pb = {
        use indicatif::{ProgressBar, ProgressStyle};
        use std::time::Duration;

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Detection of the functions".to_string());
        pb
    };

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

    #[cfg(feature = "progress_bar")]
    pb.finish_with_message(format!(
        "Detection completed! Found {} functions and saved them in functions_list.json in {output_path}/json",
        func_found.len()
    ));

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
