use crate::error;
use error::Result;
use std::{fs, borrow};
use object::{Object, ObjectSection};

/// Parse an ELF file to determine the programming language used.
///
/// This function analyzes the Dwarf information in the ELF file to determine the programming language used.
///
/// # Arguments
///
/// * `file_path` - The path to the ELF file.
///
/// # Returns
///
/// Returns a `Result` containing the programming language used, if successfully determined.
/// Analysis example from: https://github.com/gimli-rs/gimli/blob/master/crates/examples/src/bin/simple.rs
pub fn dwarf_analysis(file_path: &str) -> Result<String>{
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
