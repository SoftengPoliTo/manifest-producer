use crate::backend::error::{Error, Result};

use gimli::{AttributeValue, DwarfSections, EndianSlice, RunTimeEndian};
use goblin::elf::{section_header::SHT_PROGBITS, Elf, SectionHeader};
use memmap2::Mmap;
use object::{Object, ObjectSection};

use std::{
    borrow::{self},
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Read,
    path::Path,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BasicInfo<'a> {
    pub file_name: &'a str,
    pub file_type: &'a str,
    pub file_size: u64,
    pub arch: &'a str,
    pub pie: bool,
    pub stripped: bool,
    pub static_linking: &'a str,
    pub language: String,
    pub entry_point: u64,
}

impl<'a> BasicInfo<'a> {
    pub fn new(file_name: &'a str, file_type: &'a str) -> Self {
        Self {
            file_name,
            file_type,
            file_size: 0,
            arch: "",
            pie: false,
            stripped: false,
            static_linking: "",
            language: String::new(),
            entry_point: 0,
        }
    }

    pub fn file_size(self, file_size: u64) -> Self {
        Self { file_size, ..self }
    }

    pub fn arch(self, arch: &'a str) -> Self {
        Self { arch, ..self }
    }

    pub fn pie(self, pie: bool) -> Self {
        Self { pie, ..self }
    }

    pub fn static_linking(self, static_linking: &'a str) -> Self {
        Self {
            static_linking,
            ..self
        }
    }

    pub fn language(self, language: String) -> Self {
        Self { language, ..self }
    }

    pub fn entry_point(self, entry_point: u64) -> Self {
        Self {
            entry_point,
            ..self
        }
    }

    pub fn stripped(self, stripped: bool) -> Self {
        Self { stripped, ..self }
    }
}

pub fn pre_analysis<'a>(elf: &'a Elf<'a>, elf_path: &'a str) -> Result<BasicInfo<'a>> {
    if is_stripped(elf) {
        return Err(Error::DebugInfo);
    }
    let file_name = get_name(elf_path)?;
    let file_type = get_file_type(elf)?;
    let link_type = if is_static(elf) {
        "Statically linked"
    } else {
        "Dynamically linked"
    };
    let arch = get_architecture(elf)?;
    let pie = is_pie(elf);
    let language = get_language(elf_path)?;
    let file_size = get_file_size(elf_path)?;
    let entry_point = get_entry_point(elf)?;

    let info = BasicInfo::new(file_name, file_type)
        .file_size(file_size)
        .arch(arch)
        .pie(pie)
        .static_linking(link_type)
        .language(language)
        .entry_point(entry_point);

    Ok(info)
}

