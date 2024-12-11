// use insta::{assert_snapshot, with_settings};
// use manifest_producer_backend::{
//     analyse::analyse_functions,
//     detect::function_detection,
//     inspect::{inspect_binary, parse_elf, read_elf},
//     entry::find_root_nodes,
// };
// use serde_json::Value;
// use std::{collections::BTreeMap, env::temp_dir, fs, path::Path};

// #[test]
// fn test_c() {
//     setup_test_environment("test_c");
//     run_analysis_test("./tests/assets/minimal-fake-firmware-c-static", "test_c");
// }

// #[test]
// fn test_cpp() {
//     setup_test_environment("test_cpp");
//     run_analysis_test("./tests/assets/minimal-fake-firmware-cpp-static", "test_cpp");
// }

// #[test]
// fn test_rust() {
//     setup_test_environment("test_rust");
//     run_analysis_test("./tests/assets/fridge", "test_rust");
// }

// fn run_analysis_test(binary_path: &str, test_name: &str) {

//     let tmp = temp_dir().join(test_name);
//     let output_path = tmp.to_str().unwrap();

//     let buffer = read_elf(binary_path).unwrap();
//     let elf = parse_elf(&buffer).unwrap();

//     let info = inspect_binary(&elf, binary_path, output_path).unwrap();

//     let mut detected_functions = function_detection(&elf, &info.language).unwrap();
//     analyse_functions(
//         &elf,
//         &buffer,
//         &mut detected_functions,
//         &info.language,
//         output_path,
//     )
//     .unwrap();

//     let inspect_json_path = Path::new(output_path).join("json/basic_info.json");
//     let inspect_json = fs::read_to_string(inspect_json_path).unwrap();

//     let analyse_json_path = Path::new(output_path).join("json/functions_list.json");
//     let analyse_json = fs::read_to_string(analyse_json_path).unwrap();
//     let parsed_json: Value = serde_json::from_str(&analyse_json).unwrap();
//     let sorted_json = match parsed_json {
//         Value::Object(map) => {
//             let sorted_map: BTreeMap<_, _> = map.into_iter().collect();
//             serde_json::to_string_pretty(&sorted_map).unwrap()
//         }
//         _ => analyse_json,
//     };

//     let root_nodes = find_root_nodes(binary_path, &info.language, &detected_functions).unwrap();
//     let mut sorted_root_nodes = root_nodes.clone();
//     sorted_root_nodes.sort();
//     let root_nodes_json = serde_json::to_string_pretty(&sorted_root_nodes).unwrap();

//     with_settings!({
//         snapshot_path => format!("./snapshots/{}/", test_name),
//         prepend_module_to_snapshot => false,
//     },{
//         assert_snapshot!("basic_info", inspect_json);
//         assert_snapshot!("functions_list", sorted_json);
//         assert_snapshot!("root_nodes", root_nodes_json);
//     });
// }

// fn setup_test_environment(test_name: &str) {
//     let binding = temp_dir().join(test_name);
//     let main_path = Path::new(&binding);
//     if !main_path.exists() {
//         fs::create_dir_all(main_path).unwrap();
//     }

//     let json_path = main_path.join("json");
//     if !json_path.exists() {
//         fs::create_dir_all(&json_path).unwrap();
//     }
// }

use insta::{assert_snapshot, with_settings};
use manifest_producer_backend::{
    analyse::analyse_functions,
    detect::function_detection,
    entry::find_root_nodes,
    inspect::{inspect_binary, parse_elf, read_elf},
};
use serde_json::Value;
use std::{collections::BTreeMap, env::temp_dir, fs, path::Path};

#[test]
fn test_c() {
    run_analysis_test("./tests/assets/minimal-fake-firmware-c-static", "test_c");
}

#[test]
fn test_cpp() {
    run_analysis_test(
        "./tests/assets/minimal-fake-firmware-cpp-static",
        "test_cpp",
    );
}

#[test]
fn test_rust() {
    run_analysis_test("./tests/assets/fridge", "test_rust");
}

fn run_analysis_test(binary_path: &str, test_name: &str) {
    let output_path = setup_test_environment(test_name);

    let elf_buffer = read_elf(binary_path).unwrap();
    let elf = parse_elf(&elf_buffer).unwrap();

    let info = inspect_binary(&elf, binary_path, &output_path).unwrap();

    let mut detected_functions = function_detection(&elf, &info.language).unwrap();
    analyse_functions(
        &elf,
        &elf_buffer,
        &mut detected_functions,
        &info.language,
        &output_path,
    )
    .unwrap();

    let inspect_json = read_and_sort_json(Path::new(&output_path).join("json/basic_info.json"));
    let analyse_json = read_and_sort_json(Path::new(&output_path).join("json/functions_list.json"));

    let root_nodes = find_root_nodes(binary_path, &info.language, &detected_functions).unwrap();
    let mut sorted_root_nodes = root_nodes.clone();
    sorted_root_nodes.sort();
    let root_nodes_json = serde_json::to_string_pretty(&sorted_root_nodes).unwrap();

    with_settings!(
        {
            snapshot_path => format!("./snapshots/{}/", test_name),
            prepend_module_to_snapshot => false,
        },
        {
            assert_snapshot!("basic_info", inspect_json);
            assert_snapshot!("functions_list", analyse_json);
            assert_snapshot!("root_nodes", root_nodes_json);
        }
    );
}

fn setup_test_environment(test_name: &str) -> String {
    let test_dir = temp_dir().join(test_name);
    let json_dir = test_dir.join("json");

    for dir in [&test_dir, &json_dir] {
        if !dir.exists() {
            fs::create_dir_all(dir).unwrap();
        }
    }

    test_dir.to_str().unwrap().to_owned()
}

fn read_and_sort_json<P: AsRef<Path>>(path: P) -> String {
    let json_content = fs::read_to_string(&path).unwrap();
    let parsed_json: Value = serde_json::from_str(&json_content).unwrap();

    match parsed_json {
        Value::Object(map) => {
            let sorted_map: BTreeMap<_, _> = map.into_iter().collect();
            serde_json::to_string_pretty(&sorted_map).unwrap()
        }
        _ => json_content,
    }
}
