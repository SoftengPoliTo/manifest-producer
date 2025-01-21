# manifest_producer-backend

[![LICENSE][license badge]][license]

## Index

- [Description](#description)
- [Integration tests](#tests)
- [Dependencies](#dependencies)


## Description

The backend crate is intended for analysing and inspecting ELF binaries, with advanced features for feature detection, disassembly, and function relationship analysis. Designed for certifiers, developers, researchers and security analysts, this crate leverages libraries such as Goblin, Capstone, and Gimli to provide a versatile suite of tools for understanding the behaviour of binary files, both at the structure and execution level. and to compare the results of the analysis with the manufacturer's statements.

## Integration tests

The crate backend includes a suite of integration tests to ensure the proper integration of the various modules. These tests run on various binaries compiled in C, C++ and Rust, and compare the results with previously generated snapshots to validate the consistency of the data produced.

The binaries used for testing are contained in the `tests/assets` directory and are example binaries written in C, C++ and Rust. These are analysed to extract information such as the functions defined in the binary and the root nodes. The outputs are then compared with the reference data saved in the `tests/snapshots` directory.

*The main integration tests are as follows:
- *test_c*: Analyses a static C binary.
- *test_cpp*: Analyses a static C++ binary.
- *test_rust*: Analyses a Rust binary.

To run the tests, simply use the command:

```bash
cargo insta test 
```

In particular, the tests verify that:
- The JSON files generated for the binary details are correct.
- The functions identified in the binary are consistent with those in the previous tests.
- Extracted root nodes are aligned with the reference data.

In the event of failure due to differences detected in comparison with previous snapshots, the following command allows you to view them one by one and decide whether to accept the changes or reject them:

```bash
cargo insta review
```

Integration tests are crucial to ensure that the system continues to function as intended even with changes to components or input binaries.

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

<!-- Links -->
[license]: LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg