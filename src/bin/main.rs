use manifest_producer::back_end::{
    api_analyzer::{analyze_functions, find_root_nodes},
    elf_analyzer::{filter_source_file, parse_elf, pre_analysis, read_elf},
    func_analyzer::functions_detection,
    error::Result,
};

use std::{env, process::exit};

fn main() -> Result<()> {
    let elf_path = parse_args()?;
    let elf_data = read_elf(&elf_path)?;
    let elf = parse_elf(&elf_data)?;

    let info = pre_analysis(&elf, &elf_path)?;

    let functions = functions_detection(&elf, &info.language)?;
    let filtered_function = filter_source_file(&elf_path, &info.language)?;

    let (mut forest, _disassembly) =
        analyze_functions(&elf, &elf_data, &functions, &info.language)?;
    let _roots = find_root_nodes(&mut forest, &filtered_function)?;

    Ok(())
}

fn parse_args() -> Result<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <ELF_path>", args[0]);
        exit(-1);
    }
    Ok(args[1].clone())
}