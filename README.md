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
├── example/behaviours_assessment/
│   ├── Cargo.toml  
│   ├── README.md        
│   └── src/
│       ├── analysis.rs
│       ├── cli.rs
│       ├── dirs.rs
│       ├── error.rs
│       └── main.rs 
├── backend/
│      ├── Cargo.toml
│      ├── README.md
│      ├── tests/
│      │   ├── integration_test.rs     
│      │   ├── snapshot/
│      │   └── assets/ 
│      └── src/
│        ├── lib.rs        
│        ├── analyse.rs 
│        ├── entry.rs 
│        ├── inspect.rs 
│        ├── detect.rs
│        └── error.rs
└── frontend/
      ├── Cargo.toml 
      ├── README.md 
      ├── tests/
      │    ├── integration_test.rs     
      │    └── snapshot/ 
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
cargo run -p manifest-producer <path_for_your_ELF_binary> [ -o <path_for_your_results_folder>]
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


## Contacts

For questions, issues, or contributions, feel free to reach out:

- **Project Lead**: Giuseppe Marco Bianco  
  - Email: [giuseppemarco.bianco@libero.it](mailto:giuseppemarco.bianco@libero.it)
  - GitHub: [giusbianco](https://github.com/giusbianco)
- **Repository**: [manifest-producer on GitHub](https://github.com/SoftengPoliTo/manifest-producer)

Contributions are welcome! If you find a bug, have suggestions, or would like to collaborate, please open an issue or submit a pull request.


<!-- Links -->
[actions]: https://github.com/SoftengPoliTo/manifest-producer/actions
[codecov]: https://app.codecov.io/gh/SoftengPoliTo/manifest-producer
[license]: LICENSE-MIT
[status]: https://deps.rs/repo/github/SoftengPoliTo/manifest-producer

<!-- Badges -->
[actions badge]: https://github.com/SoftengPoliTo/manifest-producer/workflows/manifest-producer/badge.svg
[codecov badge]: https://codecov.io/gh/SoftengPoliTo/manifest-producer/branch/main/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
[status badge]: https://deps.rs/repo/github/SoftengPoliTo/manifest-producer/status.svg
