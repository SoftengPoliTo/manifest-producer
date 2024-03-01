#### MANIFEST PRODUCER

This project is a tool for analyzing ELF files to produce manifests that describe information extracted from ELF libraries and binaries.

### Project Structure

The project is divided into two main parts: a Rust library containing modules and functionalities for analyzing ELF files, and a binary for direct usage of the tool.

## Library Structure

The library is organized into the following modules:

* `elf_utils`: Utility functions for analyzing ELF files.
* `dwarf_analysis`: Analysis of ELF .debug_info section.
* `api_detection`: Searching for APIs in ELF symbols.
* `plt_mapping`: Mapping of .plt and .rela.plt sections.
* `code_section_handler`: Handling ELF code sections, disassembling the code of the APIs.
* `cleanup`: Cleaning of mangled function names.
* `manifest_creation`: Module for creating manifests.
* `error`: Definition of custom errors and result types.

## Binary Structure

The manifest-producer binary uses the library to perform analysis of ELF files. It is divided into three main parts:

1. Acquisition of ELF file data and APIs provided by the user.
2. Utilization of library functionalities for ELF file analysis.
3. Utilization of library functionalities for the Manifest creation.

### Usage

To use the manifest-producer tool, you can run the following command from the command line:

```bash
cargo run manifest-producer <elf_file_path>
```

Where `<elf_file_path>` is the path to the ELF file to be analyzed.

### Dependencies

The project uses the following main dependencies:

* [goblin](https://docs.rs/goblin/latest/goblin/) and [gimli](https://docs.rs/gimli/latest/gimli/): Rust libraries for analyzing ELF files.
* [capstone](https://docs.rs/capstone/latest/capstone/): Rust library for analyzing machine code.
* [cpp_demangle](https://docs.rs/cpp_demangle/latest/cpp_demangle/) and [rustc_demangle](https://docs.rs/rustc-demangle/latest/rustc_demangle/): Rust libraries for demangling C++ and Rust function names.
* [memmap2](https://docs.rs/memmap2/latest/memmap2/): Rust library for memory-mapping files.

