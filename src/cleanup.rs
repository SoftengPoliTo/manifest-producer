use crate::{error, elf_utils};
use error::Result;

use elf_utils::API;
use cpp_demangle::{DemangleOptions, Symbol};
use rustc_demangle::demangle;

/// Encapsulate the call flow within the appropriate structure.
///
/// This function encapsulates the call flow within the API structure, cleaning up and adding the system calls.
///
/// # Arguments
///
/// * `api` - A mutable reference to the API structure.
/// * `sys` - A vector containing the system call names.
/// * `lang` - A string indicating the programming language used (e.g., "Rust", "C++").
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn syscall_flow(api: &mut API, sys: Vec<String>, lang: &str) -> Result<()>{
    for s in sys {
        if lang.contains("Rust") {
            if let Some(name) = clean_rust(&demangle_function_name(&s, true)?) {
                api.add_syscall(name);
            }
        }
        else {
            if let Some(name) = clean_cpp(&demangle_function_name(&s, false)?) {
                api.add_syscall(name);
            }
        }
    }
    Ok(())
}

// This function attempts to demangle the mangled function names.
fn demangle_function_name(mangled_name: &str, rust: bool) -> Result<String> {
    if mangled_name.starts_with("_Z") {
        if rust {
            let demangled_name = demangle(mangled_name).to_string();
            return Ok(demangled_name);
        } 
        let options = DemangleOptions::default(); 
        let demangled_name = Symbol::new(mangled_name)?.demangle(&options)?; 
        Ok(demangled_name)
    } else {
        Ok(mangled_name.to_string())
    }
}

// This function cleans up the demangled Rust function names.
fn clean_rust(demangled_name: &str) -> Option<String> {
    let excluded_keywords = vec!["core::result", "shake_intern", "core::iter"];
    let contains_excluded = excluded_keywords.iter().any(|&keyword| demangled_name.contains(keyword));
    if contains_excluded {
        None
    } else {
        Some(demangled_name.to_string())
    }
}

// This function cleans up the demangled C/C++ function names.
fn clean_cpp(demangled_name: &str) -> Option<String> {
    let excluded_keywords = vec!["_Unwind_Resume", "shake_intern", "value_", "__cxa"];
    let contains_excluded = excluded_keywords.iter().any(|&keyword| demangled_name.contains(keyword));
    if contains_excluded {
        None
    } else {
        Some(demangled_name.to_string())
    }
}
