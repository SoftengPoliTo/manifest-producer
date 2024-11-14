# manifest_producer-frontend

## Index
- [Description](#description)
- [Crate structure](#crate-structure)
- [Dependencies](#dependencies)


## Description
The `frontend` crate serves as the visual and structural layer for rendering analysis results. It combines HTML templates, CSS styles, JavaScript functionality, and Rust-powered generation logic to produce a coherent and interactive user interface. The crate is modular, with dedicated modules for creating call trees, subtrees, and dynamically generated HTML content based on analyzed data.

## Crate Structure

Here is a summary of the crate structure:
```
frontend/
  ├── Cargo.toml  
  └── src/
      ├── templates/ 
      │   ├── index.html
      │   ├── functions_list.html
      │   ├── disassembly_view.html
      │   ├── root_functions.html
      │   ├── call_graph.html
      │   ├── css/
      │   │   └── style.css
      │   └── js/
      │       └── script.js
      ├── lib.rs 
      ├── html_generator.rs 
      ├── subtrees_generator.rs 
      └── tree_generator.rs 
```

### Component details:

  - **frontend/**: 
  - **Cargo.toml**: Configuration and dependencies for frontend modules.
  - **templates/**: Contains all HTML templates used to present the results of the analysis.
  - **html_generator.rs**: Module that takes care of the generation of the HTML content, assembling the analysed information into a readable format.
  - **tree_generator.rs**: Responsible for creating the call tree, visually representing the relationships between the analysed functions.
  - **subtrees_generator.rs**: Responsible for the creation of subtrees, aimed at simplifying the process of creating the tree itself.


## Dependencies

The project uses the following main dependencies:

- [thiserror](https://crates.io/crates/thiserror) - A library for defining custom error types in Rust.
- [serde_json](https://crates.io/crates/serde_json) - A JSON serialization and deserialization library for Rust.
- [serde](https://crates.io/crates/serde) - A framework for serializing and deserializing Rust data structures.
- [indicatif](https://crates.io/crates/indicatif) - A library for building progress bars and spinners in Rust.
- [minijinja](https://crates.io/crates/minijinja) - A fast and extensible templating engine for Rust.


