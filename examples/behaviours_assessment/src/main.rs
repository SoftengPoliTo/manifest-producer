mod analysis;
mod cli;
mod dirs;
mod error;

use error::Result;

/// Entry point for the behaviours assessment tool.
///
/// The `main` function coordinates the command-line parsing, directory setup, and
/// analysis process. It ensures that all errors are reported clearly and the program
/// exits gracefully if any step fails.
///
/// # Workflow
///
/// 1. Parses command-line arguments using [`cli::parse_arguments`].
/// 2. Sets up the output directory via [`dirs::setup_output_dir`].
/// 3. Invokes the analysis pipeline via [`analysis::perform_analysis`].
///
/// # Exit Codes
///
/// - `0`: Success.
/// - `1`: An error occurred during execution.
///
pub fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let (elf_path, output_path) = cli::parse_arguments()?;
    dirs::setup_output_dir(&output_path)?;
    analysis::perform_analysis(&elf_path, &output_path)?;

    Ok(())
}
