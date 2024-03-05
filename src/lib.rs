//! manifest-producer is a tool for analyzing ELF files to produce manifests that describe information extracted from ELF libraries and binaries.
//!
//! //! ## Supported Languages
//!
//! - C
//! - C++
//! - Rust
//! 
//! ## Manifest Produced
//! 
//! - Basic informations:
//!   - file_name: The name of the ELF file.
//!   - programming language: The programming language used to build the ELF file.
//!   - architecture: The architecture of the ELF file.
//!   - link: Whether the ELF file is statically or dynamically linked.
//!   - file_type: The type of the ELF file.
//!   - endianness: The endianness of the ELF file.
//!   - header_size: The size of the ELF header.
//!   - entry_point: The entry point of the ELF file.
//!   - APIs found: The list of identified public APIs.
//! 
//! - Flow called functions:
//!   - For each identified API, lists the function calls (system calls or subfunctions).
//! 
//! - Features associated to each APIs:
//!   - Categorizes APIs based on their functionality features.
//! 

pub mod manifest_creation;
pub mod code_section_handler;
pub mod plt_mapping;
pub mod dwarf_analysis;
pub mod api_detection;
pub mod elf_utils;
pub mod cleanup;
pub mod error;
