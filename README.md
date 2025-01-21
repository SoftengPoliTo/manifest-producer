# manifest-producer

[![Actions Status][actions badge]][actions]
[![CodeCov][codecov badge]][codecov]
[![LICENSE][license badge]][license]

## Index
- [Description](#description)
- [Use](#use)
- [License](#license)

## Description

**manifest-producer** is a project that provides a backend and frontend library for the analysis of ELF binaries. The main goal is to gather as much information as possible on ELF binaries and create call trees for functions that the tool considers to be potential entry points for possible executables. 

All collected data is presented in a user-friendly manner, generating a series of HTML templates displaying the information obtained from the analysis. This tool aims to provide an in-depth analysis of the behaviour of ELF binaries and to compare the results of the analysis with the manufacturer's statements.

## Use

**manifest-producer** is designed primarily as a library for the static reverse engineering of ELF binaries, but also includes a practical example of use.

### Requirements

Make sure you have [Rust](https://www.rust-lang.org/tools/install) (version 1.50 or higher) and `cargo`, Rust's package manager, installed.

### Execution of the example 

```bash
cargo run -p manifest-producer <path_for_your_ELF_binary>
```

This command analyses the specified ELF binary and generates the corresponding output, which will be automatically opened in the system's default browser for display. In addition, the structures used to create the trees will be saved in json format. The overall results will be available in the `public/` folder.

## License

The **manifest-producer** project is distributed under the MIT licence. This means that you may use, copy, modify, merge, publish, distribute, sublicence and/or sell copies of the software, provided that you include the following copyright notice and permission notice in your work.

For further details on the licence, see the [LICENSE](LICENSE-MIT) file in the repository.

<!-- Links -->
[actions]: https://github.com/SoftengPoliTo/manifest-producer/actions
[codecov]: https://app.codecov.io/gh/SoftengPoliTo/manifest-producer
[license]: LICENSE-MIT

<!-- Badges -->
[actions badge]: https://github.com/SoftengPoliTo/manifest-producer/workflows/manifest-producer/badge.svg
[codecov badge]: https://codecov.io/gh/SoftengPoliTo/manifest-producer/branch/main/graph/badge.svg
[license badge]: https://img.shields.io/badge/license-MIT-blue.svg
