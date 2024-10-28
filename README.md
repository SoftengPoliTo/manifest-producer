# manifest-producer

[![Actions Status][actions badge]][actions]
[![CodeCov][codecov badge]][codecov]
[![LICENSE][license badge]][license]
[![dependency status][status badge]][status]

## Index
- [Description](#description)
- [Project structure](#project-structure)
- [Use](#use)
- [Contributions](#contributions)
- [License](#license)
- [Contacts](#contacts)


## Description
**manifest-producer** is a project that provides a backend and frontend library for the analysis of ELF binaries. The main goal is to gather as much information as possible on ELF binaries and create call trees for functions that the tool considers to be potential entry points for possible executables. 

All collected data is presented in a user-friendly manner, generating a series of HTML templates displaying the information obtained from the analysis. This tool aims to provide an in-depth analysis of the behaviour of ELF binaries and to compare the results of the analysis with the manufacturer's statements.


## Project Structure

The **manifest-producer** project is organised in a workspace structure of Cargo, consisting of several packages (crates) that cooperate with each other. Here is a summary of the project structure:
```
manifest-producer/
├── Cargo.toml      
├── public/  
    ├── call_graphs
    ├── json
    └── *.html
├── example/
│   ├── Cargo.toml          
│   └── src/
│       └── main.rs 
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs        
│       ├── api_analyzer.rs 
│       ├── elf_analyzer.rs 
│       └── func_analyzer.rs
├── common/
│   ├── Cargo.toml   
│   └── src/
│       ├── lib.rs      
│       └── error.rs  
└── frontend/
    ├── Cargo.toml  
    └── src/
        ├── templates/ 
            ├── index.html
            ├── functions_list.html
            ├── disassembly_view.html
            ├── root_functions.html
            ├── call_graph.html
            ├── css/
                ├── style.css
            ├── js/
                ├── script.js
        ├── lib.rs 
        ├── html_generator.rs 
        └── tree_generator.rs 
```

### Component details:

- **example/**: 
  - **Cargo.toml**: Specifies dependencies and configurations for the main application. This can be used as an example of library usage.
  - **src/main.rs**: Application entry point, where the analysis of ELF binaries and report generation is initiated.

  - **backend/**: 
  - **Cargo.toml**: Specifies the necessary dependencies for the backend modules.
  - **src/lib.rs**: Main backend module, which integrates the various analysis functionalities.
  - **api_analyzer.rs**: Contains the logic for the API analysis of ELF binaries, identifying functions and entry points.
  - **elf_analyzer.rs**: Module dedicated to the analysis of ELF structures, including information extraction and management of binaries.
  - **func_analyzer.rs**: Deals with the analysis of functions within binaries, to determine their behaviour and interactions.

  - **common/**: 
  - **Cargo.toml**: Contains the dependencies shared between the various modules.
  - **src/lib.rs**: Main module for common data structures and functionality used throughout the project.
  - **error.rs**: Manages the definition and handling of errors for the project, enabling centralised and consistent error handling.

  - **frontend/**: 
  - **Cargo.toml**: Configuration and dependencies for frontend modules.
  - **src/templates/**: Contains all HTML templates used to present the results of the analysis.
  - **html_generator.rs**: Module that takes care of the generation of the HTML content, assembling the analysed information into a readable format.
  - **tree_generator.rs**: Responsible for creating the call tree, visually representing the relationships between the analysed functions.


## Use

**manifest-producer** is designed primarily as a library for the analysis of ELF binaries, but also includes a practical example of use.

### Requirements

Make sure you have [Rust](https://www.rust-lang.org/tools/install) (version 1.50 or higher) and `cargo`, Rust's package manager, installed.

### Cloning the Repository

Clone the repository using Git:

```bash
git clone https://github.com/SoftengPoliTo/manifest-producer.git
cd manifest-producer
```

### Execution of the example 
```bash
cargo run -p manifest-producer <path_for_your_ELF_binary>
```

This command analyses the specified ELF binary and generates the corresponding output, which will be automatically opened in the system's default browser for display. In addition, the structures used to create the trees will be saved in json format. The overall results will be available in the `public/` folder.


## Contributions

Contributions to the manifest-producer project are always welcome! If you wish to contribute, please follow these simple steps:

### How to Contribute
1. **Fork the Repository**: Start by fork the repository on GitHub to get your own copy to work on.

2. **Create a Branch**: Create a new branch for your feature or bug fix. It is recommended to follow the naming convention feature/name-feature or bugfix/name-bug.
```bash
git checkout -b feature/name-feature
```

3. **Implement Your Change**: Make your changes and be sure to test them. If you are adding new functionality, try to also include unit tests to ensure the quality of the code.

4. **Commit Changes**: commit your changes with a clear and descriptive message.
```bash
git commit -m "Something here"
```

5. **Branch Push**: Push your branch to your repository fork.
```bash
git push origin feature/name-feature
```

6. **Create a Pull Request**: Go to your fork on GitHub and click on ‘New Pull Request’. Follow the instructions to send your pull request to the main project.

### Guidelines
* Ensure that the code is formatted according to Rust standards.
* Follow the coding conventions of the project.
* Be open to feedback and discussion on your pull request.

Thank you for your interest and support of the `manifest-producer` project!


## License

The **manifest-producer** project is distributed under the MIT licence. This means that you may use, copy, modify, merge, publish, distribute, sublicence and/or sell copies of the software, provided that you include the following copyright notice and permission notice in your work.

### Copyright Notice

```plaintext
Copyright (c) 2023 Giuseppe Marco Bianco, manifest-producer contributors
```

### Authorisation Note

For further details on the licence, see the [LICENSE](LICENSE) file in the repository.



## Dependencies

The project uses the following main dependencies:

- [thiserror](https://crates.io/crates/thiserror) - A library for defining custom error types in Rust.
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
- [open](https://crates.io/crates/open) - A simple way to open files in the default application.
- [minijinja](https://crates.io/crates/minijinja) - A fast and extensible templating engine for Rust.


## Contacts

For questions, issues, or contributions, feel free to reach out:

- **Project Lead**: Giuseppe Marco Bianco  
  - Email: [giuseppe.bianco1@uniurb.it](mailto:giuseppe.bianco1@uniurb.it)
  - GitHub: [giusbianco](https://github.com/giusbianco)
- **Repository**: [manifest-producer on GitHub](https://github.com/SoftengPoliTo/manifest-producer)

Contributions are welcome! If you find a bug, have suggestions, or would like to collaborate, please open an issue or submit a pull request.


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
