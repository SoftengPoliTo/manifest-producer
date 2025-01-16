# manifest_producer-backend

[![LICENSE][license badge]][license]

## Description
The backend crate analyzes ELF binaries, focusing on feature detection, disassembly, and function relationship analysis. It provides tools for understanding binary structures and behavior.

## Integration tests
The backend crate includes integration tests to validate binary analysis results. These tests compare JSON outputs and function identifiers with saved snapshots, ensuring consistency.

To run the tests:
```bash
cargo insta test
```

## Dependencies
Key dependencies:
- [serde_json](https://crates.io/crates/serde_json) - A JSON serialization and deserialization library for Rust.
- [goblin](https://crates.io/crates/goblin) - A crate for handling PE, ELF, and Mach-O binaries.
- [capstone](https://crates.io/crates/capstone) - A disassembly framework with multiple architectures support.

<!-- Links -->
[license]: LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg