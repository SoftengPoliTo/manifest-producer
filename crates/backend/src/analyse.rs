use crate::{
    detect::demangle_function_name,
    entry::calculate_invocation_count,
    error::{Error, Result},
    inspect::{find_text_section, get_name_addr},
    syscall::detect_syscalls,
    FunctionNode,
};

use capstone::{
    arch::{self, BuildsCapstone, BuildsCapstoneSyntax},
    Capstone,
};
use goblin::elf::Elf;

use std::fmt::Write;
use std::{collections::HashMap, fs::File};

/// Disassembles and analyses functions in an ELF binary.
///
/// # Overview
///
/// Iterates through all detected functions from [`crate::detect::function_detection`], disassembles their machine code,
/// and updates their [`FunctionNode`] structures with details like child functions and disassembly results.
/// Results are also saved as JSON.
///
/// # Arguments
///
/// - `elf`: A reference to an [`Elf`] structure.
/// - `buffer`: Byte buffer of the ELF binary.
/// - `functions`: A mutable `HashMap` of detected functions as `FunctionNode` objects.
/// - `language`: The programming language of the binary.
/// - `output_path`: Directory to save the JSON file with analysis results.
///
/// # Returns
///
/// - A `Result<()>` indicating success or failure.
///
/// # Errors
///
/// - Possible errors related to the disassembly of machine code.
///
/// # Feature Flags
///
/// - `progress_bar`: If enabled, displays a progress bar indicating the progress of the disassembly code.
#[allow(clippy::implicit_hasher)]
pub fn analyse_functions(
    elf: &Elf,
    buffer: &[u8],
    functions: &mut HashMap<String, FunctionNode>,
    language: &str,
    output_path: &str,
) -> Result<()> {
    if functions.is_empty() {
        return Ok(());
    }

    #[cfg(feature = "progress_bar")]
    let pb = {
        let pb = indicatif::ProgressBar::new(functions.len() as u64);
        pb.set_message("Machine code disassembly:".to_string());
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{msg}\n{wide_bar} {pos}/{len} [{elapsed_precise}]")?,
        );
        pb
    };

    // I know, it's not elegant. At the moment I have no alternative but to clone, because
    // otherwise rust would not allow me to borrow functions as mutable more than once.
    let mut func_clone = functions.clone();
    for func in functions.values_mut() {
        let (nodes, disassembly, flag) =
            disassemble_function(elf, func, buffer, &mut func_clone, language)?;
        func.children = nodes;
        func.set_disassembly(disassembly);
        func.syscall = flag;

        #[cfg(feature = "progress_bar")]
        pb.inc(1);
    }

    calculate_invocation_count(functions);
    detect_syscalls(functions)?;

    let file = File::create(format!("{output_path}/json/functions_list.json"))?;
    serde_json::to_writer_pretty(file, &functions)?;

    #[cfg(feature = "progress_bar")]
    pb.finish_with_message("Disassembly completed!");
    Ok(())
}

fn disassemble_function(
    elf: &Elf,
    func: &FunctionNode,
    buffer: &[u8],
    functions: &mut HashMap<String, FunctionNode>,
    language: &str,
) -> Result<(Vec<String>, String, bool)> {
    let start_address = func.start_addr;
    analyse_code_slice(elf, buffer, func, start_address, functions, language)
}

fn analyse_code_slice(
    elf: &Elf,
    buffer: &[u8],
    function: &FunctionNode,
    start_address: u64,
    functions: &mut HashMap<String, FunctionNode>,
    language: &str,
) -> Result<(Vec<String>, String, bool)> {
    let cs = cs_init()?;
    let code_slice = init_disassembly(elf, function, buffer)?;

    let mut nodes = Vec::new();
    let mut disassembly_output = String::new();
    let mut flag = false;

    // Skip if there is no code to disassemble
    if code_slice.is_empty() {
        return Ok((nodes, String::new(), flag));
    }

    let instruction = cs.disasm_all(code_slice, start_address)?;

    for insn in instruction.iter() {
        let insn_name = cs.insn_name(insn.id()).unwrap_or_default();
        let op_str = insn.op_str().unwrap_or_default();

        if insn_name == "call" {
            let called_func_name = call_insn(elf, op_str, language);
            if let Some(func_name) = called_func_name {
                if functions.contains_key(&func_name) && !nodes.contains(&func_name) {
                    nodes.push(func_name.clone());
                }
                writeln!(
                    disassembly_output,
                    "0x{:x}:\t{}\t{}\t<{}>\n",
                    insn.address(),
                    insn_name,
                    op_str,
                    func_name
                )?;
            } else {
                writeln!(
                    disassembly_output,
                    "0x{:x}:\t{}\t{}\t(Register Offset-GOT)\n",
                    insn.address(),
                    insn_name,
                    op_str
                )?;
            }
        } else if insn_name == "syscall" {
            flag = true;
            writeln!(
                disassembly_output,
                "0x{:x}:\t{}\t\t(System Call Invoked)\n",
                insn.address(),
                insn_name
            )?;
        } else {
            writeln!(
                disassembly_output,
                "0x{:x}:\t{}\t{}\n",
                insn.address(),
                insn_name,
                op_str
            )?;
        }
    }

    Ok((nodes, disassembly_output, flag))
}

fn call_insn(elf: &Elf, op_str: &str, language: &str) -> Option<String> {
    op_str
        .strip_prefix("0x")
        .and_then(|addr_str| u64::from_str_radix(addr_str, 16).ok())
        .and_then(|addr| get_name_addr(elf, addr))
        .and_then(|name| demangle_function_name(name, language).ok())
        .map(|name| name.to_string())
}

fn init_disassembly<'a>(elf: &'a Elf, api: &'a FunctionNode, buffer: &'a [u8]) -> Result<&'a [u8]> {
    let text_section = find_text_section(elf).ok_or(Error::TextSectionNotFound)?;
    let text_start_index = usize::try_from(text_section.sh_offset).unwrap(); // TODO: Try to remove unwrap()!

    if api.start_addr > text_section.sh_addr {
        let func_start_offset = usize::try_from(api.start_addr - text_section.sh_addr).unwrap(); // TODO: Try to remove unwrap()!
        let func_end_offset = usize::try_from(api.end_addr - text_section.sh_addr).unwrap(); // TODO: Try to remove unwrap()!
        let code_slice =
            &buffer[text_start_index + func_start_offset..text_start_index + func_end_offset];
        Ok(code_slice)
    } else {
        Ok(&[])
    }
}

fn cs_init() -> Result<Capstone> {
    Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()
        .map_err(Into::into)
}
