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
pub fn parse_arguments() -> (String, String, Option<usize>) {
    let matches = Command::new("behaviours-assessment")
        .version("0.1.0")
        .author("Giuseppe Marco Bianco <giuseppe.bianco1@uniurb.it>")
        .about("Inspect the behaviours of a binary")
        .arg(
            Arg::new("elf_path")
                .help("The path to the ELF binary to analyse")
                .required(true)
                .value_name("ELF_PATH"),
        )
        .arg(
            Arg::new("depth")
                .help("An optional positive number for call graph depth")
                .value_name("DEPTH")
                .num_args(1)
                .value_parser(clap::value_parser!(usize)),
        )
        .get_matches();

    let elf_path = matches.get_one::<String>("elf_path").unwrap().to_string();
    let name = elf_path.split('/').next_back().unwrap();
    // let output_path = format!("./examples/results/{name}");
    let exe_path = std::env::current_exe().unwrap();
    let exe_dir = exe_path.parent().unwrap();
    let mut output_path = exe_dir.to_path_buf();
    output_path.push("results");
    output_path.push(name);

    let depth = matches.get_one::<usize>("depth").copied();

    (elf_path, output_path.to_string_lossy().into_owned(), depth)
}
