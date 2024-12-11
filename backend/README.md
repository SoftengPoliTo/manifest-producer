# manifest_producer-backend

## Index
- [Description](#description)
- [Crate structure](#crate-structure)
- [Dependencies](#dependencies)


## Description
The backend crate is intended for analysing and inspecting ELF binaries, with advanced features for feature detection, disassembly, and function relationship analysis. Designed for certifiers, developers, researchers and security analysts, this crate leverages libraries such as Goblin, Capstone, and Gimli to provide a versatile suite of tools for understanding the behaviour of binary files, both at the structure and execution level. and to compare the results of the analysis with the manufacturer's statements.


## Crate Structure

Here is a summary of the crate structure:
```
backend/
├── Cargo.toml
└── src/
    ├── lib.rs        
    ├── analyse.rs 
    ├── entry.rs 
    ├── inspect.rs 
    ├── detect.rs
    └── error.rs
```

### Component details:

  - **backend/**: 
  - **Cargo.toml**: Specifies the necessary dependencies for the backend modules.
  - **lib.rs**: Main backend module, which integrates the various analysis functionalities and structures.
  - **analyse.rs**: It contains the main logic for analysing and disassembling functions. Includes:
    - Disassembly of ELF binaries using `Capstone`.
    - Management of function structure via `FunctionNode`.
    - Export of results in JSON format.
  - **detect.rs**: It deals with:
    - Detect and demangulate function names in ELF binaries.
    - Map symbols to their respective addresses and attributes.
    - Support languages such as Rust and C++ for decoding function names.
  - **entry.rs**: Identifies root nodes for the representation of dependencies between functions. Other features:
    - Computes the number of function invocations.
    - Filters functions from irrelevant libraries such as `musl` or `libc`.
  - **inspect.rs**: Provides APIs for:
    - Inspect an ELF binary (e.g. architecture target, file type, PIE).
    - Determine the predominant source language.
    - Examine sections such as .text for code disassembly.

## Dependencies

The project uses the following main dependencies:

- [serde_json](https://crates.io/crates/serde_json) - A JSON serialization and deserialization library for Rust.
- [goblin](https://crates.io/crates/goblin) - A crate for handling PE, ELF, and Mach-O binaries.
- [object](https://crates.io/crates/object) - A crate for working with object file formats.
- [memmap2](https://crates.io/crates/memmap2) - A safe and easy-to-use wrapper around platform memory-mapped I/O APIs.
- [gimli](https://crates.io/crates/gimli) - A library for working with the DWARF debugging format.
- [rustc-demangle](https://crates.io/crates/rustc-demangle) - A demangler for Rust symbols.
- [cpp_demangle](https://crates.io/crates/cpp_demangle) - A demangler for C++ symbols.
- [capstone](https://crates.io/crates/capstone) - A disassembly framework with multiple architectures support.
- [serde](https://crates.io/crates/serde) - A framework for serializing and deserializing Rust data structures.
- [indicatif](https://crates.io/crates/indicatif) - A library for building progress bars and spinners in Rust.

