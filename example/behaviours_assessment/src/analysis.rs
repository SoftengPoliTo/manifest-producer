use manifest_producer_backend::{
    analyse::analyse_functions,
    detect::function_detection,
    entry::find_root_nodes,
    inspect::{inspect_binary, parse_elf, read_elf},
};
use manifest_producer_frontend::html_generator::html_generator;

use crate::error::Result;

pub fn perform_analysis(elf_path: &str, output_path: &str) -> Result<()> {
    let buffer = read_elf(elf_path)?;
    let elf = parse_elf(&buffer)?;

    let info = inspect_binary(&elf, elf_path, output_path)?;

    let mut detected_functions = function_detection(&elf, &info.language)?;

    analyse_functions(&elf, &buffer, &mut detected_functions, &info.language, output_path)?;

    let root_nodes = find_root_nodes(elf_path, &info.language, &detected_functions)?;

    html_generator(info, &detected_functions, &root_nodes, output_path)?;

    Ok(())
}
