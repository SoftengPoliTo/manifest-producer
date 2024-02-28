use cpp_demangle::{DemangleOptions, Symbol};
use goblin::elf::{section_header::SHT_PROGBITS, Elf, SectionHeader};
use std::fs;
use std::{collections::HashMap, path::Path};
use std::{borrow, fs::File};
use std::io::{Read, Write};
use capstone::prelude::*;
use object::{Object, ObjectSection};


pub mod error;
use error::{Error, Result};

const CATEGORIES: [(&str, &[&str]); 9] = [
    ("File Manipulation", &["fwrite", "fopen", "fclose", "File", "write"]),
    ("Network Access", &["curl", "sendto", "recvfrom", "cpr"]),
    ("Device Access", &["__libc", "ioctl", "close"]),
    ("Audio Access", &["audio", "alcOpenDevice"]),
    ("Video Access", &["video", "capture", "Camera", "rscam"]),
    ("Memory Management", &["malloc", "calloc", "realloc"]),
    ("Data Encryption/Decryption", &["encrypt", "decrypt", "crypto"]),
    ("Data Compression/Decompression", &["compress", "decompress"]),
    ("Process Management", &["fork", "exec", "wait", "exit"]),
];

// Structure used to collect API data identified in the code.
struct API {
    name: String,
    start_addr: u64,
    end_addr: u64,
    syscalls: Vec<String>,
}

impl API {
    pub fn new(name: String, start_addr: u64, end_addr: u64) -> Self {
        Self {
            name,
            start_addr,
            end_addr,
            syscalls: Vec::new(),
        }
    }
    pub fn add_syscall(&mut self, syscall: String) {
        self.syscalls.push(syscall);
    }
}

pub fn elf_analysis(file_path: &str, api_list: Vec<&str>) -> Result<()>{

    let elf_data = read_elf_file(file_path)?;
    let elf = Elf::parse(&elf_data)?;

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
        syscall_flow(api, sys)?;
    }

    basic_info_manifest(&elf, file_path, &api_found, lang)?;
    flow_call_manifest(&api_found)?;
    feature_manifest(&api_found)?;

    Ok(())
}


/* 
*
*   Fourth step: creation of the three different types of JSON manifest.
*
*/

// Feature JSON manifest: categorizes APIs based on their functionality features.
fn feature_manifest(api_list: &Vec<API>) -> Result<()> {
    let mut categorized_features: HashMap<String, Vec<String>> = HashMap::new();

    for api in api_list {
        for syscall in &api.syscalls {
            // Check if the syscall contains one of the substrings associated with each category
            for (category, substrings) in &CATEGORIES {
                if substrings.iter().any(|&substring| syscall.contains(substring)) {
                    categorize_api(&mut categorized_features, &api.name, category);
                }
            }
        }
    }

    let mut features_json: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (api_name, features) in categorized_features {
        let features_array: Vec<serde_json::Value> = features.into_iter().map(|f| serde_json::Value::String(f)).collect();
        features_json.insert(api_name, serde_json::Value::Array(features_array));
    }

    let json_obj = serde_json::json!(features_json);
    let json_str = serde_json::to_string_pretty(&json_obj)?;

    let mut file = File::create("./manifest-produced/feature_manifest.json")?;
    file.write_all(json_str.as_bytes())?;

    Ok(())
}

// Helper function to categorize API under specific feature.
fn categorize_api(categorized_features: &mut HashMap<String, Vec<String>>, api_name: &str, feature: &str) {
    if let Some(feature_list) = categorized_features.get_mut(api_name) {
        if !feature_list.contains(&feature.to_string()) {
            feature_list.push(feature.to_string());
        }
    } else {
        let mut new_feature_list = Vec::new();
        new_feature_list.push(feature.to_string());
        categorized_features.insert(api_name.to_string(), new_feature_list);
    }
}

