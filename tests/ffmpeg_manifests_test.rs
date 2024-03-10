mod common;

use std::{env::temp_dir, fs::create_dir_all, path::Path};

use common::{compare_manifest, elf_analysis};

const SNAPSHOT_PATH_DYN: &str = "../snapshots/ffmpeg/";

#[test]
fn test_ffmpeg() {
    let api_list = vec![
        "check_filter_outputs",
        "init_complex_filtergraph",
        "fg_create",
        "fg_send_command",
        "enc_open",
        "of_write_trailer",
    ];

    let tmp_dir = temp_dir();
    let folder = tmp_dir.join("ffmpeg");
    let path = folder.to_str().unwrap();
    create_dir_all(path).unwrap();

    elf_analysis("./tests/elf_file/ffmpeg", api_list, path).unwrap();

    let basic_path = format!("{}/basic_info.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&basic_path));

    let flow_path = format!("{}/flow_call.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&flow_path));

    let feature_path = format!("{}/feature_manifest.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&feature_path));
}

