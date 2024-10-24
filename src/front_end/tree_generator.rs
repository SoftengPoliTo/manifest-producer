use crate::{
    back_end::{error::Result, func_analyzer::CallTree},
    front_end::html_generator::render_tree_page,
};

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::{
    collections::HashMap,
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
    active_stack: &mut Vec<String>, // Stack to detect cycles
) -> TreeNode {
    if active_stack.contains(&function_name.to_string()) {
        let node = TreeNode::new(*id_counter, function_name); 
        *id_counter += 1;  
        return node;  
    }

    active_stack.push(function_name.to_string());

    if let Some(call_tree) = forest.get(function_name) {
        let mut node = TreeNode::new(*id_counter, &call_tree.name);
        *id_counter += 1; 

        for child_name in &call_tree.nodes {
            let child_node = tree_generator(child_name, forest, id_counter, active_stack);
            node.add_child(child_node); 
        }

        active_stack.pop();

        node
    } else {
        let node = TreeNode::new(*id_counter, function_name); 
        *id_counter += 1;

        node
    }
}


pub fn build_tree(roots: &Vec<String>, forest: &HashMap<String, CallTree>) -> Result<()> {
    let mut id_counter = 0;
    let mut active_stack = Vec::new();

    for root in roots {
        let js_tree = tree_generator(root, forest, &mut id_counter, &mut active_stack);
            json_generator(&js_tree, root)?;
            render_tree_page(root, &js_tree)?;
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
