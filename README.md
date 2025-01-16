# manifest-producer

[![Actions Status][actions badge]][actions]
[![CodeCov][codecov badge]][codecov]
[![LICENSE][license badge]][license]

## Description
**manifest-producer** is a Rust library designed for analysing ELF binaries. It provides functionality to extract detailed information from ELF files, including disassembly of individual function code, and function call trees for structural insights. 

## Supported Architectures
| Architecture | Support Level | Notes           |
| ------------ | ------------- | --------------- |
| x86_64       | ✅            | Supported       |
| ARM          | ❌            | Not supported   |
| RISC-V       | ❌            | Not supported   |

## Supported Languages

| Language | Support Level | Notes   |
| -------- | ------------- |-------- |
| C        | ✅            | Supported |
| C++      | ✅            | Supported |
| Rust     | ✅            | Supported |

## Project Structure

- **backend crate**: Contains the analysis logic for ELF binaries.
- **frontend crate**: Generates user-friendly HTML/JSON reports from the analysis data.
- **examples/behaviours_assessment**: A practical example tool that demonstrates how to use both the backend and frontend crates to analyze ELF binaries.

Inside the `examples/behaviours_assessment` folder, you'll find:
- A **demo video** showcasing the tool's functionality.
- The **HTML and JSON results** generated from analyzing an ELF binary.

## Use

### Requirements
Make sure you have [Rust](https://www.rust-lang.org/tools/install) (version 1.50 or higher) and `cargo` installed.

### Execute the Example
```bash
cargo run <path_to_your_ELF_binary>
```
This command analyzes the ELF binary and generates HTML reports along with JSON data for function interactions.

## License
Distributed under the MIT licence. See the [LICENSE](LICENSE-MIT) file for details.

<!-- Links -->
[actions]: https://github.com/SoftengPoliTo/manifest-producer/actions
[codecov]: https://app.codecov.io/gh/SoftengPoliTo/manifest-producer
[license]: LICENSE-MIT

<!-- Badges -->
[actions badge]: https://github.com/SoftengPoliTo/manifest-producer/workflows/manifest-producer/badge.svg
[codecov badge]: https://codecov.io/gh/SoftengPoliTo/manifest-producer/branch/main/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
