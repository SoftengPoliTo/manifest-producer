use std::{fs::File, io::Read, path::Path};

use crate::{
    error::{Error, Result},
    BasicInfo,
};

// use gimli::{DwarfSections, EndianSlice, RunTimeEndian};
use goblin::{
    self,
    elf::{Elf, SectionHeader},
};
#[cfg(feature = "progress_bar")]
use indicatif::{ProgressBar, ProgressStyle};
use object::{self, elf::SHT_PROGBITS /*, Object, ObjectSection*/};
#[cfg(feature = "progress_bar")]
use std::time::Duration;

/// Analyses an ELF binary file, extracts basic information, and saves it in JSON format and in a `BasicInfo` structure.
///
/// # Overview
///
/// This function is a key entry point for binary inspection. It collects details about the file,
/// such as size, architecture, entry point, and whether it is PIE-enabled or statically linked. Results are saved in JSON.
///
/// # Arguments
///
/// - `elf`: A reference to an [`Elf`] structure (see [`parse_elf`]) representing the parsed ELF binary.
/// - `elf_path`: The file path of the ELF binary.
/// - `output_path`: Directory path where the JSON file with extracted data is saved.
///
/// # Returns
///
/// - A `Result` containing a [`BasicInfo`] structure with details about the binary.
///
/// # Errors
///
/// - Returns [`Error::DebugInfo`] if the ELF is stripped of debug information.
/// - Propagates errors related to file I/O or parsing.
///
/// # Feature Flags
///
/// - `progress_bar`: If enabled, displays a spinner indicating the binary inspection.
///
/// # See also
///
/// - [`read_elf`]: Reads the ELF binary into memory.
/// - [`parse_elf`]: Parses binary data into an `Elf` structure.
#[allow(clippy::module_name_repetitions)]
pub fn inspect_binary<'a>(
    elf: &'a Elf<'a>,
    elf_path: &'a str,
    output_path: &'a str,
) -> Result<BasicInfo<'a>> {
    #[cfg(feature = "progress_bar")]
    let pb = {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Inspection of the binary".to_string());
        pb
    };

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
    // let language = get_language(elf_path)?;
    let language = "Rust".to_string(); // Hardcoded to "Rust" cause ascot is written in Rust and we don't have to support other languages.
    let file_size = get_file_size(elf_path)?;
    let entry_point = get_entry_point(elf);

    let info = BasicInfo::new(file_name, file_type)
        .file_size(file_size)
        .arch(arch)
        .pie(pie)
        .static_linking(link_type)
        .language(language)
        .entry_point(entry_point);

    let file = File::create(format!("{output_path}/json/basic_info.json"))?;
    serde_json::to_writer_pretty(file, &info)?;

    #[cfg(feature = "progress_bar")]
    pb.finish_with_message("Inspection completed!");

    Ok(info)
}

/// Reads the contents of an ELF file into a byte buffer.
///
/// # Arguments
///
/// - `file_path`: The file path to the ELF binary.
///
/// # Returns
///
/// - A `Result` containing a byte vector (`Vec<u8>`) with the ELF file contents.
///
/// # Errors
///
/// - Returns standard I/O errors (e.g., file not found or permission issues).
///
/// # See also
///
/// - [`parse_elf`]: Processes the returned byte vector to parse the ELF binary.
pub fn read_elf(file_path: &str) -> Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Parses an ELF binary from raw bytes.
///
/// # Arguments
///
/// - `elf_data`: A reference to a byte array (`&[u8]`) containing ELF binary data.
///
/// # Returns
///
/// - A `Result` containing an [`Elf`] structure.
///
/// # Errors
///
/// - Returns parsing errors from the [`goblin`] library.
///
/// # See also
///
/// - [`read_elf`]: Reads ELF file contents into memory.
pub fn parse_elf(elf_data: &[u8]) -> Result<Elf> {
    Ok(Elf::parse(elf_data)?)
}

pub(crate) fn find_text_section<'a>(elf: &'a Elf<'a>) -> Option<&'a SectionHeader> {
    elf.section_headers.iter().find(|sec| {
        sec.sh_type == SHT_PROGBITS && {
            let name = elf.shdr_strtab.get_at(sec.sh_name);
            name == Some(".text")
        }
    })
}

