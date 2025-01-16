# behaviours_assessment Tool

[![LICENSE][license badge]][license]

![Demo](../../data/demo.gif)
*Demo of the tool analyzing an ELF binary and generating HTML reports.*
The results of the demo are visible in the results folder in example.

## Description
The `behaviours_assessment` tool demonstrates how to use the `manifest-producer` library to analyze ELF binaries. It generates HTML reports with interactive call trees and structured JSON outputs for function analysis.

## Features
- **ELF Analysis**: Static analysis of ELF binaries, extracting function call trees and disassembly.
- **Interactive Reports**: HTML reports showcasing function relationships and analysis.
- **JSON Outputs**: Structured data for deeper analysis or integration.
- **Reverse Engineering Support**: Decode function relationships and assembly to understand binary logic.


## Requirements
Ensure you have the following installed:

- **Rust 1.50+** and **cargo** package manager.
   (If not already installed, please follow the [Rust installation guide](https://www.rust-lang.org/tools/install).)


## Usage
1. Clone the repository:
   ```bash
   git clone https://github.com/SoftengPoliTo/manifest-producer.git
   cd manifest-producer/examples/behaviours_assessment
   ```
2. Run the Tool:
   ```bash
   cargo run <path_to_ELF_binary>
   ```
3. Outputs: 
    - **HTML Report:** An interactive report, showcasing function call graphs, disassembly code, and more.
    - **JSON Files:** Containing structured data for function interactions and analysis results.

### Reverse Engineering Use Case
This tool is for static reverse engineering. It helps inspect elf binaries by analyzing dependencies, symbol names, and function interactions, making it easier to understand complex binaries and their inner workings.

<!-- Links -->
[license]: LICENSE-MIT

<!-- Badges -->
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg