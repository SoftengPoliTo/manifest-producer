use manifest_producer::api_detection::api_search;
use manifest_producer::cleanup::syscall_flow;
use manifest_producer::code_section_handler::code_section;
use manifest_producer::dwarf_analysis::dwarf_analysis;
use manifest_producer::elf_utils::{is_static, is_stripped, read_elf_file};
use manifest_producer::error::{Error, Result};
use manifest_producer::manifest_creation::{
    basic_info_manifest, feature_manifest, flow_call_manifest,
};
use serde_json::Value;
use std::{env, fs};

/// Perform ELF analysis including API detection, system call flow encapsulation, and manifest generation.
///
/// This function performs analysis on an ELF file, including API detection, system call flow encapsulation, and manifest generation.
///
/// # Arguments
///
/// * `file_path` - The path to the ELF file to be analyzed.
/// * `api_list` - A vector containing the names of the APIs to search for.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure of the ELF analysis.
pub fn elf_analysis(file_path: &str, api_list: Vec<&str>, path: &str) -> Result<()> {
    let elf_data = read_elf_file(file_path)?;
    let elf = goblin::elf::Elf::parse(&elf_data)?;

    let stripped = is_stripped(&elf);
    if stripped {
        return Err(Error::DebugInfo);
    }

    let lang = match dwarf_analysis(file_path)?.strip_prefix("DW_LANG_") {
        Some(stripped_lang) => stripped_lang.to_owned(),
        None => "".to_string(), //return Err(Error::PrefixNotFound),
    };

    let link = is_static(&elf);

    let mut api_found = api_search(&elf, &api_list)?;
    if api_found.is_empty() {
        return Err(Error::APIListEmpty);
    }

    for api in &mut api_found {
        let sys = code_section(&elf, api, &elf_data, link, lang.contains("Rust"))?;
        syscall_flow(api, sys, &lang)?;
    }

    basic_info_manifest(&elf, file_path, &api_found, lang, path)?;
    flow_call_manifest(&api_found, path)?;
    feature_manifest(&api_found, path)?;

    Ok(())
}

fn read_api_list(json_file_path: &str) -> Result<Vec<String>> {
    let contents = fs::read_to_string(json_file_path)?;
    let json: Value = serde_json::from_str(&contents)?;
    let api_list: Vec<String> = serde_json::from_value(json)?;
    Ok(api_list)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <ELF_file_path> <JSON_file_path>", args[0]);
        return;
    }
    let elf_file_path = &args[1];
    let json_file_path = &args[2];

    let api_list = match read_api_list(json_file_path) {
        Ok(list) => list,
        Err(error) => {
            eprintln!("Error reading API list from JSON file: {}", error);
            return;
        }
    };
    let api_list_refs: Vec<&str> = api_list.iter().map(|s| s.as_str()).collect();

    let manifest_path = "./manifest-produced";

    match elf_analysis(elf_file_path, api_list_refs, manifest_path) {
        Ok(_) => println!("Analysis performed successfully!"),
        Err(error) => eprintln!("Elf analysis failed: {}", error),
    };
}
