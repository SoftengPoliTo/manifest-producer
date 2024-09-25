//! manifest-producer is a tool for analyzing ELF files to produce 
//! manifests that describe information extracted from ELF libraries and binaries.
//!
//! //! ## Supported Languages
//!
//! - C
//! - C++
//! - Rust
//!
//! ## Results Produced
//!
//! - Basic informations:
//!   - file_name: The name of the ELF file.
//!   - programming language: The programming language used to build the ELF file.
//!   - target architecture: The architecture of the ELF file.
//!   - link: Whether the ELF file is statically or dynamically linked.
//!   - file_type: The type of the ELF file.
//!   - file size: The size of the ELF binary.
//!   - entry_point: The entry point of the ELF file.


pub mod back_end {
    pub mod api_analyzer;
    pub mod elf_analyzer;
    pub mod error;
    pub mod func_analyzer;
}
