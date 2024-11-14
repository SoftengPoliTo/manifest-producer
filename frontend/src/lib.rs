pub mod error;
pub mod html_generator;
pub mod subtrees_generator;
pub mod tree_generator;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TreeNode {
    pub id: usize,
    pub text: String,
    #[serde(rename = "children")]
    pub children: Option<Box<Vec<TreeNode>>>,
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
            self.children = Some(Box::new(vec![child]));
        }
    }
}
