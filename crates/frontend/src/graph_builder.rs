use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
};

use crate::{error::Result, TreeNode};
use manifest_producer_backend::FunctionNode;
use serde_json::to_string_pretty;

/// Constructs a function call graph and generates a JSON representation.
///
/// This function scans and processes detected functions to build a tree structure
/// representing their relationships. The resulting structure is then serialized
/// into a JSON file for visualization.
///
/// # Arguments
///
/// - `detected_functions`: A mutable reference to a map of function names to their [`FunctionNode`] objects.
/// - `root_function`: The entry point function name used as the root of the graph.
/// - `output_path`: The directory where the generated JSON file should be saved.
/// - `max_depth`: An optional depth limit for the function call graph.
///
/// # Workflow
///
/// 1. Scans and collects relevant function nodes.
/// 2. Removes unnecessary nodes.
/// 3. Builds a hierarchical tree representation.
/// 4. Serializes the tree structure into JSON.
///
/// # Returns
///
/// - `Ok(TreeNode)`: If the graph is successfully built and saved.
/// - `Err(e)`: If any errors occur during processing.
///
/// # Errors
///
/// Errors may arise from:
/// - Issues during function node scanning.
/// - Problems in constructing the tree structure.
/// - Failures in writing the JSON output file.
pub fn graph_builder<S: ::std::hash::BuildHasher>(
    detected_functions: &mut HashMap<String, FunctionNode, S>,
    root_function: &str,
    output_path: &str,
    max_depth: Option<usize>,
) -> Result<TreeNode> {
    let mut tree_nodes = HashSet::new();
    node_scanner(root_function, detected_functions, &mut tree_nodes);
    rm_useless_node(detected_functions, &mut tree_nodes);

    let mut active_stack = Vec::new();
    let mut id_counter = 0;
    let js_tree = build(
            root_function,
            &*detected_functions,
            &mut id_counter,
            &mut active_stack,
            0,
            max_depth.unwrap_or(15),
        );

    graph_json(&js_tree, root_function, output_path)?;

    Ok(js_tree)
}

fn node_scanner<S: ::std::hash::BuildHasher>(
    function_name: &str,
    detected_functions: &HashMap<String, FunctionNode, S>,
    tree_nodes: &mut HashSet<String>,
) {
    if tree_nodes.contains(function_name) {
        return;
    }

    if let Some(node) = detected_functions.get(function_name) {
        tree_nodes.insert(function_name.to_string());
        for child_name in &node.children {
            node_scanner(child_name, detected_functions, tree_nodes);
        }
    }
}

fn rm_useless_node<S: ::std::hash::BuildHasher>(
    detected_functions: &mut HashMap<String, FunctionNode, S>,
    tree_nodes: &mut HashSet<String>,
) {
    let keys: Vec<String> = detected_functions.keys().cloned().collect();
    for func in keys {
        if tree_nodes.contains(&func) {
            tree_nodes.remove(&func);
        } else {
            detected_functions.remove(&func);
        }
    }
}

fn build<S: ::std::hash::BuildHasher>(
    function_name: &str,
    detected_functions: &HashMap<String, FunctionNode, S>,
    id_counter: &mut usize,
    active_stack: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
) -> TreeNode {
    if depth >= max_depth || active_stack.contains(&function_name.to_string()) {
        let node = TreeNode::new(*id_counter, function_name);
        *id_counter += 1;
        return node;
    }

    active_stack.push(function_name.to_string());
    let mut node = TreeNode::new(*id_counter, function_name);
    *id_counter += 1;

    if let Some(call_tree) = detected_functions.get(function_name) {
        for child_name in &call_tree.children {
            let child_node = build(
                child_name,
                detected_functions,
                id_counter,
                active_stack,
                depth + 1,
                max_depth,
            );
            node.add_child(child_node);
        }
    }
    active_stack.pop();

    node
}

fn graph_json(tree: &TreeNode, root_name: &str, output_path: &str) -> Result<()> {
    let json_data = to_string_pretty(tree)?;
    let output_path = format!("{output_path}/json/{root_name}.json");
    let mut file = File::create(&output_path)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}