// FLOW CALL JSON: creates a JSON manifest that presents for each identified API, the list of function calls (system calls or subfunctions).
fn flow_call_manifest(api_list: &Vec<API>) -> Result<()> {
    let mut api_flow = Vec::new();

    for api in api_list {
        let mut api_info = serde_json::Map::new();
        let mut syscalls = Vec::new();
        
        for sys in &api.syscalls {
            // let name = demangle_function_name(sys)?;
            syscalls.push(serde_json::Value::String(sys.to_string()));
        }
    
        api_info.insert("name".to_string(), serde_json::Value::String(api.name.clone()));
        api_info.insert("syscalls".to_string(), serde_json::Value::Array(syscalls));
    
        api_flow.push(serde_json::Value::Object(api_info));
    }

    let json_obj = serde_json::json!({
        "Public APIs flow": api_flow
    });

    let json_str = serde_json::to_string_pretty(&json_obj)?;
    let mut file = File::create("./manifest-produced/flow_call.json")?;
    file.write_all(json_str.as_bytes())?;

    Ok(())

}

// Basic JSON manifest: prints general information about the elf binary and the identified public APIs.
fn basic_info_manifest(elf: &Elf, file_path: &str, api_list: &Vec<API>, language: String) -> Result<()>{
    let mut info = serde_json::Map::new();
    let file_name = Path::new(file_path).file_name().map_or(file_path, |f| f.to_str().unwrap());

    info.insert("file_name".to_string(), serde_json::Value::String(file_name.to_string()));
    info.insert("programming language".to_string(), serde_json::Value::String(language));
    info.insert("architecture".to_string(), serde_json::Value::String(get_arch(elf)?.to_owned()));
    info.insert("link".to_string(), serde_json::Value::String(if is_static(&elf) {"statically linked".to_string()} else {"dynamically linked".to_string()}));
    info.insert("file_type".to_string(), serde_json::Value::String(get_file_type(&elf)?.to_owned()));
    info.insert("endianness".to_string(), serde_json::Value::String(format!("{:?}", elf.header.endianness().unwrap())));
    info.insert("header_size".to_string(), serde_json::Value::Number(elf.header.e_ehsize.into()));
    info.insert("entry_point".to_string(), serde_json::Value::String(format!("{:#x}", elf.header.e_entry)));

    let list: Vec<serde_json::Value> = api_list.iter().map(|api| serde_json::Value::String(api.name.clone())).collect();
    info.insert("APIs found".to_string(), serde_json::Value::Array(list));

    let json_str = serde_json::to_string_pretty(&serde_json::Value::Object(info))?;
    let mut output_file = File::create("./manifest-produced/basic_info.json")?;
    output_file.write_all(json_str.as_bytes())?;

    Ok(())
}

// Attempt to clean up the mangled names
fn demangle_function_name(mangled_name: &str) -> Result<String> {
    if mangled_name.starts_with("_Z") {
        let options = DemangleOptions::default(); 
        let demangled_name = Symbol::new(mangled_name)?.demangle(&options)?; 
        Ok(demangled_name)
    } else {
        Ok(mangled_name.to_string())
    }
}

/* 
*
*   Third step: the code section is disassembled
*
*/

// Disassemble the code in the specified section, handling static or dynamic function calls.
fn disassemble(elf: &Elf, code_slice: &[u8], addr: u64, link: bool, plt_map: Option<HashMap<u64, &str>>, rust: bool) -> Result<Vec<String>> {
    let cs = cs_init()?;
    let mut sys_call: Vec<String> = vec![];

    let instructions = cs.disasm_all(code_slice, addr).unwrap();
    for insn in instructions.iter() {
        let insn_addr = insn.address();
        let insn_name = cs.insn_name(insn.id()).unwrap();
        let op_str = insn.op_str().unwrap();

        if rust && insn_name == "lea" {
            if let Some(name) = lea_instruction(elf, op_str, insn_addr, insn_name.clone()) {
                sys_call.push(name);
            }
        } else if insn_name == "call" && !rust {
            if let Some(name) = call_instruction(elf, op_str, insn_addr, insn_name.clone(), link, plt_map.clone()) {
                sys_call.push(name);
            }
        } else {
            println!("0x{:x}:\t{}\t{}", insn_addr, insn_name, op_str);
        }
    }
    Ok(sys_call)
}

// Handles the instruction 'lea', identifies the function name, and adds any interface called by API.
fn lea_instruction<'a>(elf: &'a Elf<'a>, op_str: &'a str, insn_addr: u64, insn_name: String) -> Option<String> {
    if let Some(offset_str) = op_str.strip_suffix("(%rip), %rax") {

        if offset_str.starts_with("-") {
            if let Some(addr_str) = offset_str.strip_prefix("-0x") {
                if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                    let target_addr = insn_addr.wrapping_sub(addr);
                    let target_addr_aligned = target_addr + 7;
                    if let Some(name) = get_name_addr(elf, target_addr_aligned) {
                        println!("0x{:x}:\t{}\t<{}>", insn_addr, insn_name, name);
                        return Some(name.to_string());
                    }
                }
            }
        } else {
            if let Some(addr_str) = offset_str.strip_prefix("0x") {
                if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
                    let target_addr = insn_addr.wrapping_add(addr);
                    let target_addr_aligned = target_addr + 7;
                    if let Some(name) = get_name_addr(elf, target_addr_aligned) {
                        println!("0x{:x}:\t{}\t<{}>", insn_addr, insn_name, name);
                        return Some(name.to_string());
                    }
                }
            }
        }

    }
    None
}

// Handles the instruction 'call', identifies the function name, and adds any interface called by API.
fn call_instruction<'a>(elf: &'a Elf<'a>, op_str: &'a str, address: u64, name_func: String, link: bool, plt_map: Option<HashMap<u64, &str>>) -> Option<String> {
    if let Some(addr_str) = op_str.strip_prefix("0x") {
        if let Ok(addr) = u64::from_str_radix(addr_str, 16) {
            if link {
                // statically linked
                if let Some(name) = get_name_addr(elf, addr) {
                    println!("0x{:x}:\t{}\t<{}>", address, name_func, name);
                    return Some(name.to_string());
                }
                let name = format!("CALL_to_<{}>", op_str);
                println!("0x{:x}:\t{}\t<{}>", address, name_func, name);
                return Some(name.to_string());
            }
            // dynamically linked
            match plt_map {
                Some(map) => {
                    if let Some(plt_value) = map.get(&addr) {
                        println!("0x{:x}:\t{}\t<{}>", address, name_func, plt_value);
                        return Some(plt_value.to_string());
                    }
                    if let Some(name) = get_name_addr(elf, addr) {
                        println!("0x{:x}:\t{}\t<{}>", address, name_func, name);
                        return Some(name.to_string());
                    }
                    let name = format!("CALL_to_<{}>", op_str);
                    println!("0x{:x}:\t{}\t<{}>", address, name_func, name);
                    return Some(name.to_string());
                }
                None => {
                    println!("PLT map is not available");
                }
            }
        } else {
            println!("Invalid address format: {}", op_str);
        }
    }
    None
}

// Initialize Capstone.
fn cs_init() -> Result<Capstone> {
    let cs = Capstone::new()
        .x86()
        .mode(arch::x86::ArchMode::Mode64)
        .syntax(arch::x86::ArchSyntax::Att)
        .detail(true)
        .build();
    cs.map_err(|err| {
        Error::Capstone(format!("Failed to create Capstone instance: {}", err))
    })
}

// Retrieve the name given the address.
fn get_name_addr<'a>(elf: &'a Elf<'a>, address: u64) -> Option<&'a str> {
    let symtab = &elf.syms;
    let dyntab = &elf.dynsyms;
    if let Some(sym) = symtab.iter().find(|sym| sym.st_value == address) {
        if let Some(name) = elf.strtab.get_at(sym.st_name) {
            return Some(name);
        }
    } 
    else if let Some(dsym) = dyntab.iter().find(|dsym| dsym.st_value == address) {
        if let Some(name) = elf.dynstrtab.get_at(dsym.st_name) {
            return Some(name);
        }
    }
    else if let Some(text_section) = find_text_section(elf) {
        if let Some(name) = elf.strtab.get_at(text_section.sh_name) {
            return Some(name);
        }
    }
    None
}

/* 
*
*   Second step: identify the sections of API code to analyze.
*
*/

