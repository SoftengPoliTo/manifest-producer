use manifest_producer_backend::{
    analyse::analyse_functions,
    detect::function_detection,
    entry::find_root_nodes,
    inspect::{inspect_binary, parse_elf, read_elf},
};
use manifest_producer_frontend::html_generator::html_generator;

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::error::Result;

/// Performs a full analysis on a given ELF binary and generates output results.
///
/// # Arguments
/// - `elf_path`: A string slice containing the path to the ELF binary.
/// - `output_path`: A string slice specifying the directory where the analysis results will be saved.
///
/// # Workflow
/// 1. **Read ELF File**: Reads the binary file into memory using [`read_elf`].
/// 2. **Parse ELF Structure**: Parses the ELF binary into an internal representation with [`parse_elf`].
/// 3. **Inspect Metadata**: Extracts metadata and high-level details about the binary using [`inspect_binary`].
/// 4. **Detect Functions**: Identifies functions within the binary with [`function_detection`].
/// 5. **Analyze Functions**: Performs in-depth analysis of the identified functions using [`analyse_functions`].
/// 6. **Find Root Nodes**: Identifies key entry points in the binary with [`find_root_nodes`].
/// 7. **Generate HTML Report**: Produces an interactive HTML-based summary using [`html_generator`].
///
/// # Returns
/// - `Ok(())`: If the entire pipeline executes successfully.
/// - `Err(e)`: If any step in the pipeline fails, returns an error encapsulating the failure.
///
/// # Errors
/// Errors can occur due to:
/// - Issues reading the ELF file (e.g., file not found or inaccessible).
/// - Parsing failures due to invalid or corrupted ELF binaries.
/// - Analysis errors in downstream function calls.
#[allow(clippy::module_name_repetitions)]
pub fn perform_analysis(elf_path: &str, output_path: &str) -> Result<()> {
    let buffer = read_elf(elf_path)?;
    let elf = parse_elf(&buffer)?;

    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
    );
    progress_bar.enable_steady_tick(Duration::from_millis(100));

    progress_bar.set_message("Inspection of the binary".to_string());
    let info = inspect_binary(&elf, elf_path, output_path)?;

    progress_bar.set_message("Detection of the functions".to_string());
    let mut detected_functions = function_detection(&elf, &info.language)?;

    progress_bar.set_message("Analysis of the functions".to_string());
    analyse_functions(
        &elf,
        &buffer,
        &mut detected_functions,
        &info.language,
        output_path,
    )?;

    progress_bar.set_message("Finding possible root nodes".to_string());
    let root_nodes = find_root_nodes(elf_path, &info.language, &detected_functions)?;

    progress_bar.set_message("Planting tree(s)".to_string());
    html_generator(&info, &detected_functions, &root_nodes, output_path)?;

    progress_bar.finish_with_message(format!(
        "manifest-producer completed the work succesfully! Results are exported in {output_path}"
    ));
    Ok(())
}
