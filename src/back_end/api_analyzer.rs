use std::collections::{HashMap, HashSet};

use crate::back_end::{
    elf_analyzer::{find_text_section, get_name_addr},
    error,
    func_analyzer::{demangle_function_name, get_function, CallTree, FUNC},
};

use capstone::prelude::*;
use error::{Error, Result};
use goblin::elf::Elf;
use indicatif::{ProgressBar, ProgressStyle};

type FunctionCallTrees = HashMap<String, CallTree>;
type FunctionDisassemblies = Vec<(String, String)>;

/// Finds the root nodes in a given forest of call trees.
///
/// # Arguments
///
/// * `forest` - A mutable reference to a `HashMap` where the key is the name of the function (as a `String`)
///   and the value is a `CallTree` structure.
/// * `filter` - A `HashSet<String>` containing the names of functions to be considered as potential root nodes.
///
/// # Returns
///
/// A `Result` containing a vector of root node names (`Vec<String>`) if successful, or an error.
///
/// # Errors
///
/// This function may return an error if the invocation counting process fails.
pub fn find_root_nodes(
    forest: &mut HashMap<String, CallTree>,
    filter: &HashSet<String>,
) -> Result<Vec<String>> {
    invocation_number(forest)?;
    let mut roots = Vec::new();

    for (name, func) in forest.iter() {
        if func.invocation_count == 0 && filter.contains(name) {
            roots.push(name.clone())
        }
    }
    Ok(roots)
}

/// Updates the invocation counts for each node in the forest.
fn invocation_number(forest: &mut HashMap<String, CallTree>) -> Result<()> {
    let progress_bar = ProgressBar::new(forest.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.green}] {percent}\n{msg}")?
            .progress_chars("#>-"),
    );

    let mut updated_nodes = Vec::new();
    for (name, node) in forest.iter() {
        let nodes = node.nodes.clone();
        updated_nodes.push((name.clone(), nodes));
    }

    for (name, nodes) in updated_nodes {
        progress_bar.set_message(format!("Processing node: {}", name));
        for n in nodes {
            if let Some(tree) = forest.get_mut(&n) {
                tree.invocation_count += 1;
            }
        }
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Invocation counts updated.");
    Ok(())
}

/// Analyzes the functions in the given ELF file and disassembles them.
///
/// # Arguments
/// * `elf` - A reference to the ELF file to analyze.
/// * `buffer` - A byte slice of the ELF file's content.
/// * `functions` - A list of functions (`FUNC`) to disassemble.
/// * `language` - The programming language to assist with symbol demangling.
///
/// # Returns
/// A tuple containing:
/// * `HashMap<String, CallTree>` - A map of function names to their call trees.
/// * `HashMap<String, String>` - A map of function names to their disassembled code.
pub fn analyze_functions(
    elf: &Elf,
    buffer: &[u8],
    functions: &[FUNC],
    language: &str,
) -> Result<(FunctionCallTrees, FunctionDisassemblies)> {
    let progress_bar = ProgressBar::new(functions.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.green}] {percent}\n{msg}")?
            .progress_chars("#>-"),
    );

    let mut visited = HashMap::new();
    let mut disassembly_vec = vec![];

    for func in functions.iter() {
        progress_bar.set_message(format!(
            "Disassembling the following function: {}",
            func.name
        ));
        let (tree, disassembly) =
            disassemble_function(elf, func, buffer, functions.to_owned(), language)?;
        if tree.nodes.is_empty() {
            continue;
        } else {
            visited.insert(tree.name.clone(), tree);
            let function_name = normalize_function_name(&func.name);
            disassembly_vec.push((function_name, disassembly));
        }
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Disassembling phase completed.");

    Ok((visited, disassembly_vec))
}

fn normalize_function_name(function_name: &str) -> String {
    function_name
        .replace("::", "_")
        .replace(":", "_")
        .replace(" ", "_")
}

