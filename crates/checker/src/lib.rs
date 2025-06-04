pub mod checks;

#[derive(serde::Serialize, Debug)]
pub struct ValidationResult {
    name: String,
    status: bool,
    description: String,
    metadata: Option<serde_json::Value>,
}

#[derive(serde::Serialize, Debug)]
pub struct CategoryResult {
    name: String,
    description: String,
    checks: Vec<ValidationResult>,
}

#[derive(serde::Serialize, Debug)]
pub struct ValidationReport {
    binary_path: String,
    categories: Vec<CategoryResult>,
}
