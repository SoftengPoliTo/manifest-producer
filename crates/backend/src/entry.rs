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
// pub fn find_main<S: ::std::hash::BuildHasher>(
//     functions: &HashMap<String, FunctionNode, S>,
// ) -> Result<FunctionNode> {
//     #[cfg(feature = "progress_bar")]
//     let pb = {
//         use indicatif::{ProgressBar, ProgressStyle};
//         use std::time::Duration;

//         let pb = ProgressBar::new_spinner();
//         pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}\n")?);
//         pb.enable_steady_tick(Duration::from_millis(100));
//         pb.set_message("Extracting main function: _start -> main...".to_string());
//         pb
//     };

//     if let Some(start) = functions.get("_start") {
//         if let Some(disassembly) = &start.disassembly {
//             let main_addr = extract_main(disassembly)?;
//             if main_addr != 0 {
//                 for func in functions {
//                     if func.1.start_addr == main_addr {
//                         #[cfg(feature = "progress_bar")]
//                         pb.finish_with_message("Main function found!");
//                         return Ok(func.1.clone());
//                     }
//                 }
//                 return Err(Error::FunctionNotFound("main".to_string()));
//             }
//         } else {
//             return Err(Error::FunctionNotFound("_start disassembly".to_string()));
//         }
//     } else if let Some(start) = functions.get("__dls2") {
//         if let Some(disassembly) = &start.disassembly {
//             let main_addr = extract_main(disassembly)?;
//             if main_addr != 0 {
//                 for func in functions {
//                     if func.1.start_addr == main_addr {
//                         #[cfg(feature = "progress_bar")]
//                         pb.finish_with_message("Main function found!");
//                         return Ok(func.1.clone());
//                     }
//                 }
//                 return Err(Error::FunctionNotFound("main".to_string()));
//             }
//         } else {
//             return Err(Error::FunctionNotFound("_start disassembly".to_string()));
//         }
//     }

//     #[cfg(feature = "progress_bar")]
//     pb.finish_with_message("Main function not found.");
//     Err(Error::FunctionNotFound("main".to_string()))
// }

// fn extract_main(disassembly: &str) -> Result<u64> {
//     let re_c = Regex::new(r"mov\s+\$([a-fA-F0-9x]+),\s+%rdi")?;
//     let re_rust = Regex::new(r"([0-9a-fA-F]+):\s+lea\s+0x([0-9a-fA-F]+)\(%rip\),\s+%rdi")?;

//     for line in disassembly.lines() {
//         if let Some(caps) = re_c.captures(line) {
//             let addr_str = caps.get(1).unwrap().as_str();
//             let addr = if let Some(addr_str) = addr_str.strip_prefix("0x") {
//                 u64::from_str_radix(addr_str, 16).unwrap()
//             } else {
//                 return Ok(0);
//             };
//             return Ok(addr);
//         }
//         if let Some(caps) = re_rust.captures(line) {
//             let lea_instruction_address_str = caps.get(1).unwrap().as_str();
//             let lea_instruction_address =
//                 u64::from_str_radix(lea_instruction_address_str, 16).unwrap();

//             let offset_str = caps.get(2).unwrap().as_str();
//             let offset = u64::from_str_radix(offset_str, 16).unwrap();

//             let rip_value = lea_instruction_address + 7;
//             let main_address = rip_value + offset;

//             return Ok(main_address);
//         }
//     }
//     Ok(0)
// }

// FAKE DA QUI

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
        pb.set_message("Extracting main function: main wrapper -> user main...".to_string());
        pb
    };

    // Strategia semplice: cerca direttamente "main" (che dovrebbe essere il wrapper)
    if let Some(main_wrapper) = functions.get("main") {
        if let Some(disassembly) = &main_wrapper.disassembly {
            #[cfg(feature = "progress_bar")]
            pb.set_message("Found main wrapper, extracting user main address...".to_string());

            let user_main_addr = extract_main(disassembly)?;
            if user_main_addr != 0 {
                // Cerca la funzione che corrisponde all'indirizzo del vero main
                for (func_name, func_node) in functions {
                    if func_node.start_addr == user_main_addr {
                        #[cfg(feature = "progress_bar")]
                        pb.finish_with_message(format!("User main function found: {}", func_name));
                        return Ok(func_node.clone());
                    }
                }

                // Se non troviamo la funzione per nome, potrebbe essere unnamed
                #[cfg(feature = "progress_bar")]
                pb.finish_with_message(format!(
                    "User main found at address: 0x{:x}",
                    user_main_addr
                ));

                return Err(Error::FunctionNotFound(format!(
                    "Function at main address 0x{:x} not found in function map",
                    user_main_addr
                )));
            } else {
                return Err(Error::FunctionNotFound(
                    "Could not extract main address from wrapper".to_string(),
                ));
            }
        } else {
            return Err(Error::FunctionNotFound(
                "main wrapper disassembly".to_string(),
            ));
        }
    }

    // Fallback: se non c'è "main", prova con i punti di ingresso classici
    let entry_points = ["_start", "__start", "start", "__dls2"];

    for entry_name in &entry_points {
        if let Some(start_func) = functions.get(*entry_name) {
            if let Some(disassembly) = &start_func.disassembly {
                #[cfg(feature = "progress_bar")]
                pb.set_message(format!("Trying entry point: {}", entry_name));

                let main_addr = extract_main(disassembly)?;
                if main_addr != 0 {
                    for (func_name, func_node) in functions {
                        if func_node.start_addr == main_addr {
                            #[cfg(feature = "progress_bar")]
                            pb.finish_with_message(format!(
                                "Main function found via {}: {}",
                                entry_name, func_name
                            ));
                            return Ok(func_node.clone());
                        }
                    }
                }
            }
        }
    }

    #[cfg(feature = "progress_bar")]
    pb.finish_with_message("Main function not found.");
    Err(Error::FunctionNotFound("main".to_string()))
}

