use backend::{
    api_analyzer::{analyze_functions, calculate_invocation_count, find_root_nodes},
    elf_analyzer::{extract_functions_lang, parse_elf, pre_analysis, read_elf},
    func_analyzer::extract_functions,
    error::Result,
};
use common::{FunctionNode, TreeNode};
use frontend::{
    html_generator::html_generator,
    tree_generator::{build_subtrees, build_tree, identify_subtrees},
};

use std::{collections::HashMap, env, process};

fn main() -> Result<()> {
    let elf_path = parse_args()?;
    let elf_data = read_elf(&elf_path)?;
    let elf = parse_elf(&elf_data)?;

    // Pre-analysis to gather general information
    let analysis_info = pre_analysis(&elf, &elf_path)?;

    // Detection and filtering of functions
    let detected_functions = extract_functions(&elf, &analysis_info.language)?;
    let filtered_functions = extract_functions_lang(&elf_path, &analysis_info.language)?;

    // Function analysis
    let (mut call_forest, disassembly) = analyze_functions(
        &elf,
        &elf_data,
        &detected_functions,
        &analysis_info.language,
    )?;

    calculate_invocation_count(&mut call_forest);
    let root_nodes = find_root_nodes(&call_forest, &filtered_functions)?;

    // HTML generation
    html_generator(
        analysis_info,
        detected_functions.len(),
        root_nodes.len(),
        &detected_functions,
        &call_forest,
        &disassembly,
        &root_nodes,
    )?;

    let mut node_roots: HashMap<String, FunctionNode> = HashMap::new();
    let mut sub_trees: HashMap<String, TreeNode> = HashMap::new();
    let mut id_counter = 0;

    for root in root_nodes {
        // Step 1: Identification of subtrees
        identify_subtrees(&root, &call_forest, &mut node_roots)?;

        // Step 2: Cleaning nodes with jmp equal to zero and creating subtrees
        build_subtrees(
            &mut node_roots,
            &call_forest,
            &mut sub_trees,
            &mut id_counter,
        );

        // Step 3: Construction of the tree
        build_tree(&root, &call_forest, &mut sub_trees, &mut id_counter)?;

        // Empty the structures for the next cycle
        node_roots.clear();
        sub_trees.clear();
    }

    // open_index_page()?; // Opens the browser with the generated output
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
