use std::{collections::HashMap, fs::File, io::Write};

use common::{
    error::Result,
    minijinja::{context, Environment},
    open, serde_json, BasicInfo, CallTree, TreeNode, FUNC,
};

pub fn html_generator(
    basic_info: BasicInfo,
    num_func: usize,
    num_root: usize,
    functions: &Vec<FUNC>,
    forest: &HashMap<String, CallTree>,
    disassembly: &Vec<(String, String)>,
    roots: &Vec<String>,
) -> Result<()> {
    render_index_page(basic_info, num_func, num_root)?;
    render_functions_page(functions, forest)?;
    render_disassembly_page(disassembly)?;
    render_root_page(roots)?;

    Ok(())
}

pub fn open_index_page() -> Result<()> {
    let index_path = "./public/index.html";
    open::that(index_path)?;
    Ok(())
}

pub fn render_index_page(basic_info: BasicInfo, num_func: usize, num_root: usize) -> Result<()> {
    let mut env = Environment::new();
    env.add_template("index.html", include_str!("templates/index.html"))?;

    let template = env.get_template("index.html")?;
    let rendered = template.render(context! {
        basic_info => basic_info,
        num_func => num_func,
        num_root => num_root
    })?;

    let mut file = File::create("public/index.html")?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

pub fn render_functions_page(
    functions: &Vec<FUNC>,
    forest: &HashMap<String, CallTree>,
) -> Result<()> {
    let mut combined: Vec<(FUNC, CallTree)> = Vec::new();

    for func in functions {
        let call_tree = forest.get(&func.name).cloned();
        if let Some(call_tree) = call_tree {
            combined.push((func.clone(), call_tree));
        }
    }

    let mut env = Environment::new();
    env.add_template(
        "functions_list.html",
        include_str!("templates/functions_list.html"),
    )?;
    let template = env.get_template("functions_list.html")?;
    let rendered = template.render(context! {
        combined => combined
    })?;

    let mut file = File::create("public/functions_list.html")?;
    file.write_all(rendered.as_bytes())?;

    Ok(())
}

pub fn render_disassembly_page(disassembly: &Vec<(String, String)>) -> Result<()> {
    let mut env = Environment::new();
    env.add_template(
        "disassembly_view.html",
        include_str!("templates/disassembly_view.html"),
    )?;

    let template = env.get_template("disassembly_view.html")?;
    let rendered = template.render(context! {
        disassembly => disassembly,
    })?;

    let mut file = File::create("public/disassembly_view.html")?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

pub fn render_root_page(roots: &Vec<String>) -> Result<()> {
    let mut env = Environment::new();
    env.add_template(
        "root_functions.html",
        include_str!("templates/root_functions.html"),
    )?;

    let template = env.get_template("root_functions.html")?;
    let rendered = template.render(context! {
        roots => roots,
    })?;

    let mut file = File::create("public/root_functions.html")?;
    file.write_all(rendered.as_bytes())?;
    Ok(())
}

pub fn render_tree_page(root_name: &str, js_tree: &TreeNode) -> Result<()> {
    let mut env = Environment::new();
    env.add_template("call_graph.html", include_str!("templates/call_graph.html"))?;

    let template = env.get_template("call_graph.html")?;

    let js_tree_json = serde_json::to_string(&js_tree)?;

    let rendered = template.render(context! {
        root_name => root_name,
        js_tree => js_tree_json,
    })?;

    let mut file = File::create(format!("public/call_graphs/{}.html", root_name))?;
    file.write_all(rendered.as_bytes())?;

    Ok(())
}
