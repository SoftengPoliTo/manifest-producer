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
///
/// # Example
/// ```
/// use behaviours_assessment::cli::parse_arguments;
///
/// let (elf_path, output_path) = parse_arguments().unwrap();
/// assert_eq!(output_path, "./public");
/// ```
pub fn parse_arguments() -> Result<(String, String)> {
    let matches = Command::new("cargo run")
        .arg(
            Arg::new("elf_path")
                .help("The path to the ELF binary to analyse")
                .required(true),
        )
        .try_get_matches()?;

    let elf_path = matches.get_one::<String>("elf_path").unwrap().clone();

    let output_path = "./public".to_string();

    Ok((elf_path, output_path))
}
