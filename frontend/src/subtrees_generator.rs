use manifest_producer_backend::FunctionNode;
use std::collections::HashMap;

use crate::TreeNode;

pub(crate) fn identify_subtrees(
    root_name: &str,
    forest: &HashMap<String, FunctionNode>,
    node_roots: &mut HashMap<String, FunctionNode>,
) {
    let mut active_stack = Vec::new();

    trace_roots(root_name, forest, node_roots, &mut active_stack, 0);
}

fn trace_roots(
    function_name: &str,
    forest: &HashMap<String, FunctionNode>,
    node_roots: &mut HashMap<String, FunctionNode>,
    active_stack: &mut Vec<String>,
    depth: usize,
) {
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
            trace_roots(child_name, forest, node_roots, active_stack, depth + 1);
        }
    }

    active_stack.pop();
}

pub(crate) fn build_subtrees(
    node_roots: &mut HashMap<String, FunctionNode>,
    detected_functions: &HashMap<String, FunctionNode>,
    sub_trees: &mut HashMap<String, TreeNode>,
    id_counter: &mut usize,
) {
    for (name, _node) in node_roots.iter() {
        // Construct the subtree for this node
        let mut active_stack = Vec::new();

        let sub_tree = subtree_generation(
            name,
            detected_functions,
            sub_trees,
            &mut active_stack,
            id_counter,
            0,
        );
        sub_trees.insert(name.clone(), sub_tree);
    }
}

fn subtree_generation(
    function_name: &str,
    detected_functions: &HashMap<String, FunctionNode>,
    sub_trees: &mut HashMap<String, TreeNode>,
    active_stack: &mut Vec<String>,
    id_counter: &mut usize,
    depth: usize,
) -> TreeNode {
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
                depth + 1,
            );
            node.add_child(child_node);
        }
    }
    active_stack.pop();

    sub_trees.insert(function_name.to_string(), node.clone());

    node
}
