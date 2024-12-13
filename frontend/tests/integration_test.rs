use std::{collections::HashMap, env::temp_dir, fs, path::Path};

use manifest_producer_backend::{BasicInfo, FunctionNode};
use manifest_producer_frontend::html_generator::html_generator;
use walkdir::WalkDir;

#[test]
fn run_frontend_test() {
    let info = BasicInfo::new("Test-Frontend", "Executable")
        .file_size(916_432)
        .arch("x86_64")
        .pie(false)
        .stripped(false)
        .static_linking("Statically linked")
        .language("Rust".to_string())
        .entry_point(4_199_776);

    let mut root = FunctionNode::new("root".to_string(), 4_499_220, 4_499_459);
    let mut first = FunctionNode::new("child1".to_string(), 4_445_450, 4_445_459);
    let mut second = FunctionNode::new("child2".to_string(), 4_445_460, 4_445_469);
    let mut third = FunctionNode::new("child11".to_string(), 4_499_240, 4_499_249);
    let mut fourth = FunctionNode::new("child12".to_string(), 4_445_480, 4_445_489);
    let mut fifth = FunctionNode::new("child21".to_string(), 4_455_460, 4_455_469);

    root.children.push(first.name.clone());
    root.children.push(second.name.clone());
    first.children.push(third.name.clone());
    first.children.push(fourth.name.clone());
    second.children.push(fifth.name.clone());

    let disass_root = "0x494340:\tendbr64\t\n0x494344:\tmov\t8(%rdx), %rcx\n0x494348:\tmov\t$1, %eax\n0x49434d:\tcmp\t%rcx, 8(%rsi)\n0x494351:\tja\t0x494355\n0x494353:\tsbb\t%eax, %eax\n0x494355:\tret\t\n".to_string();
    let disass_first = "0x44ff50:\tendbr64\t\n0x44ff54:\tmov\t$0xa, %eax\n0x44ff59:\tsyscall\t\n0x44ff5b:\tcmp\t$-0xfff, %rax\n0x44ff61:\tjae\t0x44ff64\n0x44ff63:\tret\t\n0x44ff64:\tmov\t$18446744073709551544, %rcx\n0x44ff6b:\tneg\t%eax\n0x44ff6d:\tmov\t%eax, %fs:(%rcx)\n0x44ff70:\tor\t$0xffffffffffffffff, %rax\n0x44ff74:\tret\t\n".to_string();
    let disass_second= "0x4368c0:\tendbr64\t\n0x4368c4:\tmov\t$18446744073709551528, %rax\n0x4368cb:\tmov\t%fs:(%rax), %rdx\n0x4368cf:\tnop\t(%rax, %rax)\n".to_string();
    let disass_third= "0x4378c0:\tendbr64\t\n0x4378c4:\tmov\t$18446744073709551528, %rax\n0x4378cb:\tmov\t%fs:(%rax), %rdx\n0x4378cf:\tnop\t(%rax, %rax)\n".to_string();
    let disass_fourth= "0x4388c0:\tendbr64\t\n0x4388c4:\tmov\t$18446744073709551528, %rax\n0x4388cb:\tmov\t%fs:(%rax), %rdx\n0x4388cf:\tnop\t(%rax, %rax)\n".to_string();
    let disass_fifth= "0x4398c0:\tendbr64\t\n0x4398c4:\tmov\t$18446744073709551528, %rax\n0x4398cb:\tmov\t%fs:(%rax), %rdx\n0x4398cf:\tnop\t(%rax, %rax)\n".to_string();

    root.set_disassembly(disass_root);
    first.set_disassembly(disass_first);
    second.set_disassembly(disass_second);
    third.set_disassembly(disass_third);
    fourth.set_disassembly(disass_fourth);
    fifth.set_disassembly(disass_fifth);

    let mut detected_functions: HashMap<String, FunctionNode> = HashMap::default();
    detected_functions.insert(root.name.clone(), root);
    detected_functions.insert(first.name.clone(), first);
    detected_functions.insert(second.name.clone(), second);
    detected_functions.insert(third.name.clone(), third);
    detected_functions.insert(fourth.name.clone(), fourth);
    detected_functions.insert(fifth.name.clone(), fifth);

    let root_nodes = vec!["root".to_string()];

    let output_path = setup_test_environment();

    html_generator(&info, &detected_functions, &root_nodes, &output_path).unwrap();

    let snapshot_path = Path::new("./snapshots");
    compare_generated_nodes(snapshot_path, Path::new(&output_path));
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

#[allow(clippy::semicolon_if_nothing_returned)]
fn compare_generated_nodes(snapshot_path: &Path, output_path: &Path) {
    for entry in WalkDir::new(output_path) {
        entry.map_or((), |e| {
            if e.path().is_file()
                && !e.path().components().any(|c| {
                    c.as_os_str() == "call_trees" || c.as_os_str() == "disassembly_view.html"
                })
            {
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
