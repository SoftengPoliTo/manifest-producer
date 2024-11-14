use clap::{Arg, Command};
use crate::error::Result;

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

    let elf_path = matches
        .get_one::<String>("elf_path")
        .unwrap()
        .clone();

    let default_output_path = "./public/".to_string();
    let output_path = matches
        .get_one::<String>("output_path")
        .map(|s| s.to_string())
        .unwrap_or(default_output_path);

    let output_path = if output_path.ends_with('/') {
        output_path
    } else {
        format!("{}/", output_path)
    };

    Ok((elf_path, output_path))
}
