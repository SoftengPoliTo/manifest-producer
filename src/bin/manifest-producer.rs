use manifest_producer::{
    elf_analyzer::{parse_elf, pre_analysis, read_elf},
    error::Result,
};

use std::{env, process::exit};

fn main() -> Result<()> {
    let elf_path = parse_args()?;
    let elf_data = read_elf(&elf_path)?;
    let elf = parse_elf(&elf_data)?;

    let _info = pre_analysis(&elf, &elf_path)?;

    Ok(())
}

fn parse_args() -> Result<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <ELF_path>", args[0]);
        exit(-1);
    }
    Ok(args[1].clone())
}
