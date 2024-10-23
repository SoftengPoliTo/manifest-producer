use crate::{
    back_end::{error::Result, func_analyzer::CallTree},
    front_end::html_generator::render_tree_page,
};

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::Write,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: usize,
    pub text: String,
    #[serde(rename = "children")]
    pub children: Option<Vec<TreeNode>>,
}
impl TreeNode {
    pub fn new(id: usize, text: &str) -> Self {
        TreeNode {
            id,
            text: text.to_string(),
            children: None,
        }
    }

    pub fn add_child(&mut self, child: TreeNode) {
        if let Some(ref mut children) = self.children {
            children.push(child);
        } else {
            self.children = Some(vec![child]);
        }
    }
}

fn tree_generator(
    function_name: &str,
    forest: &HashMap<String, CallTree>,
    id_counter: &mut usize,
    active_stack: &mut VecDeque<String>, // Stack to detect cycles
) -> Option<TreeNode> {
    // If the function is already on the stack, it means that there is a loop (function is called indirectly)
    if active_stack.contains(&function_name.to_string()) {
        return None;
    }

    // Adding the function to the stack to keep track of the fact that it is being processed
    active_stack.push_back(function_name.to_string());

    if let Some(call_tree) = forest.get(function_name) {
        let mut node = TreeNode::new(*id_counter, &call_tree.name);
        *id_counter += 1;

        for child_name in &call_tree.nodes {
            if let Some(child_node) = tree_generator(child_name, forest, id_counter, active_stack) {
                node.add_child(child_node);
            }
        }

        // Remove the function from the stack after it has been fully processed
        active_stack.pop_back();

        Some(node)
    } else {
        None
    }
}

pub fn build_tree(roots: &Vec<String>, forest: &HashMap<String, CallTree>) -> Result<()> {
    let mut id_counter = 0;
    let mut active_stack = VecDeque::new();

    for root in roots {
        if let Some(js_tree) = tree_generator(root, forest, &mut id_counter, &mut active_stack) {
            json_generator(&js_tree, root)?;
            render_tree_page(root, &js_tree)?;
        }
    }

    Ok(())
}

fn json_generator(tree: &TreeNode, root_name: &str) -> Result<()> {
    let json_data = to_string_pretty(tree)?;

    let output_path = format!("./public/json/{}.json", root_name);
    let mut file = File::create(&output_path)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}
