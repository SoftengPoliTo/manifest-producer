use insta::{assert_snapshot, with_settings};
use manifest_producer_backend::{
    analyse::analyse_functions,
    detect::function_detection,
    entry::find_root_nodes,
    inspect::{inspect_binary, parse_elf, read_elf},
};
use serde_json::Value;
use std::{collections::BTreeMap, env::temp_dir, fs, path::Path};

pub fn run_analysis_test(binary_path: &str, test_name: &str) {
    let output_path = setup_test_environment(test_name);

    let elf_buffer = read_elf(binary_path).unwrap();
    let elf = parse_elf(&elf_buffer).unwrap();

    let info = inspect_binary(&elf, binary_path, &output_path).unwrap();

    let mut detected_functions = function_detection(&elf, &info.language, &output_path).unwrap();
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
            snapshot_path => format!("../snapshots/{}/", test_name),
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
