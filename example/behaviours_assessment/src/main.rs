mod cli;
mod dirs;
mod analysis;
mod error;

use error::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let (elf_path, output_path) = cli::parse_arguments()?; 
    dirs::setup_output_dir(&output_path)?;                
    analysis::perform_analysis(&elf_path, &output_path)?;  
    Ok(())
}
