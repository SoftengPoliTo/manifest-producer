pub mod error;
pub mod html_generator;
pub mod subtrees_generator;
pub mod tree_generator;

/// Represents a tree structure used for visualizing interactions between functions.
///
/// The `TreeNode` struct is designed to facilitate the hierarchical representation of
/// functions in an analyzed binary.
///
/// This structure is serialized and deserialized to ensure compatibility
/// with web-based visualization tools.
///
/// # Fields
/// - `id`: A unique identifier for the node.
/// - `text`: A label describing the node.
/// - `children`: An optional vector of child nodes.
///
/// # Example
/// ```
/// use manifest_producer_frontend::TreeNode;
///
/// let mut root = TreeNode::new(1, "Root Node");
/// let child = TreeNode::new(2, "Child Node");
/// root.add_child(child);
///
/// assert!(root.children.is_some());
/// ```
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TreeNode {
    pub id: usize,
    pub text: String,
    #[serde(rename = "children")]
    pub children: Option<Box<Vec<TreeNode>>>,
}
impl TreeNode {
    /// Creates a new `TreeNode` with no children.
    ///
    /// # Arguments
    /// - `id`: A unique identifier for the node.
    /// - `text`: A label describing the node.
    ///
    /// # Returns
    /// A `TreeNode` instance.
    #[must_use]
    pub fn new(id: usize, text: &str) -> Self {
        TreeNode {
            id,
            text: text.to_string(),
            children: None,
        }
    }

    /// Adds a child node to the current node.
    ///
    /// # Arguments
    /// - `child`: A `TreeNode` instance to be added as a child.
    pub fn add_child(&mut self, child: TreeNode) {
        if let Some(ref mut children) = self.children {
            children.push(child);
        } else {
            self.children = Some(Box::new(vec![child]));
        }
    }
}
