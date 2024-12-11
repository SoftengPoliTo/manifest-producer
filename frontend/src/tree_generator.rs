use std::{collections::HashMap, fs::File, io::Write, time::Duration};

use crate::{error::Result, html_generator::render_tree_page, TreeNode};
use manifest_producer_backend::FunctionNode;

use indicatif::{ProgressBar, ProgressStyle};
use serde_json::to_string_pretty;

pub(crate) fn build_tree(
    root: &str,
    forest: &HashMap<String, FunctionNode>,
    sub_trees: &mut HashMap<String, TreeNode>,
    id_counter: &mut usize,
    output_path: &str,
) -> Result<()> {
    let mut active_stack = Vec::new();
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
    );
    progress_bar.enable_steady_tick(Duration::from_millis(100));

    let js_tree = generate_tree(
        root,
        forest,
        id_counter,
        &mut active_stack,
        sub_trees,
        &progress_bar,
        0,
    );
    generate_json(&js_tree, root, output_path)?;
    render_tree_page(root, &js_tree, output_path)?;

    progress_bar.finish_with_message(format!("New tree generated for '{}' function.", root));
    Ok(())
}

fn generate_tree(
    function_name: &str,
    forest: &HashMap<String, FunctionNode>,
    id_counter: &mut usize,
    active_stack: &mut Vec<String>,
    sub_trees: &mut HashMap<String, TreeNode>,
    progress_bar: &ProgressBar,
    depth: usize,
) -> TreeNode {
    progress_bar.set_message("Planting a tree...");

    if active_stack.contains(&function_name.to_string()) || depth > 10 {
        let node = TreeNode::new(*id_counter, function_name);
        *id_counter += 1;
        return node;
    }

    if let Some(existing_node) = sub_trees.get(function_name) {
        return existing_node.clone();
    }

    active_stack.push(function_name.to_string());

    let mut node = TreeNode::new(*id_counter, function_name);
    *id_counter += 1;

    if let Some(call_tree) = forest.get(function_name) {
        for child_name in &call_tree.children {
            let child_node = generate_tree(
                child_name,
                forest,
                id_counter,
                active_stack,
                sub_trees,
                progress_bar,
                depth + 1,
            );
            node.add_child(child_node);
        }
    }

    active_stack.pop();

    sub_trees.insert(function_name.to_string(), node.clone());
    node
}

fn generate_json(tree: &TreeNode, root_name: &str, output_path: &str) -> Result<()> {
    let json_data = to_string_pretty(tree)?;
    let output_path = format!("{}/json/{}.json", output_path, root_name);
    let mut file = File::create(&output_path)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}
