use cpp_demangle::{DemangleOptions, Symbol};
use rustc_demangle::demangle;

use crate::{elf_utils, error};
use elf_utils::API;
use error::Result;

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
pub fn syscall_flow(api: &mut API, sys: Vec<String>, lang: &str) -> Result<()> {
    for s in sys {
        if lang.contains("Rust") {
            if let Some(name) = clean_rust(&demangle_function_name(&s, true)?) {
                api.add_syscall(name);
            }
        } else if let Some(name) = clean_cpp(&demangle_function_name(&s, false)?) {
            api.add_syscall(name);
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
    let excluded_keywords = ["core::result", "shake_intern", "core::iter"];
    let contains_excluded = excluded_keywords
        .iter()
        .any(|&keyword| demangled_name.contains(keyword));
    if contains_excluded {
        None
    } else {
        Some(demangled_name.to_string())
    }
}

// This function cleans up the demangled C/C++ function names.
fn clean_cpp(demangled_name: &str) -> Option<String> {
    let excluded_keywords = ["_Unwind_Resume", "shake_intern", "value_", "__cxa"];
    let contains_excluded = excluded_keywords
        .iter()
        .any(|&keyword| demangled_name.contains(keyword));
    if contains_excluded {
        None
    } else {
        Some(demangled_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demangle_function_name_rust() {
        let mangled_name = "_ZN4core9panicking16panic_in_cleanup17h55eb1d85cadde1a1E";
        let demangled_name = demangle_function_name(mangled_name, true).unwrap();
        assert_eq!(
            demangled_name,
            "core::panicking::panic_in_cleanup::h55eb1d85cadde1a1"
        );
    }

    #[test]
    fn test_demangle_function_name_cpp() {
        let mangled_name = "_ZN12example_name3fooE";
        let demangled_name = demangle_function_name(mangled_name, false).unwrap();
        assert_eq!(demangled_name, "example_name::foo");
    }

    #[test]
    fn test_clean_rust_excluded_keyword() {
        let demangled_name = "core::result::Result";
        let result = clean_rust(demangled_name);
        assert_eq!(result, None);
    }

    #[test]
    fn test_clean_rust_no_excluded_keyword() {
        let demangled_name = "my_function";
        let result = clean_rust(demangled_name).unwrap();
        assert_eq!(result, "my_function");
    }

    #[test]
    fn test_clean_cpp_excluded_keyword() {
        let demangled_name = "__cxa_throw";
        let result = clean_cpp(demangled_name);
        assert_eq!(result, None);
    }

    #[test]
    fn test_clean_cpp_no_excluded_keyword() {
        let demangled_name = "my_function";
        let result = clean_cpp(demangled_name).unwrap();
        assert_eq!(result, "my_function");
    }
}
