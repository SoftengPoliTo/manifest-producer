use crate::error::Result;
use indicatif::{ProgressBar, ProgressStyle};
use manifest_producer_backend::FunctionNode;
use std::{collections::HashMap, time::Duration};

use crate::TreeNode;

pub(crate) fn identify_subtrees(
    root_name: &str,
    forest: &HashMap<String, FunctionNode>,
    node_roots: &mut HashMap<String, FunctionNode>,
) -> Result<()> {
    let mut active_stack = Vec::new();

    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
    );
    progress_bar.enable_steady_tick(Duration::from_millis(100));

    trace_roots(
        root_name,
        forest,
        node_roots,
        &mut active_stack,
        &progress_bar,
        0,
    );

    Ok(())
}

fn trace_roots(
    function_name: &str,
    forest: &HashMap<String, FunctionNode>,
    node_roots: &mut HashMap<String, FunctionNode>,
    active_stack: &mut Vec<String>,
    progress_bar: &ProgressBar,
    depth: usize,
) {
    progress_bar.set_message(format!("Detection of subtrees (Depth: {})", depth));

    if active_stack.contains(&function_name.to_string()) || depth > 10 {
        return;
    }

    if let Some(existing_node) = node_roots.get_mut(function_name) {
        existing_node.jmp += 1;
        return;
    }

    if let Some(node) = forest.get(function_name) {
        let mut new_node = node.clone();
        new_node.jmp = 0;
        node_roots.insert(function_name.to_string(), new_node);
    }

    active_stack.push(function_name.to_string());

    if let Some(node) = forest.get(function_name) {
        for child_name in &node.children {
            trace_roots(
                child_name,
                forest,
                node_roots,
                active_stack,
                progress_bar,
                depth + 1,
            );
        }
    }

    active_stack.pop();
}

pub(crate) fn build_subtrees(
    node_roots: &mut HashMap<String, FunctionNode>,
    detected_functions: &HashMap<String, FunctionNode>,
    sub_trees: &mut HashMap<String, TreeNode>,
    id_counter: &mut usize,
) -> Result<()> {
    for (name, _node) in node_roots.iter() {
        // Construct the subtree for this node
        let mut active_stack = Vec::new();
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
        );
        progress_bar.enable_steady_tick(Duration::from_millis(100));

        let sub_tree = subtree_generation(
            name,
            detected_functions,
            sub_trees,
            &mut active_stack,
            id_counter,
            &progress_bar,
            0,
        );
        sub_trees.insert(name.clone(), sub_tree);
        progress_bar
            .finish_with_message(format!("New subtrees generated for '{}' function.", name));
    }
    Ok(())
}

fn subtree_generation(
    function_name: &str,
    detected_functions: &HashMap<String, FunctionNode>,
    sub_trees: &mut HashMap<String, TreeNode>,
    active_stack: &mut Vec<String>,
    id_counter: &mut usize,
    progress_bar: &ProgressBar,
    depth: usize,
) -> TreeNode {
    progress_bar.set_message("Detecting subtrees...");

    if active_stack.contains(&function_name.to_string()) || depth > 10 {
        let node = TreeNode::new(*id_counter, function_name);
        *id_counter += 1;
        return node;
    }

    if let Some(existing_subtree) = sub_trees.get(function_name) {
        return existing_subtree.clone();
    }

    active_stack.push(function_name.to_string());

    let mut node = TreeNode::new(*id_counter, function_name);
    *id_counter += 1;

    if let Some(function_node) = detected_functions.get(function_name) {
        for child_name in &function_node.children {
            let child_node = subtree_generation(
                child_name,
                detected_functions,
                sub_trees,
                active_stack,
                id_counter,
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
