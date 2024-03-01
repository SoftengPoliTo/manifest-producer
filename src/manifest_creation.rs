use std::{collections::HashMap, fs::File, io::Write, path::Path};

use crate::{elf_utils, error};
use error::Result;
use goblin::elf::Elf;

use elf_utils::{get_arch, get_file_type, is_static, API};


/* 
*
*   Creation of JSON manifests.
*
*/

const CATEGORIES: [(&str, &[&str]); 9] = [
    ("File Manipulation", &["fwrite", "fopen", "fclose", "File", "write"]),
    ("Network Access", &["curl", "sendto", "recvfrom", "cpr"]),
    ("Device Access", &["__libc", "ioctl", "close"]),
    ("Audio Access", &["audio", "alcOpenDevice"]),
    ("Video Access", &["video", "capture", "Camera", "rscam"]),
    ("Memory Management", &["malloc", "calloc", "realloc"]),
    ("Data Encryption/Decryption", &["encrypt", "decrypt", "crypto"]),
    ("Data Compression/Decompression", &["compress", "decompress"]),
    ("Process Management", &["fork", "exec", "wait", "exit"]),
];

// Category JSON manifest: Categorizes APIs based on their functionality features.
pub fn feature_manifest(api_list: &Vec<API>) -> Result<()> {
    let mut categorized_features: HashMap<String, Vec<String>> = HashMap::new();

    for api in api_list {
        for syscall in &api.syscalls {
            // Check if the syscall contains one of the substrings associated with each category
            for (category, substrings) in &CATEGORIES {
                if substrings.iter().any(|&substring| syscall.contains(substring)) {
                    categorize_api(&mut categorized_features, &api.name, category);
                }
            }
        }
    }

    let mut features_json: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    for (api_name, features) in categorized_features {
        let features_array: Vec<serde_json::Value> = features.into_iter().map(|f| serde_json::Value::String(f)).collect();
        features_json.insert(api_name, serde_json::Value::Array(features_array));
    }

    let json_obj = serde_json::json!(features_json);
    let json_str = serde_json::to_string_pretty(&json_obj)?;

    let mut file = File::create("./manifest-produced/feature_manifest.json")?;
    file.write_all(json_str.as_bytes())?;

    Ok(())
}

// Helper function to categorize API under specific feature.
fn categorize_api(categorized_features: &mut HashMap<String, Vec<String>>, api_name: &str, feature: &str) {
    if let Some(feature_list) = categorized_features.get_mut(api_name) {
        if !feature_list.contains(&feature.to_string()) {
            feature_list.push(feature.to_string());
        }
    } else {
        let mut new_feature_list = Vec::new();
        new_feature_list.push(feature.to_string());
        categorized_features.insert(api_name.to_string(), new_feature_list);
    }
}

// Function call JSON manifest: Creates a JSON manifest that presents for each identified API, the list of function calls (system calls or subfunctions).
pub fn flow_call_manifest(api_list: &Vec<API>) -> Result<()> {
    let mut api_flow = Vec::new();

    for api in api_list {
        let mut api_info = serde_json::Map::new();
        let mut syscalls = Vec::new();
        
        for sys in &api.syscalls {
            syscalls.push(serde_json::Value::String(sys.to_string()));
        }
    
        api_info.insert("name".to_string(), serde_json::Value::String(api.name.clone()));
        api_info.insert("syscalls".to_string(), serde_json::Value::Array(syscalls));
    
        api_flow.push(serde_json::Value::Object(api_info));
    }

    let json_obj = serde_json::json!({
        "Public APIs flow": api_flow
    });

    let json_str = serde_json::to_string_pretty(&json_obj)?;
    let mut file = File::create("./manifest-produced/flow_call.json")?;
    file.write_all(json_str.as_bytes())?;

    Ok(())

}

// Basic JSON manifest: Prints general information about the ELF binary and the identified public APIs.
pub fn basic_info_manifest(elf: &Elf, file_path: &str, api_list: &Vec<API>, language: String) -> Result<()>{
    let mut info = serde_json::Map::new();
    let file_name = Path::new(file_path).file_name().map_or(file_path, |f| f.to_str().unwrap());

    info.insert("file_name".to_string(), serde_json::Value::String(file_name.to_string()));
    info.insert("programming language".to_string(), serde_json::Value::String(language));
    info.insert("architecture".to_string(), serde_json::Value::String(get_arch(elf)?.to_owned()));
    info.insert("link".to_string(), serde_json::Value::String(if is_static(&elf) {"statically linked".to_string()} else {"dynamically linked".to_string()}));
    info.insert("file_type".to_string(), serde_json::Value::String(get_file_type(&elf)?.to_owned()));
    info.insert("endianness".to_string(), serde_json::Value::String(format!("{:?}", elf.header.endianness().unwrap())));
    info.insert("header_size".to_string(), serde_json::Value::Number(elf.header.e_ehsize.into()));
    info.insert("entry_point".to_string(), serde_json::Value::String(format!("{:#x}", elf.header.e_entry)));

    let list: Vec<serde_json::Value> = api_list.iter().map(|api| serde_json::Value::String(api.name.clone())).collect();
    info.insert("APIs found".to_string(), serde_json::Value::Array(list));

    let json_str = serde_json::to_string_pretty(&serde_json::Value::Object(info))?;
    let mut output_file = File::create("./manifest-produced/basic_info.json")?;
    output_file.write_all(json_str.as_bytes())?;

    Ok(())
}
