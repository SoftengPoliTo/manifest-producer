pub mod backend {
    pub mod api_analyzer;
    pub mod elf_analyzer;
    pub mod error;
    pub mod func_analyzer;
}

pub mod frontend {
    pub mod html_generator;
    pub mod tree_generator;
}
