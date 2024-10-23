use crate::{
    backend::{error::Result, func_analyzer::CallTree},
    frontend::html_generator::render_tree_page,
};

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

use std::{cell::RefCell, collections::HashMap, fs::File, io::Write, rc::Rc};

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

// FUNZIONA MA INCOMPLETO
// fn tree_generator(
//     function_name: &str,
//     forest: &HashMap<String, CallTree>,
//     id_counter: &mut usize,
//     active_stack: &mut Vec<String>, // Stack to detect cycles
// ) -> TreeNode {
//     if active_stack.contains(&function_name.to_string()) {
//         let node = TreeNode::new(*id_counter, function_name);
//         *id_counter += 1;
//         return node;
//     }

//     active_stack.push(function_name.to_string());
//     let mut node = TreeNode::new(*id_counter, function_name);
//     *id_counter += 1;

//     if let Some(call_tree) = forest.get(function_name) {
//         for child_name in &call_tree.nodes {
//             let child_node = tree_generator(child_name, forest, id_counter, active_stack);
//             node.add_child(child_node);
//         }
//     }
//     active_stack.pop();
//     node 
// }

// COMPLETO MA NON FUNZIONA
// fn tree_generator(
//     function_name: &str,
//     forest: &HashMap<String, CallTree>,
//     id_counter: &mut usize,
//     active_stack: &mut Vec<String>, // Stack to detect cycles
// ) -> TreeNode {
//     if active_stack.contains(&function_name.to_string()) {
//         let node = TreeNode::new(*id_counter, function_name);
//         *id_counter += 1;
//         return node;
//     }

//     active_stack.push(function_name.to_string());
//     let mut node = TreeNode::new(*id_counter, function_name);
//     *id_counter += 1;

//     if let Some(call_tree) = forest.get(function_name) {
//         for child_name in &call_tree.nodes {
//             let child_node = tree_generator(child_name, forest, id_counter, active_stack);
//             node.add_child(child_node);
//         }
//         active_stack.pop();
//     }
//     node
// }

// fn tree_generator(
//     function_name: &str,
//     forest: &HashMap<String, CallTree>,
//     id_counter: &mut usize,
//     active_stack: &mut Vec<String>, // Stack to detect cycles
//     node_ptr: &mut HashMap<String, Rc<TreeNode>>
// ) -> TreeNode {
//     if active_stack.contains(&function_name.to_string()) {
//         let node = TreeNode::new(*id_counter, function_name);
//         *id_counter += 1;
//         return node;
//     }

//     // Recupera il sottoalbero già esplorato
//     if let Some(existing_node) = node_ptr.get(function_name) {
//         println!("SOTTOALBERO: {:?}", existing_node);
//         return existing_node.as_ref().clone();
//     }

//     active_stack.push(function_name.to_string());
//     // println!("{:?}", active_stack);
//     let mut node = TreeNode::new(*id_counter, function_name);
//     *id_counter += 1;
//     node_ptr.insert(function_name.to_string(), Rc::new(node.clone().into()));

//     if let Some(call_tree) = forest.get(function_name) {
//         for child_name in &call_tree.nodes {
//             let child_node = tree_generator(child_name, forest, id_counter, active_stack, node_ptr);
//             node.add_child(child_node);
//         }
//         // active_stack.pop();
//     }
//     active_stack.pop();
//     // println!("removed {}", function_name);
//     // println!("{:?}", active_stack);
//     node
// }


fn tree_generator(
    function_name: &str,
    forest: &HashMap<String, CallTree>,
    id_counter: &mut usize,
    active_stack: &mut Vec<String>, // Stack to detect cycles
    node_ptr: &mut HashMap<String, Rc<RefCell<TreeNode>>>
) -> TreeNode {
    // Verifica cicli
    if active_stack.contains(&function_name.to_string()) {
        let node = TreeNode::new(*id_counter, function_name);
        *id_counter += 1;
        return node;
    }

    // Recupera il sottoalbero già esplorato
    if let Some(existing_node) = node_ptr.get(function_name) {
        return existing_node.borrow().clone();  // Usa `borrow` per restituire una copia del nodo
    }

    active_stack.push(function_name.to_string());
    let node = Rc::new(RefCell::new(TreeNode::new(*id_counter, function_name)));
    *id_counter += 1;
    node_ptr.insert(function_name.to_string(), Rc::clone(&node));

    // Aggiunta dei figli
    if let Some(call_tree) = forest.get(function_name) {
        for child_name in &call_tree.nodes {
            let child_node = tree_generator(child_name, forest, id_counter, active_stack, node_ptr);
            node.borrow_mut().add_child(child_node); // Usa `borrow_mut` per accedere al nodo mutabile
        }
    }

    active_stack.pop();

    // Ritorna una copia del nodo finale come `TreeNode`
    let x = node.borrow().clone(); x
}


// TEST
pub fn build_tree(roots: &Vec<String>, forest: &HashMap<String, CallTree>) -> Result<()> {
    let mut id_counter = 0;
    let mut active_stack = Vec::new();
    let mut node_ptr = HashMap::new();

    for root in roots {
        let js_tree = tree_generator(root, forest, &mut id_counter, &mut active_stack, &mut node_ptr);
        json_generator(&js_tree, root)?;
        render_tree_page(root, &js_tree)?;
    }

    Ok(())
}


// FUNZIONA
// pub fn build_tree(roots: &Vec<String>, forest: &HashMap<String, CallTree>) -> Result<()> {
//     let mut id_counter = 0;
//     let mut active_stack = Vec::new();

//     for root in roots {
//         let js_tree = tree_generator(root, forest, &mut id_counter, &mut active_stack);
//         json_generator(&js_tree, root)?;
//         render_tree_page(root, &js_tree)?;
//     }

//     Ok(())
// }

fn json_generator(tree: &TreeNode, root_name: &str) -> Result<()> {
    let json_data = to_string_pretty(tree)?;
    let output_path = format!("./public/json/{}.json", root_name);
    let mut file = File::create(&output_path)?;
    file.write_all(json_data.as_bytes())?;

    Ok(())
}
