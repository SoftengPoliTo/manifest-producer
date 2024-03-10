use std::fs;
use std::path::Path;

use manifest_producer::api_detection::api_search;
use manifest_producer::cleanup::syscall_flow;
use manifest_producer::code_section_handler::code_section;
use manifest_producer::dwarf_analysis::dwarf_analysis;
use manifest_producer::elf_utils::{is_static, is_stripped, read_elf_file};
use manifest_producer::error::{Error, Result};
use manifest_producer::manifest_creation::{
    basic_info_manifest, feature_manifest, flow_call_manifest,
};

pub fn elf_analysis(file_path: &str, api_list: Vec<&str>, path: &str) -> Result<()> {
    let elf_data = read_elf_file(file_path)?;
    let elf = goblin::elf::Elf::parse(&elf_data)?;

    let stripped = is_stripped(&elf);
    if stripped {
        return Err(Error::DebugInfo);
    }

    let lang = match dwarf_analysis(file_path)?.strip_prefix("DW_LANG_") {
        Some(stripped_lang) => stripped_lang.to_owned(),
        None => "NOT_FOUND".to_string(), //return Err(Error::PrefixNotFound),
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

pub fn compare_manifest(snapshot_path: &Path, path: &Path) {
    let content = fs::read_to_string(path).unwrap();
    let name = path.file_name().and_then(|v| v.to_str());
    insta::with_settings!({
        snapshot_path => snapshot_path,
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(name, content);
    })
}
