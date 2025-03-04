use std::collections::HashMap;

use regex::Regex;

use crate::{
    error::{Error, Result},
    FunctionNode,
};

/// Identifies the main function starting from the _start function in the disassembly.
///
/// # Overview
///
/// The `find_main` function extracts the address of the `main` function by analyzing
/// the disassembly of the `_start` function. In the x86-64 calling convention, the first argument
/// to a function is passed in the `%rdi` register. Before invoking `__libc_start_main`, the address
/// of the `main` function is loaded into `%rdi`. The `find_main` function looks for a `mov` instruction
/// that loads the address of `main` into `%rdi` in the disassembly of `_start`. This is the point at which
/// the address of `main` is set up for `__libc_start_main`. Once the address is extracted, it searches
/// through the functions to find the one corresponding to `main`. If the `main` function cannot be found,
/// an error is returned.
///
/// # Arguments
///
/// - `functions`: A mutable reference to a `HashMap` mapping function names to their corresponding
///   [`FunctionNode`] structures. The function names should include `_start` and possibly `main`.
///
/// # Returns
///
/// - A `Result` containing the [`FunctionNode`] corresponding to the `main` function if found,
///   or an error if not.
///
/// # Errors
///
/// - `Error::FunctionNotFound("main")`: If the main function is not found in the map of functions,
///   or if no valid main function address is extracted from `_start`'s disassembly.
/// - `Error::FunctionNotFound("_start")`: If the `_start` function is not present in the functions map.
/// - `Error::FunctionNotFound("_start disassembly")`: If no disassembly is available for `_start`.
/// - `Error::InvalidRegex`: If there is an issue while parsing the disassembly with the regex used
///   to extract the main address.
///
/// # Feature Flags
///
/// - `progress_bar`: If enabled, displays a spinner indicating the extraction process of the main function.
///
pub fn find_main<S: ::std::hash::BuildHasher>(
    functions: &HashMap<String, FunctionNode, S>,
) -> Result<FunctionNode> {
    #[cfg(feature = "progress_bar")]
    let pb = {
        use indicatif::{ProgressBar, ProgressStyle};
        use std::time::Duration;

        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}\n")?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Extracting main function: _start -> main...".to_string());
        pb
    };

    if let Some(start) = functions.get("_start") {
        if let Some(disassembly) = &start.disassembly {
            let main_addr = extract_main(disassembly)?;
            if main_addr != 0 {
                for func in functions {
                    if func.1.start_addr == main_addr {
                        #[cfg(feature = "progress_bar")]
                        pb.finish_with_message("Main function found!");
                        return Ok(func.1.clone());
                    }
                }
                return Err(Error::FunctionNotFound("main".to_string()));
            }
        } else {
            return Err(Error::FunctionNotFound("_start disassembly".to_string()));
        }
    } else if let Some(start) = functions.get("__dls2") {
        if let Some(disassembly) = &start.disassembly {
            let main_addr = extract_main(disassembly)?;
            if main_addr != 0 {
                for func in functions {
                    if func.1.start_addr == main_addr {
                        #[cfg(feature = "progress_bar")]
                        pb.finish_with_message("Main function found!");
                        return Ok(func.1.clone());
                    }
                }
                return Err(Error::FunctionNotFound("main".to_string()));
            }
        } else {
            return Err(Error::FunctionNotFound("_start disassembly".to_string()));
        }
    }

    #[cfg(feature = "progress_bar")]
    pb.finish_with_message("Main function not found.");
    Err(Error::FunctionNotFound("main".to_string()))
}

fn extract_main(disassembly: &str) -> Result<u64> {
    let re_c = Regex::new(r"mov\s+\$([a-fA-F0-9x]+),\s+%rdi")?;
    let re_rust = Regex::new(r"([0-9a-fA-F]+):\s+lea\s+0x([0-9a-fA-F]+)\(%rip\),\s+%rdi")?;

    for line in disassembly.lines() {
        if let Some(caps) = re_c.captures(line) {
            let addr_str = caps.get(1).unwrap().as_str();
            let addr = if let Some(addr_str) = addr_str.strip_prefix("0x") {
                u64::from_str_radix(addr_str, 16).unwrap()
            } else {
                return Ok(0);
            };
            return Ok(addr);
        }
        if let Some(caps) = re_rust.captures(line) {
            let lea_instruction_address_str = caps.get(1).unwrap().as_str();
            let lea_instruction_address =
                u64::from_str_radix(lea_instruction_address_str, 16).unwrap();

            let offset_str = caps.get(2).unwrap().as_str();
            let offset = u64::from_str_radix(offset_str, 16).unwrap();

            let rip_value = lea_instruction_address + 7;
            let main_address = rip_value + offset;

            return Ok(main_address);
        }
    }
    Ok(0)
}

pub(crate) fn calculate_invocation_count(functions: &mut HashMap<String, FunctionNode>) {
    let nodes_to_update: Vec<_> = functions
        .values()
        .flat_map(|node| node.children.clone())
        .collect();

    for node_name in nodes_to_update {
        if let Some(func) = functions.get_mut(&node_name) {
            func.invocation_entry += 1;
        }
    }
}
