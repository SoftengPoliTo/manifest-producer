use manifest_producer_backend::{
    analyse::analyse_functions,
    detect::function_detection,
    entry::find_root_nodes,
    inspect::{inspect_binary, parse_elf, read_elf},
};
use manifest_producer_frontend::html_generator::html_generator;

use crate::error::Result;

/// Performs a full analysis on a given ELF binary and generates output results.
///
/// # Arguments
///
/// - `elf_path`: A string slice containing the path to the ELF binary.
/// - `output_path`: A string slice specifying the directory where the analysis results will be saved.
///
/// # Workflow
///
/// 1. **Read ELF File**: Reads the binary file into memory using [`read_elf`].
/// 2. **Parse ELF Structure**: Parses the ELF binary into an internal representation with [`parse_elf`].
/// 3. **Inspect Metadata**: Extracts metadata and high-level details about the binary using [`inspect_binary`].
/// 4. **Detect Functions**: Identifies functions within the binary with [`function_detection`].
/// 5. **Analyze Functions**: Performs in-depth analysis of the identified functions using [`analyse_functions`].
/// 6. **Find Root Nodes**: Identifies key entry points in the binary with [`find_root_nodes`].
/// 7. **Generate HTML Report**: Produces an interactive HTML-based summary using [`html_generator`].
///
/// # Returns
///
/// - `Ok(())`: If the entire pipeline executes successfully.
/// - `Err(e)`: If any step in the pipeline fails, returns an error encapsulating the failure.
///
/// # Errors
///
/// Errors can occur due to:
/// - Issues reading the ELF file (e.g., file not found or inaccessible).
/// - Parsing failures due to invalid or corrupted ELF binaries.
/// - Analysis errors in downstream function calls.
/// - HTML generation failures.
#[allow(clippy::module_name_repetitions)]
pub fn perform_analysis(elf_path: &str, output_path: &str) -> Result<()> {
    let buffer = read_elf(elf_path)?;
    let elf = parse_elf(&buffer)?;

    let info = inspect_binary(&elf, elf_path, output_path)?;
    let mut detected_functions = function_detection(&elf, &info.language)?;
    analyse_functions(
        &elf,
        &buffer,
        &mut detected_functions,
        &info.language,
        output_path,
    )?;
    let root_nodes = find_root_nodes(elf_path, &info.language, &detected_functions)?;
    html_generator(&info, &detected_functions, &root_nodes, output_path)?;

    Ok(())
}
