use manifest_producer::{
    backend::{
        api_analyzer::{analyze_functions, find_root_nodes},
        elf_analyzer::{filter_source_file, parse_elf, pre_analysis, read_elf},
        error::Result,
        func_analyzer::functions_detection,
    },
    frontend::{
        html_generator::{html_generator, open_index_page},
        tree_generator::build_tree,
    },
};

use std::{env, process};

fn main() -> Result<()> {
    let elf_path = parse_args()?;
    let elf_data = read_elf(&elf_path)?;
    let elf = parse_elf(&elf_data)?;

    // Pre-analysis to gather general information
    let analysis_info = pre_analysis(&elf, &elf_path)?;

    // Detection and filtering of functions
    let detected_functions = functions_detection(&elf, &analysis_info.language)?;
    let filtered_functions = filter_source_file(&elf_path, &analysis_info.language)?;

    // Function analysis
    let (mut call_forest, disassembly) = analyze_functions(
        &elf,
        &elf_data,
        &detected_functions,
        &analysis_info.language,
    )?;
    let root_nodes = find_root_nodes(&mut call_forest, &filtered_functions)?;

    // Tree construction and HTML generation
    build_tree(&root_nodes, &call_forest)?;

    html_generator(
        analysis_info,
        detected_functions.len(),
        root_nodes.len(),
        detected_functions,
        call_forest,
        &disassembly,
        &root_nodes,
    )?;

    open_index_page()?; // Opens the browser with the generated output
    Ok(())
}

fn parse_args() -> Result<String> {
    let args: Vec<String> = env::args().collect();
    if let Some(elf_path) = args.get(1) {
        Ok(elf_path.clone())
    } else {
        eprintln!("Usage: <manifest-producer> <ELF_path>");
        process::exit(1)
    }
}
