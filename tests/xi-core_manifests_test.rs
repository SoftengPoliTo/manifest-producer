mod common;

use std::{env::temp_dir, fs::create_dir_all, path::Path};

use common::{compare_manifest, elf_analysis};

const SNAPSHOT_PATH_DYN: &str = "../snapshots/xi-core/";

#[test]
fn test_xi_core() {
    let api_list = vec![
        "get_flags",
    ];

    let tmp_dir = temp_dir();
    let folder = tmp_dir.join("xi-core");
    let path = folder.to_str().unwrap();
    create_dir_all(path).unwrap();

    elf_analysis("./tests/elf_file/xi-core", api_list, path).unwrap();

    let basic_path = format!("{}/basic_info.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&basic_path));

    let flow_path = format!("{}/flow_call.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&flow_path));

    let feature_path = format!("{}/feature_manifest.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&feature_path));
}
