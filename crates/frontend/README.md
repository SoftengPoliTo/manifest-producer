# manifest_producer-frontend

[![LICENSE][license badge]][license]

## Index

- [Description](#description)
- [integration tests](#tests)
- [Dependencies](#dependencies)


## Description

The `frontend` crate serves as the visual and structural layer for rendering analysis results. It combines HTML templates, CSS styles, JavaScript functionality, and Rust-powered generation logic to produce a coherent and interactive user interface. The crate is modular, with dedicated modules for creating call trees, subtrees, and dynamically generated HTML content based on analyzed data.

## Integration tests

The frontend crate includes integration tests that verify the correct generation of HTML files from the analysed data. These tests compare the generated HTML output with the saved snapshots to ensure that any changes to the frontend do not introduce regressions in the visual results.

The integration tests can be found in the file `tests/integration_tests.rs`. Here, a sample binary, together with a set of nodes and function disassemblies, is passed to the frontend HTML generator. The generated output is then compared with the snapshots previously recorded in the `tests/snapshots` directory of the crate.

The main test, `run_frontend_test`, deals with:
- Creating a data structure that simulates a binary with functions and disassemblies.
- Pass this data to the frontend HTML generator.
- Compare the result with the stored snapshots to check for correctness.

To run tests, use the command:

```bash
cargo insta test
```

The comparison of the generated HTML output is done via the compare_generated_html function, which uses the insta framework to verify that each generated HTML file is equal to the reference snapshots. Any changes to the HTML files in the frontend are then automatically tested for unintended changes.
These tests are essential to ensure that the user interface remains consistent and that the information rendering functionality is correct, even during changes and improvements to the frontend code.

## Dependencies

The project uses the following main dependencies:

- [serde_json](https://crates.io/crates/serde_json) - A JSON serialization and deserialization library for Rust.
- [serde](https://crates.io/crates/serde) - A framework for serializing and deserializing Rust data structures.
- [indicatif](https://crates.io/crates/indicatif) - A library for building progress bars and spinners in Rust.
- [minijinja](https://crates.io/crates/minijinja) - A fast and extensible templating engine for Rust.

<!-- Links -->
[license]: LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg