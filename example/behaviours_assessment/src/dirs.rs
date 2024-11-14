use std::{fs, path::Path};
use crate::error::Result;

pub fn setup_output_dir(output_path: &str) -> Result<()> {
    let main_path = Path::new(output_path);
    if !main_path.exists() {
        fs::create_dir_all(main_path)?;
    }

    let json_path = main_path.join("json");
    if !json_path.exists() {
        fs::create_dir_all(&json_path)?;
    }

    let call_graphs_path = main_path.join("call_graphs");
    if !call_graphs_path.exists() {
        fs::create_dir_all(&call_graphs_path)?;
    }

    Ok(())
}
