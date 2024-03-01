use std::env;
use manifest_producer::manifest_creation::{basic_info_manifest, feature_manifest, flow_call_manifest};
use manifest_producer::cleanup::syscall_flow;
use manifest_producer::code_section_handler::code_section;
use manifest_producer::error::{Error, Result};
use manifest_producer::api_detection::api_search;
use manifest_producer::dwarf_analysis::dwarf_analysis;
use manifest_producer::elf_utils::{is_static, is_stripped, read_elf_file};


pub fn elf_analysis(file_path: &str, api_list: Vec<&str>) -> Result<()> {
    let elf_data = read_elf_file(file_path)?;
    let elf = goblin::elf::Elf::parse(&elf_data)?;

    let stripped = is_stripped(&elf);
    if stripped {
        return Err(Error::DebugInfo);
    }

    let lang = match dwarf_analysis(file_path)?.strip_prefix("DW_LANG_") {
        Some(stripped_lang) => stripped_lang.to_owned(),
        None => return Err(Error::PrefixNotFound),
    };

    let link = is_static(&elf);

    let mut api_found = api_search(&elf, &api_list)?;
    if api_found.is_empty() {
        return Err(Error::APIListEmpty);
    } 

    for api in &mut api_found {
        let sys = code_section(&elf, api, &elf_data, link, if lang.contains("Rust") {true} else {false})?;
        syscall_flow(api, sys, &lang)?;
    }

    basic_info_manifest(&elf, file_path, &api_found, lang)?;
    flow_call_manifest(&api_found)?;
    feature_manifest(&api_found)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <elf_file_path>", args[0]);
        return;
    }
    let elf_file_path = &args[1];

    let api_list = vec![
        "writeOnDrive", 
        "turnLampOn", 
        "accessAudioDriver", 
        "turnLampOff", 
        "accessNetwork", 
        "accessWebcam", 
        "write_on_drive",
        "access_network",
        "access_webcam",
    ];

    match elf_analysis(elf_file_path, api_list) {
        Ok(_) => println!("Analysis performed successfully!"),
        Err(error) => eprintln!("Elf analysis failed: {}", error)
    };
}
