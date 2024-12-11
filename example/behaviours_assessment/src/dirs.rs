use crate::error::Result;
use std::{fs, path::Path};

pub(crate) fn setup_output_dir(output_path: &str) -> Result<()> {
    let main_path = Path::new(output_path);
    if !main_path.exists() {
        fs::create_dir_all(main_path)?;
    }

    let json_path = main_path.join("json");
    if !json_path.exists() {
        fs::create_dir_all(&json_path)?;
    }

    let call_trees_path = main_path.join("call_trees");
    if !call_trees_path.exists() {
        fs::create_dir_all(&call_trees_path)?;
    }

    Ok(())
}
