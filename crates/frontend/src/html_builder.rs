use std::{collections::HashMap, fs::File, io::Write};

use crate::{error::Result, graph_builder::graph_builder, TreeNode};
use manifest_producer_backend::{BasicInfo, FunctionNode};

use minijinja::{context, Environment};
use serde_json;

/// Generates HTML reports from analysis results.
///
/// This function creates various HTML pages summarizing the results of the analysis, including:
///
/// - An index page with general metadata.
/// - A functions page listing detected functions.
/// - A root page displaying the entry point functions.
/// - A call graph page visualizing function relationships.
///
/// # Arguments
///
/// - `basic_info`: Metadata about the binary being analyzed.
/// - `detected_functions`: A mutable map of function names to their associated `FunctionNode` objects.
/// - `root_nodes`: The root function names identified during analysis.
/// - `output_path`: The directory where the HTML files should be saved.
/// - `max_depth`: An optional depth limit for the function call graph.
///
/// # Workflow
///
/// 1. Generates the index page using `render_index_page`.
/// 2. Creates the functions listing with `render_functions_page`.
/// 3. Renders the root function overview using `render_root_page`.
/// 4. Builds the function call graph using `graph_builder`.
/// 5. Produces the tree visualization with `render_tree_page`.
///
/// # Returns
///
/// - `Ok(())`: If all HTML files are generated successfully.
/// - `Err(e)`: If any operation fails, an error is returned.
///
/// # Errors
///
/// Errors may arise from:
/// - File I/O failures during HTML generation.
/// - Issues with input data structures.
/// - Failures in function call graph construction.
pub fn html_builder<S: ::std::hash::BuildHasher>(
    basic_info: &BasicInfo,
    detected_functions: &mut HashMap<String, FunctionNode, S>,
    root_nodes: &str,
    output_path: &str,
    max_depth: Option<usize>,
) -> Result<()> {
    let safe_root_name = sanitize_name(root_nodes);
    render_index_page(basic_info, detected_functions.len(), output_path)?;
    render_functions_page(detected_functions, output_path)?;
    render_root_page(&safe_root_name, output_path)?;

    let js_tree = graph_builder(detected_functions, &safe_root_name, output_path, max_depth)?;
    render_tree_page(&safe_root_name, &js_tree, output_path)?;
    Ok(())
}

fn render_index_page(basic_info: &BasicInfo, num_func: usize, output_path: &str) -> Result<()> {
    let mut env = Environment::new();
    env.add_template("index.html", include_str!("templates/index.html"))?;

    let template = env.get_template("index.html")?;
    let rendered = template.render(context! {
        basic_info => basic_info,
        num_func => num_func,
    })?;

    let mut file = File::create(format!("{output_path}/index.html"))?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

fn render_functions_page<S: ::std::hash::BuildHasher>(
    detected_functions: &mut HashMap<String, FunctionNode, S>,
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

fn render_root_page(roots: &str, output_path: &str) -> Result<()> {
    let mut env = Environment::new();
    env.add_template(
        "root_functions.html",
        include_str!("templates/root_functions.html"),
    )?;

    let template = env.get_template("root_functions.html")?;
    let rendered = template.render(context! {
        root => roots,
    })?;

    let mut file = File::create(format!("{output_path}/root_functions.html"))?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

fn render_tree_page(root_name: &str, js_tree: &TreeNode, output_path: &str) -> Result<()> {
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

pub(crate) fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            ':' | '<' | '>' | '"' | '|' | '*' | '?' | '\r' | '\n' => '_',
            _ => c,
        })
        .collect()
}