// Extracts the code of the specified section, managing static or dynamic linking.
fn code_section(elf: &Elf, api: &API, buffer: &[u8], link: bool, rust: bool) -> Result<Vec<String>> {
    let text_section = find_text_section(elf).ok_or(Error::TextSectionNotFound)?;
    let code_slice:&[u8];
    let mut sys_call= vec![];

    if link {
        // Static linking
        let text_start_index = text_section.sh_offset as usize;
        let func_start_offset = (&api.start_addr - text_section.sh_addr) as usize ;
        let func_end_offset = (&api.end_addr - text_section.sh_addr) as usize ;
        code_slice = &buffer[text_start_index + func_start_offset
            ..text_start_index + func_end_offset];

        println!("\n{:#x}\t<{}>", &api.start_addr, &api.name);
        sys_call = disassemble(&elf, code_slice, api.start_addr, link, None, rust)?;

    } else {
        // Dynamic linking
        code_slice = &buffer[(api.start_addr) as usize
            ..(api.end_addr) as usize];
        let plt_section = find_plt_section(elf).ok_or(Error::PLTSectionNotFound)?;
        let plt_entry_size = plt_section.sh_entsize as usize;
        let tbl = load_rela_plt_relocations(elf, plt_section, plt_entry_size);

        println!("\n{:#x}\t<{}>", &api.start_addr, &api.name);
        sys_call = disassemble(&elf,code_slice, api.start_addr, link, tbl, rust)?;
    }
    
    Ok(sys_call)
}

// Encapsulate the call flow within the appropriate structure.
fn syscall_flow(api: &mut API, sys: Vec<String>) -> Result<()>{
    for s in sys {
        let name = demangle_function_name(&s)?;
        // if !name.starts_with("_") {
            api.add_syscall(name);
        // }
    }
    Ok(())
}

// Locate the .text section.
fn find_text_section<'a>(elf: &'a Elf<'a>) -> Option<&'a SectionHeader>{
    elf
        .section_headers
        .iter()
        .find(|sec| sec.sh_type == SHT_PROGBITS && {
            let name = elf.shdr_strtab.get_at(sec.sh_name);
            name == Some(".text")
        })
}


/*
*
*       First step: identify the functions declared by the developer.
*
*/

// Do an API lookup in the symbol table.
fn api_search<'a>(elf: &'a Elf<'a>, api_list: &'a Vec<&'a str>) -> Result<Vec<API>> {
    let mut api_found = Vec::new();
    for symbol in &elf.syms {
        if symbol.st_type() == goblin::elf::sym::STT_FUNC && symbol.st_shndx != 0 {
            if let Some(function_name) = get_name_sym(&elf, &symbol.to_owned()) {
                if api_list.contains(&function_name) {
                    api_found.push(API::new(function_name.to_string(), symbol.st_value, symbol.st_value+symbol.st_size));
                }
            }
        }
    }
    Ok(api_found)
}

// Retrieve the name given the symbol.
fn get_name_sym<'a>(elf: &'a Elf, symbol: &'a goblin::elf::Sym) -> Option<&'a str> {
    let name_offset = symbol.st_name as usize;
    let name_str: &'a str = &elf.strtab.get_at(name_offset)?;
    Some(name_str)
}


/*
*
*   Functions useful for retrieving basic information.
*
*/

// Read the contents of an ELF file.
fn read_elf_file(file_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(&file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// Check whether the specified ELF has been stripped of debug symbols.
fn is_stripped(elf: &Elf) -> bool {
    match elf.header.e_ident[goblin::elf::header::EI_CLASS] {
        goblin::elf::header::ELFCLASS64
        | goblin::elf::header::ELFCLASS32 => {
            !has_sections(&elf, goblin::elf::section_header::SHT_SYMTAB)
                || !has_sections(&elf, goblin::elf::section_header::SHT_STRTAB)
        }
        _ => true,
    }
}

fn has_sections(elf: &Elf, section_type: u32) -> bool {
    elf.section_headers.iter().any(|section| section.sh_type == section_type)
}

// Get the architecture type.
fn get_arch<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_machine {
        goblin::elf::header::EM_X86_64 =>  Ok("x86-64"),
        goblin::elf::header::EM_386 =>  Ok("x86"),
        _ =>  Err(Error::InvalidElf { source: goblin::error::Error::Malformed("Unknown Architecture".to_string())}),
    }
}

// Return the file type.
fn get_file_type<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_type {
        goblin::elf::header::ET_EXEC => Ok("Executable"),
        goblin::elf::header::ET_DYN => Ok("Dynamic Library"),
        goblin::elf::header::ET_CORE => Ok("File core"),
        _ => Err(Error::InvalidElf { source: goblin::error::Error::Malformed("Unknown File Type".to_string())}),
    }
}