fn extract_main(disassembly: &str) -> Result<u64> {
    // Regex per istruzioni di movimento diretto (mov, movabs, movl, movq, etc.) verso %rdi
    let re_direct_mov = Regex::new(r"mov[ablqw]*\s+\$0x([a-fA-F0-9]+),\s+%rdi")?;

    // Regex per lea con offset RIP positivo
    let re_rust_lea_pos = Regex::new(r"([0-9a-fA-F]+):\s+lea\s+0x([0-9a-fA-F]+)\(%rip\),\s+%rdi")?;

    // Regex per lea con offset RIP negativo
    let re_rust_lea_neg = Regex::new(r"([0-9a-fA-F]+):\s+lea\s+-0x([0-9a-fA-F]+)\(%rip\),\s+%rdi")?;

    // Regex più permissiva per catturare variazioni
    let re_flexible_mov = Regex::new(r"mov[a-z]*\s+\$([0-9a-fA-F]+|0x[0-9a-fA-F]+),\s+%rdi")?;

    for line in disassembly.lines() {
        println!("Debug line: {}", line);

        // Prova con le istruzioni di movimento diretto
        if let Some(caps) = re_direct_mov.captures(line) {
            let addr_str = caps.get(1).unwrap().as_str();
            let addr = u64::from_str_radix(addr_str, 16).unwrap();
            println!("Found main address via direct mov: 0x{:x}", addr);
            return Ok(addr);
        }

        // Prova con regex più flessibile
        if let Some(caps) = re_flexible_mov.captures(line) {
            let addr_str = caps.get(1).unwrap().as_str();
            let addr_str = addr_str.strip_prefix("0x").unwrap_or(addr_str);
            let addr = u64::from_str_radix(addr_str, 16).unwrap();
            println!("Found main address via flexible mov: 0x{:x}", addr);
            return Ok(addr);
        }

        // Prova con lea + offset RIP positivo
        if let Some(caps) = re_rust_lea_pos.captures(line) {
            let lea_instruction_address_str = caps.get(1).unwrap().as_str();
            let lea_instruction_address =
                u64::from_str_radix(lea_instruction_address_str, 16).unwrap();

            let offset_str = caps.get(2).unwrap().as_str();
            let offset = u64::from_str_radix(offset_str, 16).unwrap();

            // Per l'istruzione lea, RIP punta all'istruzione successiva
            // La lunghezza tipica di "lea offset(%rip), %rdi" è 7 byte
            let rip_value = lea_instruction_address + 7;
            let main_address = rip_value + offset;

            println!(
                "Found main address via lea (positive): 0x{:x}",
                main_address
            );
            return Ok(main_address);
        }

        // Prova con lea + offset RIP negativo
        if let Some(caps) = re_rust_lea_neg.captures(line) {
            let lea_instruction_address_str = caps.get(1).unwrap().as_str();
            let lea_instruction_address =
                u64::from_str_radix(lea_instruction_address_str, 16).unwrap();

            let offset_str = caps.get(2).unwrap().as_str();
            let offset = u64::from_str_radix(offset_str, 16).unwrap();

            // Per offset negativo, sottraiamo dall'indirizzo RIP
            let rip_value = lea_instruction_address + 7;
            let main_address = rip_value - offset;

            println!(
                "Found main address via lea (negative): 0x{:x}",
                main_address
            );
            return Ok(main_address);
        }

        // Debug per linee che contengono rdi ma non matchano
        if line.contains("rdi") {
            println!("Line contains rdi but no match: {}", line);
        }
    }

    println!("No main address pattern found");
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
