pub mod analyse;
pub mod detect;
pub mod entry;
pub mod error;
pub mod inspect;

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
pub struct FunctionNode {
    pub name: String,
    pub start_addr: u64,
    pub end_addr: u64,
    pub invocation_entry: usize,
    pub jmp: usize,
    pub children: Vec<String>,
    pub disassembly: Option<String>,
}

impl FunctionNode {
    pub fn new(name: String, start_addr: u64, end_addr: u64) -> Self {
        Self {
            name,
            start_addr,
            end_addr,
            invocation_entry: 0,
            jmp: 0,
            children: Vec::new(),
            disassembly: None,
        }
    }

    pub fn set_disassembly(&mut self, disassembly: String) {
        self.disassembly = Some(disassembly);
    }
}