pub(crate) fn get_name_addr<'a>(elf: &'a Elf<'a>, address: u64) -> Option<&'a str> {
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

fn get_file_size(elf_path: &str) -> Result<u64> {
    let path = Path::new(elf_path);
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

fn get_entry_point(elf: &Elf) -> u64 {
    elf.header.e_entry
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

fn get_architecture<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_machine {
        goblin::elf::header::EM_X86_64 => Ok("x86_64"),
        goblin::elf::header::EM_ARM => Err(Error::InvalidFormat(goblin::error::Error::Malformed(
            "ARM architecture currently not supported.".to_string(),
        ))),
        _ => Err(Error::InvalidFormat(goblin::error::Error::Malformed(
            "Architecture currently not supported.".to_string(),
        ))),
    }
}

fn get_file_type<'a>(elf: &'a Elf<'a>) -> Result<&'a str> {
    match elf.header.e_type {
        goblin::elf::header::ET_EXEC => Ok("Executable"),
        goblin::elf::header::ET_DYN => Ok("Dynamic Library"),
        _ => Err(Error::InvalidFormat(goblin::error::Error::Malformed(
            "Unknown File Type".to_string(),
        ))),
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

// fn get_language(elf_path: &str) -> Result<String> {
//     let file = fs::File::open(elf_path)?;
//     let mmap = unsafe { memmap2::Mmap::map(&file)? };
//     let object = object::File::parse(&*mmap)?;
//     let endian = if object.is_little_endian() {
//         gimli::RunTimeEndian::Little
//     } else {
//         gimli::RunTimeEndian::Big
//     };

//     let language = code_language(&object, endian)?;
//     Ok(language)
// }

// fn code_language<'b>(object: &'b object::File<'b>, endian: gimli::RunTimeEndian) -> Result<String> {
//     let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>> {
//         match object.section_by_name(id.name()) {
//             Some(ref section) => Ok(section
//                 .uncompressed_data()
//                 .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
//             None => Ok(borrow::Cow::Borrowed(&[][..])),
//         }
//     };
//     let dwarf_sections = DwarfSections::load(&load_section)?;
//     let borrow_section: &dyn for<'a> Fn(&'a borrow::Cow<[u8]>) -> EndianSlice<'a, RunTimeEndian> =
//         &|section| EndianSlice::new(section, endian);
//     let dwarf = dwarf_sections.borrow(&borrow_section);
//     let mut iter = dwarf.units();
//     let mut language_counts = HashMap::new();

//     while let Some(header) = iter.next()? {
//         let unit = dwarf.unit(header)?;
//         let mut entries = unit.entries();

//         while let Some((_, entry)) = entries.next_dfs()? {
//             if let Some(language_attr) = entry.attr_value(gimli::DW_AT_language)? {
//                 let gimli::AttributeValue::Language(language) = language_attr else {
//                     continue;
//                 };
//                 increment_language_count(&mut language_counts, &language.to_string());
//             }
//         }
//     }
//     let mut max_count = 0;
//     let mut max_language = String::new();

//     // The presence of C99 in the Rust program is due to the musl library, used to statically compile the binary
//     if language_counts.contains_key("DW_LANG_C99") && language_counts.contains_key("DW_LANG_Rust") {
//         language_counts.remove_entry("DW_LANG_C99");
//     }
//     for (language, count) in language_counts {
//         if count > max_count {
//             max_count = count;
//             max_language.clone_from(&language);
//         }
//     }
//     let lang = match max_language.strip_prefix("DW_LANG_") {
//         Some(stripped_lang) => stripped_lang.to_owned(),
//         None => return Err(Error::LangNotFound),
//     };
//     Ok(lang)
// }

// fn increment_language_count(map: &mut HashMap<String, u32>, language: &str) {
//     let count = map.entry(language.to_string()).or_insert(0);
//     *count += 1;
// }

fn has_sections(elf: &Elf, section_type: u32) -> bool {
    elf.section_headers
        .iter()
        .any(|section| section.sh_type == section_type)
}
