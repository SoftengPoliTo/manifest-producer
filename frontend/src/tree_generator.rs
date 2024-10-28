use std::{collections::HashMap, fs::File, io::Write, time::Duration};

use common::{
    error::Result,
    indicatif::{ProgressBar, ProgressStyle},
    serde_json::to_string_pretty,
    CallTree, FunctionNode, TreeNode,
};

use crate::html_generator::render_tree_page;

pub fn identify_subtrees(
    root_name: &str,
    forest: &HashMap<String, CallTree>,
    node_roots: &mut HashMap<String, FunctionNode>,
) -> Result<()> {
    let mut active_stack = Vec::new();

    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
    );
    progress_bar.enable_steady_tick(Duration::from_millis(100)); // Tick every 100ms for a continuous animation

    trace_roots(
        root_name,
        forest,
        node_roots,
        &mut active_stack,
        &progress_bar,
    );

    Ok(())
}

fn trace_roots(
    function_name: &str,
    forest: &HashMap<String, CallTree>,
    node_roots: &mut HashMap<String, FunctionNode>,
    active_stack: &mut Vec<String>,
    progress_bar: &ProgressBar,
) {
    progress_bar.set_message(format!("Processing: {}", function_name));

    // Avoid loops, if present in the active stack
    if active_stack.contains(&function_name.to_string()) {
        return;
    }

    // If the node already exists in node_roots, increment `jmp` and avoid descending into children
    if let Some(existing_node) = node_roots.get_mut(function_name) {
        existing_node.jmp += 1;
        return;
    }

    node_roots.insert(
        function_name.to_string(),
        FunctionNode {
            name: function_name.to_owned(),
            jmp: 0,
        },
    );

    active_stack.push(function_name.to_string());

    // Scroll recursively in children, if any
    if let Some(call_tree) = forest.get(function_name) {
        for child_name in &call_tree.nodes {
            trace_roots(child_name, forest, node_roots, active_stack, progress_bar);
        }
    }

    active_stack.pop();
}

pub fn build_subtrees(
    node_roots: &mut HashMap<String, FunctionNode>,
    forest: &HashMap<String, CallTree>,
    sub_trees: &mut HashMap<String, TreeNode>,
    id_counter: &mut usize,
) {
    // Filter nodes with jmp > 0
    let mut nodes_to_remove: Vec<String> = vec![];

    for (name, node) in node_roots.iter() {
        if node.jmp == 0 {
            nodes_to_remove.push(name.clone());
        } else {
            // Construct the subtree for this node
            let mut active_stack = Vec::new();
            let sub_tree =
                subtree_generation(name, forest, sub_trees, &mut active_stack, id_counter);
            sub_trees.insert(name.clone(), sub_tree);
        }
    }

    // Remove nodes with jmp = 0
    for name in nodes_to_remove {
        node_roots.remove(&name);
    }

    // Finds and removes leaf nodes (those without children)
    let leaf_nodes: Vec<String> = sub_trees
        .iter()
        .filter(|(_, node)| {
            node.children
                .as_ref()
                .map_or(true, |children| children.is_empty())
        })
        .map(|(name, _)| name.clone())
        .collect();

    for name in leaf_nodes {
        sub_trees.remove(&name);
    }
}

fn subtree_generation(
    function_name: &str,
    forest: &HashMap<String, CallTree>,
    sub_trees: &mut HashMap<String, TreeNode>,
    active_stack: &mut Vec<String>, // Stack to detect cycles
    id_counter: &mut usize,
) -> TreeNode {
    // Avoid loops, if present in the active stack
    if active_stack.contains(&function_name.to_string()) {
        return TreeNode::new(*id_counter, function_name);
    }

    if let Some(existing_subtree) = sub_trees.get(function_name) {
        return existing_subtree.clone();
    }

    active_stack.push(function_name.to_string());

    let mut node = TreeNode::new(*id_counter, function_name);
    *id_counter += 1;

    if let Some(call_tree) = forest.get(function_name) {
        for child_name in &call_tree.nodes {
            let child_node =
                subtree_generation(child_name, forest, sub_trees, active_stack, id_counter);
            node.add_child(child_node);
        }
    }

    active_stack.pop();

    sub_trees.insert(function_name.to_string(), node.clone());

    node
}

pub fn build_tree(
    root: &str,
    forest: &HashMap<String, CallTree>,
    sub_trees: &mut HashMap<String, TreeNode>,
    id_counter: &mut usize,
) -> Result<()> {
    let mut active_stack = Vec::new();
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}\nElapsed: {elapsed_precise}")?,
    );
    progress_bar.enable_steady_tick(Duration::from_millis(100));
    progress_bar.set_message(format!("Constructing call graph for {}", root));

    let js_tree = generate_tree(
        root,
        forest,
        id_counter,
        &mut active_stack,
        sub_trees,
        &progress_bar,
    );
    generate_json(&js_tree, root)?;
    render_tree_page(root, &js_tree)?;

    progress_bar.finish_with_message(format!("Completed graph for '{}'", root));
    Ok(())
}

fn generate_tree(
    function_name: &str,
    forest: &HashMap<String, CallTree>,
    id_counter: &mut usize,
    active_stack: &mut Vec<String>,
    sub_trees: &mut HashMap<String, TreeNode>,
    progress_bar: &ProgressBar,
) -> TreeNode {
    progress_bar.set_message(format!("Processing: {}", function_name));

    if active_stack.contains(&function_name.to_string()) {
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
        for child_name in &call_tree.nodes {
            let child_node = generate_tree(
                child_name,
                forest,
                id_counter,
                active_stack,
                sub_trees,
                progress_bar,
            );
            node.add_child(child_node);
        }
    }

    active_stack.pop();

    sub_trees.insert(function_name.to_string(), node.clone());
    node
}

fn generate_json(tree: &TreeNode, root_name: &str) -> Result<()> {
    let json_data = to_string_pretty(tree)?;
    let output_path = format!("./public/json/{}.json", root_name);
    let mut file = File::create(&output_path)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}
