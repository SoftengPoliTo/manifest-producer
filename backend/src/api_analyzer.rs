use common::{
    capstone::{
        arch::{self, BuildsCapstone, BuildsCapstoneSyntax},
        Capstone,
    },
    error::{Error, Result},
    goblin::elf::Elf,
    indicatif::{ProgressBar, ProgressStyle},
    CallTree, FUNC,
};

use std::collections::{HashMap, HashSet};

use crate::{
    elf_analyzer::{find_text_section, get_name_addr},
    func_analyzer::{demangle_function_name, get_function},
};

type FunctionCallTrees = HashMap<String, CallTree>;
type FunctionDisassemblies = Vec<(String, String)>;

pub fn find_root_nodes(
    forest: &FunctionCallTrees,
    filter: &HashSet<String>,
) -> Result<Vec<String>> {
    let root_nodes: Vec<_> = forest
        .iter()
        .filter_map(|(name, func)| {
            if func.invocation_count == 0 && filter.contains(name) {
                Some(name.clone())
            } else {
                None
            }
        })
        .collect();

    if root_nodes.is_empty() {
        Ok(vec!["main".to_string()])
    } else {
        Ok(root_nodes)
    }
}

pub fn calculate_invocation_count(forest: &mut FunctionCallTrees) {
    let nodes_to_update: Vec<_> = forest
        .values()
        .flat_map(|node| node.nodes.clone())
        .collect();

    for node_name in nodes_to_update {
        if let Some(tree) = forest.get_mut(&node_name) {
            tree.invocation_count += 1;
        }
    }
}

pub fn analyze_functions(
    elf: &Elf,
    buffer: &[u8],
    functions: &[FUNC],
    language: &str,
) -> Result<(FunctionCallTrees, FunctionDisassemblies)> {
    if functions.is_empty() {
        return Ok((FunctionCallTrees::new(), FunctionDisassemblies::new()));
    }

    let progress_bar = ProgressBar::new(functions.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar:.green}] {percent}%\n{msg}")?
            .progress_chars("#>-"),
    );

    let mut visited = FunctionCallTrees::new();
    let mut disassembly_vec = FunctionDisassemblies::with_capacity(functions.len());

    for func in functions.iter() {
        progress_bar.set_message(format!("Disassembling function: {}", func.name));
        let (tree, disassembly) = disassemble_function(elf, func, buffer, functions, language)?;
        if !tree.nodes.is_empty() {
            visited.insert(tree.name.clone(), tree);
            disassembly_vec.push((normalize_function_name(&func.name), disassembly));
        }
        progress_bar.inc(1);
    }
    progress_bar.finish_with_message("Disassembling phase completed.");
    Ok((visited, disassembly_vec))
}

fn normalize_function_name(function_name: &str) -> String {
    function_name.replace([':', ' '], "_")
}

fn disassemble_function(
    elf: &Elf,
    func: &FUNC,
    buffer: &[u8],
    functions: &[FUNC],
    language: &str,
) -> Result<(CallTree, String)> {
    let start_address = func.start_address;
    analyze_code_slice(elf, buffer, func, start_address, functions, language)
}

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

    if code_slice.is_empty() {
        return Ok((nodes, String::new())); // Skip if there is no code to disassemble
    }

    let instruction = cs.disasm_all(code_slice, start_address)?;
    let mut disassembly_output = String::new();

    for insn in instruction.iter() {
        let insn_name = cs.insn_name(insn.id()).unwrap_or_default();
        let op_str = insn.op_str().unwrap_or_default();

        if insn_name == "call" {
            let called_func_name = call_insn(elf, op_str, language);
            if let Some(func_name) = called_func_name {
                get_function(functions, &func_name);
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

fn init_disassembly<'a>(elf: &'a Elf, api: &'a FUNC, buffer: &'a [u8]) -> Result<&'a [u8]> {
    let text_section = find_text_section(elf).ok_or(Error::TextSectionNotFound)?;
    let text_start_index = text_section.sh_offset as usize;

    if api.start_address > text_section.sh_addr {
        let func_start_offset = (api.start_address - text_section.sh_addr) as usize;
        let func_end_offset = (api.end_address - text_section.sh_addr) as usize;
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