// Check if the file is statically linked.
fn is_static(elf: &Elf) -> bool {
    if elf.dynamic.is_some() {
        false
    } else {
        true
    }
}

// Parse an ELF file to determine the programming language used. 
// Analysis example from: https://github.com/gimli-rs/gimli/blob/master/crates/examples/src/bin/simple.rs
fn dwarf_analysis(file_path: &str) -> Result<String>{
    let file = fs::File::open(&file_path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let object = object::File::parse(&*mmap)?;
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    let lang = analyze_elf_file(&object, endian)?;
    Ok(lang.to_string())
}

// Parse the dwarf format in the .debug_info section. Language attributes table available here: https://dwarfstd.org/languages.html
fn analyze_elf_file<'b>(object: &'b object::File<'b>, endian: gimli::RunTimeEndian) -> Result<&'b str> {
    let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>> {
        match object.section_by_name(id.name()) {
            Some(ref section) => Ok(section
                .uncompressed_data()
                .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
            None => Ok(borrow::Cow::Borrowed(&[][..])),
        }
    };
    let mut lang = "";
    let dwarf_cow = gimli::Dwarf::load(&load_section)?;
    let borrow_section: &dyn for<'a> Fn(
        &'a borrow::Cow<[u8]>,
    ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(&*section, endian);

    let dwarf = dwarf_cow.borrow(&borrow_section);
    let mut iter = dwarf.units();

    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;
        let mut entries = unit.entries();

        while let Some((_, entry)) = entries.next_dfs()? {
            if let Some(language_attr) = entry.attr_value(gimli::DW_AT_language)? {
                let language = match language_attr {
                    gimli::AttributeValue::Language(language) => language,
                    _ => continue,
                };
                match language.static_string() {
                    Some(name) => {
                        if lang.contains(name){
                            return Ok(lang);
                        }
                        lang = name;
                    },
                    None => {},
                }
            }
        }
    }
    Ok(lang)
}

/* 
*
*   Functions for mapping the .plt and .rela.plt sections in the case of dynamically linked elf binaries.
*
*/

// Load the .rela.plt relocations and return a map of the PLT entry addresses and their symbol names.
fn load_rela_plt_relocations<'a>(elf: &'a Elf<'a>, plt_section: &'a SectionHeader, plt_entry_size: usize) -> Option<HashMap<u64, &'a str>> {
    let mut tbl = HashMap::new();
    let mut i = 0;
    for (section_index, relocations) in &elf.shdr_relocs {
        if let Some(section_header) = elf.section_headers.get(*section_index as usize) {
            if let Some(section_name) = elf.shdr_strtab.get_at(section_header.sh_name) {
                if section_name == ".rela.plt" {
                    for r in relocations {
                        if let Some(symbol) = &elf.dynsyms.get(r.r_sym as usize) {
                            if let Some(name) = elf.dynstrtab.get_at(symbol.st_name) {
                                let plt_entry_index = i + 1;
                                let result = plt_entry_address(plt_section, plt_entry_index, plt_entry_size);
                                // println!("{:#x} --> {}", result, name);
                                tbl.insert(result, name);
                                i += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    Some(tbl)
}

fn plt_entry_address(plt_section: &SectionHeader, index: usize, plt_entry_size: usize) -> u64 {
    let offset = index * plt_entry_size;
    plt_section.sh_addr + offset as u64
}

// Find the .plt section.
fn find_plt_section<'a>(elf: &'a Elf<'a>) -> Option<&'a SectionHeader> {
    elf
        .section_headers
        .iter()
        .find(|sec| sec.sh_type == SHT_PROGBITS && {
            let name = elf.shdr_strtab.get_at(sec.sh_name);
            name == Some(".plt")
        })
}

