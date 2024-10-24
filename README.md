# manifest-producer

[![Actions Status][actions badge]][actions]
[![CodeCov][codecov badge]][codecov]
[![LICENSE][license badge]][license]
[![dependency status][status badge]][status]

This project is a tool for analyzing ELF files to produce manifests that describe information extracted from ELF libraries and binaries.

## Project Structure

The project is divided into two main parts: a Rust library containing modules and functionalities for analyzing ELF files, and a binary for direct usage of the tool.

### Library Structure

The library is organized into the following modules:

* `elf_utils`: Utility functions for analyzing ELF files.
* `dwarf_analysis`: Analysis of ELF .debug_info section.
* `api_detection`: Searching for APIs in ELF symbols.
* `plt_mapping`: Mapping of .plt and .rela.plt sections.
* `code_section_handler`: Handling ELF code sections, disassembling the code of the APIs.
* `cleanup`: Cleaning of mangled function names.
* `manifest_creation`: Module for creating manifests.
* `error`: Definition of custom errors and result types.

### Binary Structure

The manifest-producer binary uses the library to perform analysis of ELF files. It is divided into three main parts:

1. Acquisition of ELF file data and APIs provided by the user.
2. Utilization of library functionalities for ELF file analysis.
3. Utilization of library functionalities for the Manifest creation.

## Usage

To use the manifest-producer tool, you can run the following command from the command line:

```bash
cargo run <ELF_file_path> <JSON_file_path>
```

`<ELF_file_path>` represents the path to the ELF file intended for analysis, while `<JSON_file_path>` denotes the path to the JSON file containing the list of APIs.

## Dependencies

The project uses the following main dependencies:

- [capstone](https://crates.io/crates/capstone) - A disassembly framework with multiple architectures support.
- [goblin](https://crates.io/crates/goblin) - A crate for handling PE, ELF, and Mach-O binaries.
- [serde_json](https://crates.io/crates/serde_json) - A JSON serialization and deserialization library for Rust.
- [thiserror](https://crates.io/crates/thiserror) - A library for defining custom error types in Rust.
- [object](https://crates.io/crates/object) - A crate for working with object file formats.
- [cpp_demangle](https://crates.io/crates/cpp_demangle) - A demangler for C++ symbols.
- [gimli](https://crates.io/crates/gimli) - A library for working with the DWARF debugging format.
- [memmap2](https://crates.io/crates/memmap2) - A safe and easy-to-use wrapper around platform memory-mapped I/O APIs.
- [rustc-demangle](https://crates.io/crates/rustc-demangle) - A demangler for Rust symbols.

## License

Released under the [MIT License](LICENSES/MIT.txt)

<!-- Links -->
[actions]: https://github.com/SoftengPoliTo/prin-task-2.2/actions
[codecov]: https://app.codecov.io/gh/SoftengPoliTo/prin-task-2.2
[license]: LICENSES/MIT.txt
[status]: https://deps.rs/repo/github/SoftengPoliTo/prin-task-2.2

<!-- Badges -->
[actions badge]: https://github.com/SoftengPoliTo/prin-task-2.2/workflows/manifest-producer/badge.svg
[codecov badge]: https://codecov.io/gh/SoftengPoliTo/prin-task-2.2/branch/main/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
[status badge]: https://deps.rs/repo/github/SoftengPoliTo/prin-task-2.2/status.svg
