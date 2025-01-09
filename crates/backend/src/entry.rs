use std::{
    borrow,
    collections::{HashMap, HashSet},
    fs,
};

use gimli::{AttributeValue, DwarfSections, EndianSlice, RunTimeEndian};
use memmap2::Mmap;
use object::{Object, ObjectSection};

use crate::{error::Result, FunctionNode};

#[cfg(feature = "progress_bar")]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(feature = "progress_bar")]
use std::time::Duration;

/// Identifies root nodes in a set of analysed functions.
///
/// # Overview
/// Root nodes are functions with no invocation entries but contain child functions and match specific filters.
/// If no such nodes are found, "main" is returned as the default root.
///
/// # Arguments
/// - `binary_path`: Path to the binary for filtering criteria.
/// - `language`: The programming language of the binary (used for demangling).
/// - `functions`: A map of function names to their [`FunctionNode`] representations.
///
/// # Returns
/// - A `Result` containing a vector of root function names as `Vec<String>`.
///
/// # Errors
/// - Propagates errors from `filtering` used for function selection.
///
/// # Feature Flags
/// - `progress_bar`: If enabled, displays a spinner indicating the possible root nodes detection.
///
pub fn find_root_nodes<S: ::std::hash::BuildHasher>(
    binary_path: &str,
    language: &str,
    functions: &HashMap<String, FunctionNode, S>,
) -> Result<Vec<String>> {
    #[cfg(feature = "progress_bar")]
    let pb = {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
        );
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message("Finding possible root nodes".to_string());
        pb
    };
    let filter = filtering_function(binary_path, language)?;
    let root_nodes: Vec<_> = functions
        .values()
        .filter_map(|func| {
            if func.invocation_entry == 0
                && filter.contains(&func.name)
                && !func.children.is_empty()
            {
                Some(func.name.clone())
            } else {
                None
            }
        })
        .collect();

    // TODO: Add a default root node if no root nodes are found
    #[cfg(feature = "progress_bar")]
    pb.finish_with_message(format!("Found {} possible root(s)", root_nodes.len()));

    if root_nodes.is_empty() {
        Ok(vec!["main".to_string()])
    } else {
        Ok(root_nodes)
    }
}

pub(crate) fn calculate_invocation_count(functions: &mut HashMap<String, FunctionNode>) {
    let nodes_to_update: Vec<_> = functions
        .values()
        .flat_map(|node| node.children.clone())
        .collect();

    for node_name in nodes_to_update {
        if let Some(func) = functions.get_mut(&node_name) {
            func.invocation_entry += 1;
        }
    }
}

fn filtering_function(binary_path: &str, language: &str) -> Result<HashSet<String>> {
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
                            "compiler", "lib",
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
