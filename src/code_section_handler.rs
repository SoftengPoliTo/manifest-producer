use std::collections::HashMap;

use goblin::elf::Elf;

use crate::{
    elf_utils::{cs_init, find_text_section, get_name_addr, API},
    error,
    plt_mapping::{find_plt_section, load_rela_plt_relocations},
};
use error::{Error, Result};

/// Extracts and disassembles code sections of APIs, handling static or dynamic linking.
///
/// This function extracts and disassembles the code of the specified API section, managing static or dynamic linking.
/// It returns the disassembled code and any system calls made by the API.
///
/// # Arguments
///
/// * `elf` - The ELF object representing the binary.
/// * `api` - The API structure containing information about the API section.
/// * `buffer` - The buffer containing the binary data of the ELF file.
/// * `link` - A boolean indicating whether static linking is used (`true`) or dynamic linking (`false`).
/// * `rust` - A boolean indicating whether the API section is written in Rust (`true`) or not (`false`).
///
/// # Returns
///
/// Returns a `Result` containing a vector of disassembled code strings and any system calls made by the API.
pub fn code_section(
    elf: &Elf,
    api: &API,
    buffer: &[u8],
    link: bool,
    rust: bool,
) -> Result<Vec<String>> {
    let text_section = find_text_section(elf).ok_or(Error::TextSectionNotFound)?;
    let code_slice: &[u8];
    let sys_call;

    if link {
        // Static linking
        let text_start_index = text_section.sh_offset as usize;
        let func_start_offset = (api.start_addr - text_section.sh_addr) as usize;
        let func_end_offset = (api.end_addr - text_section.sh_addr) as usize;
        code_slice =
            &buffer[text_start_index + func_start_offset..text_start_index + func_end_offset];

        println!("\n{:#x}\t<{}>", &api.start_addr, &api.name);
        sys_call = disassemble(elf, code_slice, api.start_addr, link, None, rust)?;
    } else {
        // Dynamic linking
        code_slice = &buffer[(api.start_addr) as usize..(api.end_addr) as usize];

        let mut found_plt_sec = false;
        let plt_section =
            find_plt_section(elf, &mut found_plt_sec).ok_or(Error::PLTSectionNotFound)?;
        let plt_entry_size = plt_section.sh_entsize as usize;
        let tbl = load_rela_plt_relocations(elf, plt_section, plt_entry_size, found_plt_sec);

        println!("\n{:#x}\t<{}>", &api.start_addr, &api.name);
        sys_call = disassemble(elf, code_slice, api.start_addr, link, tbl, rust)?;
    }

    Ok(sys_call)
}

// Disassembles the code in the specified section, handling static or dynamic function calls.
//
// This function disassembles the code in the specified section, handling static or dynamic function calls based on the given parameters.
// It returns the disassembled code and any system calls made by the API.
fn disassemble(
    elf: &Elf,
    code_slice: &[u8],
    addr: u64,
    link: bool,
    plt_map: Option<HashMap<u64, &str>>,
    rust: bool,
) -> Result<Vec<String>> {
    let cs = cs_init()?;
    let mut sys_call: Vec<String> = vec![];

    let instructions = cs.disasm_all(code_slice, addr).unwrap();
    for insn in instructions.iter() {
        let insn_addr = insn.address();
        let insn_name = cs.insn_name(insn.id()).unwrap();
        let op_str = insn.op_str().unwrap();

        if rust && insn_name == "lea" {
            if let Some(name) = lea_instruction(elf, op_str, insn_addr, insn_name.clone()) {
                sys_call.push(name);
            }
        } else if insn_name == "call" && !rust {
            if let Some(name) = call_instruction(
                elf,
                op_str,
                insn_addr,
                insn_name.clone(),
                link,
                plt_map.clone(),
            ) {
                sys_call.push(name);
            }
        } else {
            println!("0x{:x}:\t{}\t{}", insn_addr, insn_name, op_str);
        }
    }
    Ok(sys_call)
}

// Handles the instruction 'lea', identifies the function name, and adds any interface called by API.
fn lea_instruction<'a>(
    elf: &'a Elf<'a>,
    op_str: &'a str,
    insn_addr: u64,
    insn_name: String,
) -> Option<String> {
    if let Some(offset_str) = op_str.strip_suffix("(%rip), %rax") {
        if offset_str.starts_with('-') {
            if let Some(addr_str) = offset_str.strip_prefix("-0x") {
                if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                    let target_addr = insn_addr.wrapping_sub(addr);
                    let target_addr_aligned = target_addr + 7;
                    if let Some(name) = get_name_addr(elf, target_addr_aligned) {
                        println!("0x{:x}:\t{}\t<{}>", insn_addr, insn_name, name);
                        return Some(name.to_string());
                    }
                }
            }
        } else if let Some(addr_str) = offset_str.strip_prefix("0x") {
            if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                let target_addr = insn_addr.wrapping_add(addr);
                let target_addr_aligned = target_addr + 7;
                if let Some(name) = get_name_addr(elf, target_addr_aligned) {
                    println!("0x{:x}:\t{}\t<{}>", insn_addr, insn_name, name);
                    return Some(name.to_string());
                }
            }
        }
    }
    None
}

// Handles the instruction 'call', identifies the function name, and adds any interface called by API.
fn call_instruction<'a>(
    elf: &'a Elf<'a>,
    op_str: &'a str,
    address: u64,
    name_func: String,
    link: bool,
    plt_map: Option<HashMap<u64, &str>>,
) -> Option<String> {
    if let Some(addr_str) = op_str.strip_prefix("0x") {
        if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
            if link {
                // Statically linked
                if let Some(name) = get_name_addr(elf, addr) {
                    println!("0x{:x}:\t{}\t<{}>", address, name_func, name);
                    return Some(name.to_string());
                }
                let name = format!("CALL_to_<{}>", op_str);
                println!("0x{:x}:\t{}\t<{}>", address, name_func, name);
                return Some(name.to_string());
            }
            // Dynamically linked
            match plt_map {
                Some(map) => {
                    if let Some(plt_value) = map.get(&addr) {
                        println!("0x{:x}:\t{}\t<{}>", address, name_func, plt_value);
                        return Some(plt_value.to_string());
                    }
                    if let Some(name) = get_name_addr(elf, addr) {
                        println!("0x{:x}:\t{}\t<{}>", address, name_func, name);
                        return Some(name.to_string());
                    }
                }
                None => {
                    println!("PLT map is not available");
                }
            }
        } else {
            println!("Invalid address format: {}", op_str);
        }
    }
    None
}