pub fn read_elf(file_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn parse_elf(elf_data: &[u8]) -> Result<Elf> {
    Ok(Elf::parse(elf_data)?)
}

pub fn find_text_section<'a>(elf: &'a Elf<'a>) -> Option<&'a SectionHeader> {
    elf.section_headers.iter().find(|sec| {
        sec.sh_type == SHT_PROGBITS && {
            let name = elf.shdr_strtab.get_at(sec.sh_name);
            name == Some(".text")
        }
    })
}

pub fn get_name_addr<'a>(elf: &'a Elf<'a>, address: u64) -> Option<&'a str> {
    let symtab = &elf.syms;

    if let Some(sym) = symtab.iter().find(|sym| sym.st_value == address) {
        if let Some(name) = elf.strtab.get_at(sym.st_name) {
            return Some(name);
        }
    } else if let Some(txt_sec) = find_text_section(elf) {
        if let Some(name) = elf.strtab.get_at(txt_sec.sh_name) {
            return Some(name);
        }
    }
    None
}

pub fn filter_source_file(binary_path: &str, language: &str) -> Result<HashSet<String>> {
    let file = fs::File::open(binary_path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let object = object::File::parse(&*mmap)?;
    let endian = if object.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };

    let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>> {
        match object.section_by_name(id.name()) {
            Some(ref section) => Ok(section.uncompressed_data()?),
            None => Ok(borrow::Cow::Borrowed(&[][..])),
        }
    };

    let dwarf_sections = DwarfSections::load(&load_section)?;
    let borrow_section: &dyn for<'a> Fn(&'a borrow::Cow<[u8]>) -> EndianSlice<'a, RunTimeEndian> =
        &|section| EndianSlice::new(section, endian);
    let dwarf = dwarf_sections.borrow(&borrow_section);

    let mut iter = dwarf.units();
    let mut functions: HashSet<String> = HashSet::new();

    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;
        let mut entries = unit.entries();

        while let Some((_, entry)) = entries.next_dfs()? {
            if entry.tag() == gimli::DW_TAG_compile_unit {
                if let Some(path_attr) = entry.attr(gimli::DW_AT_name)? {
                    let file_name = match path_attr.value() {
                        AttributeValue::DebugStrRef(name_ref) => {
                            dwarf.string(name_ref)?.to_string_lossy().into_owned()
                        }
                        AttributeValue::DebugStrOffsetsIndex(index) => {
                            let index_ref = dwarf.string_offset(&unit, index)?;
                            dwarf.string(index_ref)?.to_string_lossy().into_owned()
                        }
                        _ => {
                            // Unsupported attribute value type
                            continue;
                        }
                    };

                    let valid_extension = match language {
                        "Rust" => file_name.contains(".rs"),
                        "C99" => file_name.contains(".c"),
                        "C_plus_plus_14" => file_name.contains(".cpp"),
                        _ => false,
                    };

                    if valid_extension {
                        let keywords = [
                            "musl", "libc", "std", "library", "core", ".cargo", "crypto", "ssl",
                            "compiler",
                        ];
                        if !keywords.iter().any(|&keyword| file_name.contains(keyword)) {
                            let mut sub_entries = unit.entries();
                            while let Some((_, sub_entry)) = sub_entries.next_dfs()? {
                                if sub_entry.tag() == gimli::DW_TAG_subprogram {
                                    if let Some(func_ref) = sub_entry.attr(gimli::DW_AT_name)? {
                                        if let Some(function_name) = match func_ref.value() {
                                            AttributeValue::DebugStrOffsetsIndex(func_name) => {
                                                let index_func_ref =
                                                    dwarf.string_offset(&unit, func_name)?;
                                                Some(
                                                    dwarf
                                                        .string(index_func_ref)?
                                                        .to_string_lossy()
                                                        .into_owned(),
                                                )
                                            }
                                            AttributeValue::DebugStrRef(func_name_ref) => Some(
                                                dwarf
                                                    .string(func_name_ref)?
                                                    .to_string_lossy()
                                                    .into_owned(),
                                            ),
                                            _ => None,
                                        } {
                                            if !functions.contains(&function_name) {
                                                functions.insert(function_name);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(functions)
}

fn get_file_size(elf_path: &str) -> Result<u64> {
    let path = Path::new(elf_path);
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

fn get_entry_point(elf: &Elf) -> Result<u64> {
    Ok(elf.header.e_entry)
}

fn get_name(elf_path: &str) -> Result<&str> {
    let path = Path::new(elf_path);
    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
        Ok(file_name)
    } else {
        Err(Error::InvalidFileName)
    }
}

fn is_stripped(elf: &Elf) -> bool {
    match elf.header.e_ident[goblin::elf::header::EI_CLASS] {
        goblin::elf::header::ELFCLASS64 | goblin::elf::header::ELFCLASS32 => {
            !has_sections(elf, goblin::elf::section_header::SHT_SYMTAB)
                || !has_sections(elf, goblin::elf::section_header::SHT_STRTAB)
        }
        _ => true,
    }
}

fn has_sections(elf: &Elf, section_type: u32) -> bool {
    elf.section_headers
        .iter()
        .any(|section| section.sh_type == section_type)
}

fn get_architecture<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_machine {
        goblin::elf::header::EM_X86_64 => Ok("x86_64"),
        goblin::elf::header::EM_ARM => Err(Error::InvalidElf {
            source: goblin::error::Error::Malformed(
                "ARM architecture currently not supported.".to_string(),
            ),
        }),
        _ => Err(Error::InvalidElf {
            source: goblin::error::Error::Malformed(
                "Architecture currently not supported.".to_string(),
            ),
        }),
    }
}

fn get_file_type<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_type {
        goblin::elf::header::ET_EXEC => Ok("Executable"),
        goblin::elf::header::ET_DYN => Ok("Dynamic Library"),
        _ => Err(Error::InvalidElf {
            source: goblin::error::Error::Malformed("Unknown File Type".to_string()),
        }),
    }
}

fn is_static(elf: &Elf) -> bool {
    for ph in &elf.program_headers {
        if ph.p_type == goblin::elf::program_header::PT_DYNAMIC {
            return false;
        }
    }
    true
}

fn is_pie(elf: &Elf) -> bool {
    if let Some(dynamic) = &elf.dynamic {
        dynamic.dyns.iter().any(|d| {
            d.d_tag == goblin::elf::dynamic::DT_FLAGS_1
                && d.d_val & goblin::elf::dynamic::DF_1_PIE != 0
        })
    } else {
        false
    }
}

fn get_language(elf_path: &str) -> Result<String> {
    let file = fs::File::open(elf_path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let object = object::File::parse(&*mmap)?;
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    let language = code_language(&object, endian)?;
    Ok(language)
}

fn code_language<'b>(object: &'b object::File<'b>, endian: gimli::RunTimeEndian) -> Result<String> {
    let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>> {
        match object.section_by_name(id.name()) {
            Some(ref section) => Ok(section
                .uncompressed_data()
                .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
            None => Ok(borrow::Cow::Borrowed(&[][..])),
        }
    };
    let dwarf_sections = DwarfSections::load(&load_section)?;
    let borrow_section: &dyn for<'a> Fn(&'a borrow::Cow<[u8]>) -> EndianSlice<'a, RunTimeEndian> =
        &|section| EndianSlice::new(section, endian);
    let dwarf = dwarf_sections.borrow(&borrow_section);
    let mut iter = dwarf.units();
    let mut language_counts = HashMap::new();

    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;
        let mut entries = unit.entries();

        while let Some((_, entry)) = entries.next_dfs()? {
            if let Some(language_attr) = entry.attr_value(gimli::DW_AT_language)? {
                let language = match language_attr {
                    gimli::AttributeValue::Language(language) => language,
                    _ => continue,
                };
                increment_language_count(&mut language_counts, &language.to_string());
            }
        }
    }
    let mut max_count = 0;
    let mut max_language = "".to_string();

    // The presence of C99 in the Rust program is due to the musl library, used to statically compile the binary
    if language_counts.contains_key("DW_LANG_C99") && language_counts.contains_key("DW_LANG_Rust") {
        language_counts.remove_entry("DW_LANG_C99");
    }
    for (language, count) in language_counts {
        if count > max_count {
            max_count = count;
            max_language.clone_from(&language);
        }
    }
    let lang = match max_language.strip_prefix("DW_LANG_") {
        Some(stripped_lang) => stripped_lang.to_owned(),
        None => return Err(Error::LangNotFound),
    };
    Ok(lang)
}

fn increment_language_count(map: &mut HashMap<String, u32>, language: &str) {
    let count = map.entry(language.to_string()).or_insert(0);
    *count += 1;
}
