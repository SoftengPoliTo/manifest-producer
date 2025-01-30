use crate::error::Result;
use clap::{Arg, Command};

/// Parses command-line arguments for the behaviours assessment tool.
///
/// # Returns
///
/// - `Ok((elf_path, output_path))`: A tuple containing the path to the ELF binary and the output directory.
/// - `Err(e)`: If argument parsing fails, an error is returned.
///
/// # Arguments Parsed
///
/// - `elf_path` (required): Path to the ELF binary to be analyzed.
pub fn parse_arguments() -> Result<(String, String)> {
    let matches = Command::new("behaviours-assessment")
        .version("0.1.0")
        .author("Giuseppe Marco Bianco <giuseppe.bianco1@uniurb.it>")
        .about("Inspect the behaviours of a binary")
        .arg(
            Arg::new("elf_path")
                .help("The path to the ELF binary to analyse")
                .required(true),
        )
        .get_matches();
    let elf_path = matches.get_one::<String>("elf_path").unwrap().to_string();
    let name = elf_path.split("/").last().unwrap();
    let output_path = format!("./examples/results/{}", name);

    Ok((elf_path, output_path))
}