/// Disassembles a single function from the ELF file.
fn disassemble_function(
    elf: &Elf,
    func: &FUNC,
    buffer: &[u8],
    functions: Vec<FUNC>,
    language: &str,
) -> Result<(CallTree, String)> {
    let start_address = func.start_address;
    let (tree, disassembly) =
        analyze_code_slice(elf, buffer, func, start_address, &functions, language).unwrap();

    Ok((tree, disassembly))
}

/// Analyzes a slice of code and generates a call tree.
fn analyze_code_slice(
    elf: &Elf,
    buffer: &[u8],
    function: &FUNC,
    start_address: u64,
    functions: &[FUNC],
    language: &str,
) -> Result<(CallTree, String)> {
    let cs = cs_init()?;
    let mut nodes: CallTree = CallTree::new(function.name.clone());
    let code_slice = init_disassembly(elf, function, buffer)?;
    let mut disassembly_output = String::new();

    // Skip analysis if code_slice is empty
    if code_slice.is_empty() {
        return Ok((nodes, disassembly_output));
    }

    let instruction = cs.disasm_all(code_slice, start_address).unwrap();

    for insn in instruction.iter() {
        let insn_name = cs.insn_name(insn.id()).unwrap();
        let op_str = insn.op_str().unwrap();

        if insn_name == "call" {
            // Focusing on call instruction for potential syscall identification
            let called_func_name = call_insn(elf, op_str, language);
            if let Some(func_name) = called_func_name {
                get_function(functions, &func_name);
                if func_name.contains("_Unwind_") {
                    // TODO: Complete the end of the recursive traverse including the incorrect call function (in short, where we have to compute GOT offset)
                    disassembly_output.push_str(&format!(
                        "0x{:x}:\t{}\t{}   <{}>\n",
                        insn.address(),
                        insn_name,
                        op_str,
                        func_name
                    ));
                    continue;
                } else {
                    disassembly_output.push_str(&format!(
                        "0x{:x}:\t{}\t{}   <{}>\n",
                        insn.address(),
                        insn_name,
                        op_str,
                        func_name
                    ));
                    if !nodes.nodes.contains(&func_name) {
                        nodes.add_node(func_name);
                    }
                }
            } else {
                disassembly_output.push_str(&format!(
                    "0x{:x}:\t{}\t{}\t\t(Register Offset-GOT)\n",
                    insn.address(),
                    insn_name,
                    op_str
                ));
            }
        } else {
            // Store the disassembled instruction in the disassembly_output
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

/// Extracts the name of the function called in a call instruction.
fn call_insn<'a>(elf: &'a Elf<'a>, op_str: &'a str, language: &str) -> Option<String> {
    if let Some(addr_str) = op_str.strip_prefix("0x") {
        if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
            if let Some(name) = get_name_addr(elf, addr) {
                let demangle_name = demangle_function_name(name, language).unwrap();
                return Some(demangle_name.to_string());
            }
        }
    }
    None
}

/// Initializes the disassembly process by extracting the code slice for the given function.
fn init_disassembly<'a>(elf: &'a Elf<'a>, api: &'a FUNC, buffer: &'a [u8]) -> Result<&'a [u8]> {
    let text_section = find_text_section(elf).ok_or(Error::TextSectionNotFound)?;
    let text_start_index = text_section.sh_offset as usize;

    // Check if the API start address is within the text section
    if api.start_address > text_section.sh_addr {
        let func_start_offset = (api.start_address - text_section.sh_addr) as usize;
        let func_end_offset = (api.end_address - text_section.sh_addr) as usize;
        let code_slice =
            &buffer[text_start_index + func_start_offset..text_start_index + func_end_offset];

        Ok(code_slice)
    } else {
        // Skip this API by returning an empty slice
        Ok(&[])
    }
}

/// Initializes a Capstone disassembler for x86_64 architecture with AT&T syntax.
fn cs_init() -> Result<Capstone> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build();
    cs.map_err(|err| Error::Capstone(format!("Failed to create Capstone instance: {}", err)))
}
