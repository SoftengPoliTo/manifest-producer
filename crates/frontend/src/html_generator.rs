use std::{collections::HashMap, fs::File, io::Write};

use crate::{
    error::Result,
    subtrees_generator::{build_subtrees, identify_subtrees},
    tree_generator::build_tree,
    TreeNode,
};
use manifest_producer_backend::{BasicInfo, FunctionNode};

use minijinja::{context, Environment};
use serde_json;

/// Generates HTML reports from analysis results.
///
/// This function orchestrates the creation of various HTML pages that summarize the results
/// of the analysis, including:
///
/// - An index page with general information.
/// - A page listing detected functions.
/// - A root page displaying entry points (root nodes).
///
/// Additionally, it identifies subtrees, cleans redundant nodes, and constructs trees
/// for visual representation.
///
/// # Arguments
///
/// - `basic_info`: Metadata about the binary being analyzed.
/// - `detected_functions`: A map of function names to their associated `FunctionNode` objects.
/// - `root_nodes`: A vector of root function names identified during analysis.
/// - `output_path`: The directory where the HTML files should be saved.
///
/// # Workflow
///
/// 1. Renders the main HTML pages using helper functions.
/// 2. Iterates over `root_nodes` to build and render subtrees and visual trees.
/// 3. Cleans temporary structures (`node_roots` and `sub_trees`) after processing each root.
///
/// # Returns
///
/// - `Ok(())`: If all HTML files are generated successfully.
/// - `Err(e)`: If any operation fails, an error is returned.
///
/// # Errors
///
/// Errors can arise from:
/// - File I/O failures during HTML generation.
/// - Invalid or incomplete data structures passed to the function.
/// - Errors in subtree or tree construction.
///
/// # Example
///
/// ```
/// use manifest_producer_frontend::html_generator;
/// use manifest_producer_backend::BasicInfo;
/// use std::collections::HashMap;
/// use manifest_producer_frontend::html_generator::html_generator;
///
/// let basic_info = BasicInfo::new("example.elf", "Executable");
/// let detected_functions = HashMap::new();
/// let root_nodes = vec!["main".to_string()];
/// let output_path = "./output";
///
/// if let Err(err) = html_generator(&basic_info, &detected_functions, &root_nodes, output_path) {
///     eprintln!("HTML generation failed: {}", err);
/// }
/// ```
#[allow(clippy::implicit_hasher)]
pub fn html_generator(
    basic_info: &BasicInfo,
    detected_functions: &HashMap<String, FunctionNode>,
    root_nodes: &Vec<String>,
    output_path: &str,
) -> Result<()> {

    #[cfg(feature = "progress_bar")]
    let pb = {
        let pb = indicatif::ProgressBar::new((root_nodes.len()*2) as u64);
        pb.set_message("Building call graphs:".to_string());
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{msg}\n{wide_bar} {pos}/{len} [{elapsed_precise}]")?,
        );
        pb
    };

    let mut node_roots: HashMap<String, FunctionNode> = HashMap::new();
    let mut sub_trees: HashMap<String, TreeNode> = HashMap::new();
    let mut id_counter = 0;

    render_index_page(
        basic_info,
        detected_functions.len(),
        root_nodes.len(),
        output_path,
    )?;
    render_functions_page(detected_functions, output_path)?;
    render_root_page(root_nodes, output_path)?;

    for root in root_nodes {
        #[cfg(feature = "progress_bar")]
        pb.inc(1);

        // Step 1: Identification of subtrees
        identify_subtrees(root, detected_functions, &mut node_roots);

        // Step 2: Cleaning nodes with jmp equal to zero and creating subtrees
        build_subtrees(
            &mut node_roots,
            detected_functions,
            &mut sub_trees,
            &mut id_counter,
        );

        // Step 3: Construction of the tree
        build_tree(
            root,
            detected_functions,
            &mut sub_trees,
            &mut id_counter,
            output_path,
        )?;

        // Empty the structures for the next cycle
        node_roots.clear();
        sub_trees.clear();

        #[cfg(feature = "progress_bar")]
        pb.inc(1);
    }
    #[cfg(feature = "progress_bar")]
    pb.finish_with_message("Call graphs built!");
    Ok(())
}

pub(crate) fn render_index_page(
    basic_info: &BasicInfo,
    num_func: usize,
    num_root: usize,
    output_path: &str,
) -> Result<()> {
    let mut env = Environment::new();
    env.add_template("index.html", include_str!("templates/index.html"))?;

    let template = env.get_template("index.html")?;
    let rendered = template.render(context! {
        basic_info => basic_info,
        num_func => num_func,
        num_root => num_root
    })?;

    let mut file = File::create(format!("{output_path}/index.html"))?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

pub(crate) fn render_functions_page(
    detected_functions: &HashMap<String, FunctionNode>,
    output_path: &str,
) -> Result<()> {
    let functions: Vec<FunctionNode> = detected_functions.values().cloned().collect();

    let mut env = Environment::new();
    env.add_template(
        "functions_list.html",
        include_str!("templates/functions_list.html"),
    )?;
    let template = env.get_template("functions_list.html")?;
    let rendered = template.render(context! {
        functions => functions
    })?;

    let mut file = File::create(format!("{output_path}/functions_list.html"))?;
    file.write_all(rendered.as_bytes())?;

    Ok(())
}

pub(crate) fn render_root_page(roots: &Vec<String>, output_path: &str) -> Result<()> {
    let mut env = Environment::new();
    env.add_template(
        "root_functions.html",
        include_str!("templates/root_functions.html"),
    )?;

    let template = env.get_template("root_functions.html")?;
    let rendered = template.render(context! {
        roots => roots,
    })?;

    let mut file = File::create(format!("{output_path}/root_functions.html"))?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

pub(crate) fn render_tree_page(
    root_name: &str,
    js_tree: &TreeNode,
    output_path: &str,
) -> Result<()> {
    let mut env = Environment::new();
    env.add_template("call_tree.html", include_str!("templates/call_tree.html"))?;

    let template = env.get_template("call_tree.html")?;

    let js_tree_json = serde_json::to_string(&js_tree)?;

    let rendered = template.render(context! {
        root_name => root_name,
        js_tree => js_tree_json,
    })?;

    let mut file = File::create(format!("{output_path}/call_trees/{root_name}.html"))?;
    file.write_all(rendered.as_bytes())?;

    Ok(())
}
