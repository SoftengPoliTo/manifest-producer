use crate::error::Result;
use clap::{Arg, Command};

/// Parses command-line arguments for the behaviours assessment tool.
///
/// # Returns
/// - `Ok((elf_path, output_path))`: A tuple containing the path to the ELF binary and the output directory.
/// - `Err(e)`: If argument parsing fails, an error is returned.
///
/// # Arguments Parsed
/// - `elf_path` (required): Path to the ELF binary to be analyzed.
/// - `output_path` (optional): Directory where the analysis results should be saved.
///   - If not provided, defaults to `./public`.
///
/// # Example
/// ```
/// use behaviours_assessment::cli::parse_arguments;
///
/// let (elf_path, output_path) = parse_arguments().unwrap();
/// assert_eq!(output_path, "./public"); // Default value if not specified.
/// ```
pub fn parse_arguments() -> Result<(String, String)> {
    let matches = Command::new("cargo run")
        .arg(
            Arg::new("elf_path")
                .help("The path to the ELF binary to analyse")
                .required(true),
        )
        .arg(
            Arg::new("output_path")
                .short('o')
                .long("output")
                .help("Optional path to save the analysis results (default: ./public/*)"),
        )
        .try_get_matches()?;

    let elf_path = matches.get_one::<String>("elf_path").unwrap().clone();

    let default_output_path = "./public".to_string();
    let output_path = matches
        .get_one::<String>("output_path")
        .map_or(default_output_path, std::string::ToString::to_string);

    Ok((elf_path, output_path))
}
