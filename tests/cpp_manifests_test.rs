mod common;

use std::{env::temp_dir, fs::create_dir_all, path::Path};

use common::{compare_manifest, elf_analysis};

const SNAPSHOT_PATH_DYN: &str = "../snapshots/cpp-dynamic/";
const SNAPSHOT_PATH_STATIC: &str = "../snapshots/cpp-static/";

#[test]
fn test_cpp_dynamic() {
    let api_list = vec![
        "writeOnDrive",
        "accessNetwork",
        "accessWebcam",
        "turnLampOn",
        "turnLampOff",
    ];

    let tmp_dir = temp_dir();
    let folder = tmp_dir.join("cpp-dynamic");
    let path = folder.to_str().unwrap();
    create_dir_all(path).unwrap();

    elf_analysis("./tests/elf_file/fake-firmware-cpp-dynamic", api_list, path).unwrap();

    let basic_path = format!("{}/basic_info.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&basic_path));

    let flow_path = format!("{}/flow_call.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&flow_path));

    let feature_path = format!("{}/feature_manifest.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_DYN), Path::new(&feature_path));
}

#[test]
fn test_cpp_static() {
    let api_list = vec![
        "writeOnDrive",
        "accessNetwork",
        "accessWebcam",
        "turnLampOn",
        "turnLampOff",
    ];

    let tmp_dir = temp_dir();
    let folder = tmp_dir.join("cpp-static");
    let path = folder.to_str().unwrap();
    create_dir_all(path).unwrap();

    elf_analysis(
        "./tests/elf_file/minimal-fake-firmware-cpp-static",
        api_list,
        path,
    )
    .unwrap();

    let basic_path = format!("{}/basic_info.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_STATIC), Path::new(&basic_path));

    let flow_path = format!("{}/flow_call.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_STATIC), Path::new(&flow_path));

    let feature_path = format!("{}/feature_manifest.json", path);
    compare_manifest(Path::new(SNAPSHOT_PATH_STATIC), Path::new(&feature_path));
}
