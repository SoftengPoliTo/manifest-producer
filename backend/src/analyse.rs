use crate::{
    detect::demangle_function_name,
    entry::calculate_invocation_count,
    error::{Error, Result},
    inspect::{find_text_section, get_name_addr},
    FunctionNode,
};

use capstone::{
    arch::{self, BuildsCapstone, BuildsCapstoneSyntax},
    Capstone,
};
use goblin::elf::Elf;
use indicatif::{ProgressBar, ProgressStyle};

use std::{collections::HashMap, fs::File};

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

    let progress_bar = ProgressBar::new(functions.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.green}] {percent}%\n{msg}")?
            .progress_chars("#>-"),
    );

    // I know, it's not elegant. At the moment I have no alternative but to clone, because
    // otherwise rust would not allow me to borrow functions as mutable more than once.
    let mut func_clone = functions.clone();
    for func in functions.values_mut() {
        progress_bar.set_message(format!("Disassembling function: {}", func.name));
        let (nodes, disassembly) =
            disassemble_function(elf, func, buffer, &mut func_clone, language)?;
        func.children = nodes;
        func.set_disassembly(disassembly);
        progress_bar.inc(1);
    }
    calculate_invocation_count(functions);

    let file = File::create(format!("{}json/functions_list.json", output_path))?;
    serde_json::to_writer_pretty(file, &functions)?;

    progress_bar.finish_with_message("All the functions in the .text section have been analysed.");
    Ok(())
}

fn disassemble_function(
    elf: &Elf,
    func: &FunctionNode,
    buffer: &[u8],
    functions: &mut HashMap<String, FunctionNode>,
    language: &str,
) -> Result<(Vec<String>, String)> {
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
) -> Result<(Vec<String>, String)> {
    let cs = cs_init()?;
    let code_slice = init_disassembly(elf, function, buffer)?;

    let mut nodes = Vec::new();
    let mut disassembly_output = String::new();

    // Skip if there is no code to disassemble
    if code_slice.is_empty() {
        return Ok((nodes, String::new())); 
    }

    let instruction = cs.disasm_all(code_slice, start_address)?;

    for insn in instruction.iter() {
        let insn_name = cs.insn_name(insn.id()).unwrap_or_default();
        let op_str = insn.op_str().unwrap_or_default();

        if insn_name == "call" {
            let called_func_name = call_insn(elf, op_str, language);
            if let Some(func_name) = called_func_name {
                if functions.contains_key(&func_name) {
                    if !nodes.contains(&func_name) {
                        nodes.push(func_name.clone());
                    }
                }
                disassembly_output.push_str(&format!(
                    "0x{:x}:\t{}\t{}   <{}>\n",
                    insn.address(),
                    insn_name,
                    op_str,
                    func_name
                ));
            } else {
                disassembly_output.push_str(&format!(
                    "0x{:x}:\t{}\t{}\t\t(Register Offset-GOT)\n",
                    insn.address(),
                    insn_name,
                    op_str
                ));
            }
        } else {
            disassembly_output.push_str(&format!(
                "0x{:x}:\t{}\t{}\n",
                insn.address(),
                insn_name,
                op_str
            ));
        }
    }

    Ok((nodes, disassembly_output))
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
    let text_start_index = text_section.sh_offset as usize;

    if api.start_addr > text_section.sh_addr {
        let func_start_offset = (api.start_addr - text_section.sh_addr) as usize;
        let func_end_offset = (api.end_addr - text_section.sh_addr) as usize;
        let code_slice =
            &buffer[text_start_index + func_start_offset..text_start_index + func_end_offset];
        Ok(code_slice)
    } else {
        Ok(&[])
    }
}

fn cs_init() -> Result<Capstone> {
    Ok(Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build()?)
}
