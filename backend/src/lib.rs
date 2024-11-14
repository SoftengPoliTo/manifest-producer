pub mod api_analyzer;
pub mod elf_analyzer;
pub mod func_analyzer;
pub mod error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BasicInfo<'a> {
    pub file_name: &'a str,
    pub file_type: &'a str,
    pub file_size: u64,
    pub arch: &'a str,
    pub pie: bool,
    pub stripped: bool,
    pub static_linking: &'a str,
    pub language: String,
    pub entry_point: u64,
}

impl<'a> BasicInfo<'a> {
    pub fn new(file_name: &'a str, file_type: &'a str) -> Self {
        Self {
            file_name,
            file_type,
            file_size: 0,
            arch: "",
            pie: false,
            stripped: false,
            static_linking: "",
            language: String::new(),
            entry_point: 0,
        }
    }

    pub fn file_size(self, file_size: u64) -> Self {
        Self { file_size, ..self }
    }

    pub fn arch(self, arch: &'a str) -> Self {
        Self { arch, ..self }
    }

    pub fn pie(self, pie: bool) -> Self {
        Self { pie, ..self }
    }

    pub fn static_linking(self, static_linking: &'a str) -> Self {
        Self {
            static_linking,
            ..self
        }
    }

    pub fn language(self, language: String) -> Self {
        Self { language, ..self }
    }

    pub fn entry_point(self, entry_point: u64) -> Self {
        Self {
            entry_point,
            ..self
        }
    }

    pub fn stripped(self, stripped: bool) -> Self {
        Self { stripped, ..self }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FUNC {
    pub name: String,
    pub start_address: u64,
    pub end_address: u64,
}
impl FUNC {
    pub fn new(name: String, start_address: u64, end_address: u64) -> Self {
        Self {
            name,
            start_address,
            end_address,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CallTree {
    pub name: String,
    pub invocation_count: usize,
    pub nodes: Vec<String>,
}
impl CallTree {
    pub fn new(name: String) -> Self {
        Self {
            name,
            invocation_count: 0,
            nodes: vec![],
        }
    }
    pub fn add_node(&mut self, node: String) {
        self.nodes.push(node);
    }
}