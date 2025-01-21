# behaviours_assessment Tool

[![LICENSE][license badge]][license]

## Description

The `behaviours_assessment` tool serves as both an **example implementation** and a **practical application** of the `manifest-producer` library. It demonstrates the integration of the **backend** and **frontend** crates to analyze ELF binaries. This tool can also be used as a standalone utility for conducting **in-depth binary analysis**, offering significant support for static reverse engineering practices.

Designed for certifiers, developers, and researchers, the tool facilitates understanding the internal logic of an ELF binary by:
- Extracting structural and behavioral information.
- Generating detailed HTML reports with interactive call trees.
- Providing JSON outputs that represent function interactions and disassembly results.

## Features

- **ELF Analysis**: Performs static analysis of ELF binaries to extract function call trees, disassembly, and structural insights.
- **Interactive Reports**: Generates user-friendly HTML reports to explore the analyzed data.
- **JSON Outputs**: Exports structured JSON files for deeper custom processing or integration with other tools.
- **Modular Design**: Leverages the `manifest-producer-backend` and `manifest-producer-frontend` crates.
- **Reverse Engineering Support**: Helps reverse engineers to decode binary logic by analyzing dependencies, function calls, and symbol demangling.

## Requirements

Ensure you have the following installed:

- **Rust**: Version 1.50 or higher. Install from [Rust's official website](https://www.rust-lang.org/tools/install).
- **cargo**: Rust's package manager, included with Rust.

## Usage

Follow these steps to analyze an ELF binary:

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/SoftengPoliTo/manifest-producer.git
   cd manifest-producer/example/behaviours_assessment
   ```
2. **Run the Tool:** Use the following command to analyze your ELF binary:
   ```bash
   cargo run <path_to_ELF_binary>
   ```
3. **Outputs:** 
    - **HTML Report:** An interactive report, showcasing function call graphs, disassembly, and more.
    - **JSON Files:** Containing structured data for function interactions and analysis results.

## Reverse Engineering Use Case

The `behaviours_assessment` is particularly valuable for **static reverse engineering** workflows:

- **Understand Functionality:** Explore the relationships between functions and their dependencies.
- **Disassembly Analysis:** Review low-level assembly code to infer logic.
- **Symbol Demangling:** Decode function names from languages like Rust and C++ for clarity.

These features help analysts efficiently deduce the behavior and intent of an ELF binary.

<!-- Links -->
[license]: LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg