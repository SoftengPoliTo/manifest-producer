use std::{collections::HashMap, env::temp_dir, fs, path::Path};

use manifest_producer_backend::{BasicInfo, FunctionNode};
use manifest_producer_frontend::html_generator::html_generator;
use walkdir::WalkDir;

#[test]
fn run_frontend_test() {
    let info = BasicInfo::new("Test-Frontend", "Executable")
        .file_size(916432)
        .arch("x86_64")
        .pie(false)
        .stripped(false)
        .static_linking("Statically linked")
        .language("Rust".to_string())
        .entry_point(4199776);

    let mut node1 = FunctionNode::new("root".to_string(), 4499220, 4499459);
    let mut node2 = FunctionNode::new("child1".to_string(), 4445450, 4445459);
    let mut node3 = FunctionNode::new("child2".to_string(), 4445460, 4445469);
    let mut node21 = FunctionNode::new("child11".to_string(), 4499240, 4499249);
    let mut node22 = FunctionNode::new("child12".to_string(), 4445480, 4445489);
    let mut node31 = FunctionNode::new("child21".to_string(), 4455460, 4455469);

    node1.children.push(node2.name.clone());
    node1.children.push(node3.name.clone());
    node2.children.push(node21.name.clone());
    node2.children.push(node22.name.clone());
    node3.children.push(node31.name.clone());

    let disassembly1 = "0x494340:\tendbr64\t\n0x494344:\tmov\t8(%rdx), %rcx\n0x494348:\tmov\t$1, %eax\n0x49434d:\tcmp\t%rcx, 8(%rsi)\n0x494351:\tja\t0x494355\n0x494353:\tsbb\t%eax, %eax\n0x494355:\tret\t\n".to_string();
    let disassembly2 = "0x44ff50:\tendbr64\t\n0x44ff54:\tmov\t$0xa, %eax\n0x44ff59:\tsyscall\t\n0x44ff5b:\tcmp\t$-0xfff, %rax\n0x44ff61:\tjae\t0x44ff64\n0x44ff63:\tret\t\n0x44ff64:\tmov\t$18446744073709551544, %rcx\n0x44ff6b:\tneg\t%eax\n0x44ff6d:\tmov\t%eax, %fs:(%rcx)\n0x44ff70:\tor\t$0xffffffffffffffff, %rax\n0x44ff74:\tret\t\n".to_string();
    let disassembly3= "0x4368c0:\tendbr64\t\n0x4368c4:\tmov\t$18446744073709551528, %rax\n0x4368cb:\tmov\t%fs:(%rax), %rdx\n0x4368cf:\tnop\t(%rax, %rax)\n".to_string();
    let disassembly21= "0x4368c0:\tendbr64\t\n0x4368c4:\tmov\t$18446744073709551528, %rax\n0x4368cb:\tmov\t%fs:(%rax), %rdx\n0x4368cf:\tnop\t(%rax, %rax)\n".to_string();
    let disassembly22= "0x4368c0:\tendbr64\t\n0x4368c4:\tmov\t$18446744073709551528, %rax\n0x4368cb:\tmov\t%fs:(%rax), %rdx\n0x4368cf:\tnop\t(%rax, %rax)\n".to_string();
    let disassembly31= "0x4368c0:\tendbr64\t\n0x4368c4:\tmov\t$18446744073709551528, %rax\n0x4368cb:\tmov\t%fs:(%rax), %rdx\n0x4368cf:\tnop\t(%rax, %rax)\n".to_string();

    node1.set_disassembly(disassembly1);
    node2.set_disassembly(disassembly2);
    node3.set_disassembly(disassembly3);
    node21.set_disassembly(disassembly21);
    node22.set_disassembly(disassembly22);
    node31.set_disassembly(disassembly31);

    let mut detected_functions: HashMap<String, FunctionNode> = Default::default();
    detected_functions.insert(node1.name.clone(), node1);
    detected_functions.insert(node2.name.clone(), node2);
    detected_functions.insert(node3.name.clone(), node3);
    detected_functions.insert(node21.name.clone(), node21);
    detected_functions.insert(node22.name.clone(), node22);
    detected_functions.insert(node31.name.clone(), node31);

    let root_nodes = vec!["root".to_string()];

    let output_path = setup_test_environment();

    html_generator(info, &detected_functions, &root_nodes, &output_path).unwrap();

    let snapshot_path = Path::new("./snapshots");
    compare_generated_html(snapshot_path, Path::new(&output_path));
}

fn setup_test_environment() -> String {
    let test_dir = temp_dir().join("frontend");
    let tree_dir = test_dir.join("call_trees");
    let json_dir = test_dir.join("json");

    for dir in [&test_dir, &tree_dir, &json_dir] {
        if !dir.exists() {
            fs::create_dir_all(dir).unwrap();
        }
    }

    test_dir.to_str().unwrap().to_owned()
}

fn compare_generated_html(snapshot_path: &Path, output_path: &Path) {
    for entry in WalkDir::new(output_path).into_iter() {
        entry.map_or((), |e| {
            if e.path().is_file() && !e.path().components().any(|c| c.as_os_str() == "call_trees") {
                compare(snapshot_path, output_path, e.path());
            }
        })
    }
}

fn compare(snapshot_path: &Path, output_path: &Path, entry: &Path) {
    let content = fs::read_to_string(entry).unwrap();
    let name = entry.file_name().and_then(|v| v.to_str()).unwrap();

    insta::with_settings!({
        snapshot_path => snapshot_path
        .join(entry.strip_prefix(output_path).unwrap())
        .parent()
        .unwrap(),
        prepend_module_to_snapshot => false,
    }, {
        insta::assert_snapshot!(name, content);
    });
}
