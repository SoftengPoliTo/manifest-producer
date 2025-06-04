use std::{fs, path::Path};

use colored::Colorize;
use goblin::elf::{
    dynamic::{DT_BIND_NOW, DT_NEEDED, DT_RPATH, DT_RUNPATH},
    header::{self, ET_CORE, ET_DYN, ET_REL},
    program_header::{PF_W, PF_X, PT_GNU_RELRO, PT_INTERP, PT_LOAD},
    section_header::{SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE},
    Elf,
};

use crate::{CategoryResult, ValidationReport, ValidationResult};

pub fn bad_magic_report(file_path: &Path) -> ValidationReport {
    let mut report = ValidationReport {
        binary_path: file_path.display().to_string(),
        categories: vec![],
    };
    let category = CategoryResult {
        name: "Basic Structural Validation".to_string(),
        description: "Validates the basic structure of the ELF file.".to_string(),
        checks: vec![ValidationResult {
            name: "Magic Number".to_string(),
            status: false,
            description:
                "Invalid magic number: file is likely not a valid ELF binary or is corrupted."
                    .to_string(),
            metadata: None,
        }],
    };
    report.categories.push(category);
    report
}

pub fn malformed_report(file_path: &Path, error: &str) -> ValidationReport {
    let mut report = ValidationReport {
        binary_path: file_path.display().to_string(),
        categories: vec![],
    };
    let category = CategoryResult {
        name: "Basic Structural Validation".to_string(),
        description: "Validates the basic structure of the ELF file.".to_string(),
        checks: vec![ValidationResult {
            name: "Malformed ELF".to_string(),
            status: false,
            description: format!("{:?}", error),
            metadata: None,
        }],
    };
    report.categories.push(category);
    report
}

pub fn validate_elf_file(elf: &Elf, file_path: &Path, file_size: u64) -> ValidationReport {
    let mut report = ValidationReport {
        binary_path: file_path.display().to_string(),
        categories: Vec::new(),
    };

    let basic_structure_check = validate_elf_basic_structure(elf, file_size);
    report.categories.push(basic_structure_check);

    let memory_mapping_check = validate_memory_mapping(elf);
    report.categories.push(memory_mapping_check);

    let security_mitigation_check = validate_security_mitigation(elf);
    report.categories.push(security_mitigation_check);

    let dependencies_check = validate_dependencies(elf, file_path);
    report.categories.push(dependencies_check);

    // Add more categories

    report
}

fn read_file(file_path: &Path) -> Vec<u8> {
    match fs::read(file_path) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            Vec::new()
        }
    }
}

pub fn display_cli_results(report: &ValidationReport) {
    println!(
        "\n{} {}",
        "📊".bold(),
        "ELF Integrity Checking:".bold().underline()
    );
    println!("📁 File: {}\n", report.binary_path.cyan());

    for category in &report.categories {
        println!("{} {}", "🔍".bold(), category.name.bold().yellow());

        for check in &category.checks {
            let status_icon = if check.status {
                "✅".green()
            } else {
                "❌".red()
            };

            println!(
                "  {} {}: {}",
                status_icon,
                check.name.bold(),
                check.description
            );
        }

        println!();
    }
}

pub fn json_results(report: &ValidationReport) {
    let json_report = serde_json::to_string_pretty(report).unwrap();
    let json_path = format!("{}.json", report.binary_path);
    fs::write(&json_path, &json_report).unwrap();
    println!("\nJSON report saved to: {}", json_path.cyan());
}

// ################## BASIC STRUCTURAL VALIDATION ##################
fn validate_elf_basic_structure(elf: &Elf, file_size: u64) -> CategoryResult {
    let mut category = CategoryResult {
        name: "Basic Structural Validation".to_string(),
        description: "Validates the basic structure of the ELF file.".to_string(), // More detailed description
        checks: Vec::new(),
    };

    let magic_number = check_magic_number(elf);
    category.checks.push(magic_number);

    let class = check_class(elf);
    category.checks.push(class);

    let data_encoding = check_data_encoding(elf);
    category.checks.push(data_encoding);

    let version = check_version(elf);
    category.checks.push(version);

    let data_encoding_consistency = check_data_encoding_consistency(elf, elf.header.e_machine);
    category.checks.push(data_encoding_consistency);

    let arch = elf.header.e_machine;
    let class_arch_consistency = check_class_arch_consistency(elf, arch);
    category.checks.push(class_arch_consistency);

    let type_consistency = check_type_consistency(elf);
    category.checks.push(type_consistency);

    let header_size_consistency = check_header_size_consistency(elf);
    category.checks.push(header_size_consistency);

    let ph_offset = check_ph_offset(elf);
    category.checks.push(ph_offset);

    let sh_offset = check_sh_offset(elf);
    category.checks.push(sh_offset);

    let ph_bound = check_ph_bound(elf, file_size);
    category.checks.push(ph_bound);

    let sh_bound = check_sh_bound(elf, file_size);
    category.checks.push(sh_bound);

    let shstrndx_check = check_valid_internal_references(elf);
    category.checks.push(shstrndx_check);

    // Add more checks here
    category
}

fn check_magic_number(elf: &Elf) -> ValidationResult {
    ValidationResult {
        name: "Magic Number".to_string(),
        status: true,
        description:
            "Valid ELF signature found (magic number OK). File is recognized as ELF format."
                .to_string(),
        metadata: Some(serde_json::json!({
            "expected": "[127, 69, 76, 70]",
            "found": format!("{:?}", &elf.header.e_ident[..4])
        })),
    }
}

fn check_class(elf: &Elf) -> ValidationResult {
    let class_str = if elf.is_64 { "64" } else { "32" };
    ValidationResult {
        name: "Class".to_string(),
        status: true,
        description: format!("ELF class is valid: detected {}-bit format. This affects how memory addresses are interpreted and may impact compatibility.", class_str),
        metadata: Some(serde_json::json!({
            "expected": "ELF32 or ELF64",
            "found": format!("ELF{}", class_str)
        })),
    }
}

fn check_data_encoding(elf: &Elf) -> ValidationResult {
    let encoding = match elf.header.e_ident[5] {
        1 => "little",
        2 => "big",
        _ => "unknown",
    };

    ValidationResult {
        name: "Data Encoding".to_string(),
        status: encoding != "unknown",
        description: if encoding != "unknown" {
            format!(
                "ELF data encoding is valid: {} endian format detected.",
                encoding
            )
        } else {
            "Invalid or unrecognized ELF data encoding. This could indicate corruption or unsupported architecture.".to_string()
        },
        metadata: Some(serde_json::json!({
            "expected": "little or big endian",
            "found": encoding
        })),
    }
}

fn check_version(elf: &Elf) -> ValidationResult {
    let version = elf.header.e_ident[6];
    ValidationResult {
        name: "Version".to_string(),
        status: version == 1,
        description: if version == 1 {
            "ELF version is valid (current standard).".to_string()
        } else {
            format!("Unsupported ELF version detected: {}.", version)
        },
        metadata: Some(serde_json::json!({
            "expected": 1,
            "found": version
        })),
    }
}

fn check_data_encoding_consistency(elf: &Elf, arch: u16) -> ValidationResult {
    let data_encoding = elf.header.e_ident[5];
    let encoding_str = data_encoding_to_string(data_encoding);
    let arch_str = arch_to_string(arch);
    let consistent = is_data_encoding_consistent(arch, data_encoding);

    ValidationResult {
        name: "Data Encoding Consistency".to_string(),
        status: consistent,
        description: if consistent {
            format!(
                "Data encoding is consistent with the detected architecture ({} endian on {}).",
                encoding_str, arch_str
            )
        } else {
            format!(
                "Mismatch between data encoding and architecture: {} endian on {} is unusual or unsupported.",
                encoding_str, arch_str
            )
        },
        metadata: Some(serde_json::json!({
            "data_encoding": encoding_str,
            "architecture": arch_str,
        })),
    }
}

fn is_data_encoding_consistent(arch: u16, data_encoding: u8) -> bool {
    // Typically little endian
    let little_endian_archs = [
        header::EM_386,
        header::EM_X86_64,
        header::EM_ARM,
        header::EM_AARCH64,
    ];

    // Typically big endian
    let big_endian_archs = [
        header::EM_SPARC,
        header::EM_SPARCV9,
        header::EM_PPC,
        header::EM_PPC64,
    ];

    match data_encoding {
        header::ELFDATA2LSB => {
            !big_endian_archs.contains(&arch) || little_endian_archs.contains(&arch)
        }
        header::ELFDATA2MSB => {
            !little_endian_archs.contains(&arch) || big_endian_archs.contains(&arch)
        }
        _ => false, // Invalid data encoding
    }
}

fn data_encoding_to_string(data_encoding: u8) -> String {
    match data_encoding {
        header::ELFDATA2LSB => "Little Endian".to_string(),
        header::ELFDATA2MSB => "Big Endian".to_string(),
        header::ELFDATANONE => "No data encoding".to_string(),
        _ => format!("Unknown data encoding ({})", data_encoding),
    }
}

fn arch_to_string(arch: u16) -> String {
    match arch {
        header::EM_386 => "Intel 80386 (i386)".to_string(),
        header::EM_X86_64 => "AMD x86-64".to_string(),
        header::EM_ARM => "ARM".to_string(),
        header::EM_AARCH64 => "ARM 64-bit (AArch64)".to_string(),
        header::EM_PPC => "PowerPC".to_string(),
        header::EM_PPC64 => "PowerPC 64-bit".to_string(),
        header::EM_SPARC => "SPARC".to_string(),
        header::EM_SPARCV9 => "SPARC 64-bit (v9)".to_string(),
        header::EM_MIPS => "MIPS".to_string(),
        _ => format!("Unknown architecture ({})", arch),
    }
}

fn check_class_arch_consistency(elf: &Elf, arch: u16) -> ValidationResult {
    let is_64 = elf.is_64;
    let class_str = class_to_string(is_64);
    let arch_str = arch_to_string(arch);
    let consistent = is_class_arch_consistent(arch, is_64);

    ValidationResult {
        name: "Class and Architecture Consistency".to_string(),
        status: consistent,
        description: if consistent {
            format!(
                "ELF class matches the target architecture ({} on {}).",
                class_str, arch_str
            )
        } else {
            format!(
                "Mismatch between ELF class and target architecture: {} on {} is likely invalid.",
                class_str, arch_str
            )
        },
        metadata: Some(serde_json::json!({
            "class": class_str,
            "architecture": arch_str,
        })),
    }
}

fn is_class_arch_consistent(arch: u16, is_64: bool) -> bool {
    match arch {
        // Typical 64-bit architectures
        header::EM_X86_64 | header::EM_AARCH64 | header::EM_PPC64 | header::EM_SPARCV9 => is_64,
        // Typical 32-bit architectures
        header::EM_386 | header::EM_ARM | header::EM_PPC | header::EM_SPARC => !is_64,
        // For other architectures
        _ => true,
    }
}

fn class_to_string(is_64: bool) -> String {
    if is_64 {
        "64-bit".to_string()
    } else {
        "32-bit".to_string()
    }
}

fn check_type_consistency(elf: &Elf) -> ValidationResult {
    use goblin::elf::header;

    let file_type = elf.header.e_type;
    let has_entry_point = elf.header.e_entry != 0;
    let has_loadable_segments = elf
        .program_headers
        .iter()
        .any(|ph| ph.p_type == goblin::elf::program_header::PT_LOAD);

    let (status, description) = match file_type {
        header::ET_EXEC => {
            if has_entry_point && has_loadable_segments {
                (
                    true,
                    "Valid executable: has entry point and loadable segments.".to_string(),
                )
            } else if !has_entry_point {
                (
                    false,
                    "Invalid executable: missing entry point.".to_string(),
                )
            } else {
                (
                    false,
                    "Invalid executable: no loadable segments found.".to_string(),
                )
            }
        }
        header::ET_DYN => {
            if has_loadable_segments {
                (
                    true,
                    "Valid shared object: loadable segments present.".to_string(),
                )
            } else {
                (
                    false,
                    "Invalid shared object: no loadable segments found.".to_string(),
                )
            }
        }
        header::ET_REL => {
            if !has_entry_point {
                (
                    true,
                    "Valid relocatable file: no entry point as expected.".to_string(),
                )
            } else {
                (
                    false,
                    "Unexpected entry point in relocatable file.".to_string(),
                )
            }
        }
        header::ET_CORE => (true, "Core dump file.".to_string()),
        _ => (
            false,
            format!("Unknown or unsupported file type (type ID: {}).", file_type),
        ),
    };

    ValidationResult {
        name: "File Type Consistency".to_string(),
        status,
        description,
        metadata: Some(serde_json::json!({
            "file_type": file_type,
            "has_entry_point": has_entry_point,
            "has_loadable_segments": has_loadable_segments
        })),
    }
}

fn check_header_size_consistency(elf: &Elf) -> ValidationResult {
    let header_size = elf.header.e_ehsize;
    let expected_ehsize = if elf.is_64 { 64 } else { 52 };
    let class_str = if elf.is_64 { "64" } else { "32" };

    let is_valid = header_size == expected_ehsize;

    ValidationResult {
        name: "Header Size Consistency".to_string(),
        status: is_valid,
        description: if is_valid {
            format!(
                "ELF header size is valid for a {}-bit file ({} bytes).",
                class_str, header_size
            )
        } else {
            format!(
                "Invalid ELF header size ({} bytes) for a {}-bit file; expected {} bytes.",
                header_size, class_str, expected_ehsize
            )
        },
        metadata: Some(serde_json::json!({
            "header_size": header_size,
            "expected_header_size": expected_ehsize,
            "class": format!("{}-bit", class_str)
        })),
    }
}

fn check_ph_offset(elf: &Elf) -> ValidationResult {
    ValidationResult {
        name: "Program Header Offset".to_string(),
        status: elf.header.e_phoff >= elf.header.e_ehsize as u64,
        description: if elf.header.e_phoff >= elf.header.e_ehsize as u64 {
            format!(
                "Program header offset ({}) is valid: it is correctly positioned after the ELF header (>= {}).",
                elf.header.e_phoff, elf.header.e_ehsize
            )
        } else {
            format!(
                "Program header offset ({}) is invalid: it should be at least {} to ensure proper alignment of the program header in the ELF file. The current value is below the expected minimum offset.",
                elf.header.e_phoff, elf.header.e_ehsize
            )
        },
        metadata: Some(serde_json::json!({
            "program_header_offset": elf.header.e_phoff,
            "expected_minimum_offset": elf.header.e_ehsize
        })),
    }
}

fn check_sh_offset(elf: &Elf) -> ValidationResult {
    ValidationResult {
        name: "Section Header Offset".to_string(),
        status: elf.header.e_shoff >= elf.header.e_ehsize as u64,
        description: if elf.header.e_shoff >= elf.header.e_ehsize as u64 {
            format!(
                "Section header offset ({}) is valid: it is correctly positioned after the ELF header (>= {}).",
                elf.header.e_shoff, elf.header.e_ehsize
            )
        } else {
            format!(
                "Section header offset ({}) is invalid: it should be at least {} to ensure proper alignment of the section headers in the ELF file. The current value is below the expected minimum offset.",
                elf.header.e_shoff, elf.header.e_ehsize
            )
        },
        metadata: Some(serde_json::json!({
            "section_header_offset": elf.header.e_shoff,
            "expected_minimum_offset": elf.header.e_ehsize
        })),
    }
}

fn check_ph_bound(elf: &Elf, file_size: u64) -> ValidationResult {
    let ph_offset = elf.header.e_phoff;
    let ph_table_size = elf.header.e_phnum as u64 * elf.header.e_phentsize as u64;
    let ph_table_end = if ph_offset > 0 {
        ph_offset + ph_table_size
    } else {
        0
    };

    ValidationResult {
        name: "Program Header Table Bound".to_string(),
        status: ph_offset == 0 || ph_table_end <= file_size,
        description: if ph_offset == 0 || ph_table_end <= file_size {
            format!(
                "Program header table is within file bounds: it ends at offset {} which is <= the file size ({}).",
                ph_table_end, file_size
            )
        } else {
            format!(
                "Program header table exceeds file bounds: it ends at offset {} which is > the file size ({}).",
                ph_table_end, file_size
            )
        },
        metadata: Some(serde_json::json!({
            "program_header_offset": ph_offset,
            "program_header_table_size": ph_table_size,
            "file_size": file_size,
            "program_header_table_end": ph_table_end
        })),
    }
}

fn check_sh_bound(elf: &Elf, file_size: u64) -> ValidationResult {
    let sh_offset = elf.header.e_shoff;
    let sh_table_size = elf.header.e_shnum as u64 * elf.header.e_shentsize as u64;
    let sh_table_end = if sh_offset > 0 {
        sh_offset + sh_table_size
    } else {
        0
    };

    ValidationResult {
        name: "Section Header Table Bound".to_string(),
        status: sh_offset == 0 || sh_table_end <= file_size,
        description: if sh_offset == 0 || sh_table_end <= file_size {
            format!(
                "Section header table is within file bounds: it ends at offset {} which is <= the file size ({}).",
                sh_table_end, file_size
            )
        } else {
            format!(
                "Section header table exceeds file bounds: it ends at offset {} which is > the file size ({}).",
                sh_table_end, file_size
            )
        },
        metadata: Some(serde_json::json!({
            "section_header_offset": sh_offset,
            "section_header_table_size": sh_table_size,
            "file_size": file_size,
            "section_header_table_end": sh_table_end
        })),
    }
}

fn check_valid_internal_references(elf: &Elf) -> ValidationResult {
    let sh_str_idx = elf.header.e_shstrndx as usize;
    ValidationResult {
        name: "String section index".to_string(),
        status: sh_str_idx < elf.section_headers.len() || sh_str_idx == 0,
        description: if sh_str_idx < elf.section_headers.len() || sh_str_idx == 0 {
            "The string section index is valid: it points to an existing section or is set to 0 (indicating no string table).".to_string()
        } else {
            format!(
                "Invalid string section index: {} is out of bounds for the section table ({} sections). The index should refer to a valid section or be 0.",
                sh_str_idx,
                elf.section_headers.len()
            )
        },
        metadata: Some(serde_json::json!({
            "shstrndx": sh_str_idx,
            "section_count": elf.section_headers.len()
        })),
    }
}

// ################ PROTECTION MECHANISMS AND HARDENING ##################
fn validate_security_mitigation(elf: &Elf) -> CategoryResult {
    let mut category = CategoryResult {
        name: "Protection Mechanisms and Hardening".to_string(),
        description: "Evaluates the implementation of countermeasures and protections designed to mitigate the most common attack vectors.".to_string(),
        checks: Vec::new(),
    };

    let nx_protection = check_nx_protection(elf);
    category.checks.push(nx_protection);

    let rerlo_protection = check_rerlo_protection(elf);
    category.checks.push(rerlo_protection);

    let canary_protection = check_stack_canary(elf);
    category.checks.push(canary_protection);

    let pie_protection = check_pie_protection(elf);
    category.checks.push(pie_protection);

    let wx_protection = check_wx_segments(elf);
    category.checks.push(wx_protection);

    let suspicious_entry_point = check_suspicious_entry_point(elf);
    category.checks.push(suspicious_entry_point);

    let isolated_executable_sections = check_isolated_executable_sections(elf);
    category.checks.push(isolated_executable_sections);

    category
}

fn check_nx_protection(elf: &Elf) -> ValidationResult {
    for ph in &elf.program_headers {
        if (ph.p_flags & PF_W) != 0 && (ph.p_flags & PF_X) != 0 {
            return ValidationResult {
                name: "NX Protection".to_string(),
                status: false,
                description: "The ELF file contains executable and writable segments, which violates NX (No eXecute) protection. This configuration can lead to security vulnerabilities like buffer overflow exploits.".to_string(),
                metadata: Some(serde_json::json!({
                    "segment_type": ph.p_type,
                    "flags": ph.p_flags
                })),
            };
        }
    }
    ValidationResult {
        name: "NX Protection".to_string(),
        status: true,
        description: "The ELF file has non-executable segments, ensuring NX protection is in place and enhancing security.".to_string(),
        metadata: None,
    }
}

fn check_rerlo_protection(elf: &Elf) -> ValidationResult {
    let mut has_rerlo = false;
    let mut has_bind_now = false;

    // Check for RERLO protection
    for ph in &elf.program_headers {
        if ph.p_type == PT_GNU_RELRO {
            has_rerlo = true;
            break;
        }
    }

    // Check for BIND_NOW protection
    if let Some(dynamic) = &elf.dynamic {
        for dyna in &dynamic.dyns {
            if dyna.d_tag == DT_BIND_NOW {
                has_bind_now = true;
                break;
            }
        }
    }

    match (has_rerlo, has_bind_now) {
        (true, true) => ValidationResult {
            name: "RERLO Protection".to_string(),
            status: true,
            description: "The ELF file has both RERLO and BIND_NOW protections enabled, which enhances security by preventing relocation during execution and ensuring immediate symbol resolution.".to_string(),
            metadata: Some(serde_json::json!({
                "has_rerlo": has_rerlo,
                "has_bind_now": has_bind_now
            })),
        },
        (true, false) => ValidationResult {
            name: "RERLO Protection".to_string(),
            status: false,
            description: "The ELF file has RERLO protection but lacks BIND_NOW. Without BIND_NOW, dynamic symbols may be resolved lazily during execution, which could reduce security.".to_string(),
            metadata: Some(serde_json::json!({
                "has_rerlo": has_rerlo,
                "has_bind_now": has_bind_now
            })),
        },
        (false, true) => ValidationResult {
            name: "RERLO Protection".to_string(),
            status: false,
            description: "The ELF file lacks RERLO protection but has BIND_NOW. While BIND_NOW ensures immediate symbol resolution, the absence of RERLO protection allows some sections to be writable, which might open up security risks.".to_string(),
            metadata: Some(serde_json::json!({
                "has_rerlo": has_rerlo,
                "has_bind_now": has_bind_now
            })),
        },
        (false, false) => ValidationResult {
            name: "RERLO Protection".to_string(),
            status: false,
            description: "The ELF file lacks both RERLO and BIND_NOW protections, which significantly reduces security. Both protections should ideally be enabled to prevent potential security vulnerabilities.".to_string(),
            metadata: Some(serde_json::json!({
                "has_rerlo": has_rerlo,
                "has_bind_now": has_bind_now
            })),
        },
    }
}

fn check_stack_canary(elf: &Elf) -> ValidationResult {
    let canary_symbol = "__stack_chk_fail";

    // Check if the ELF has the stack canary symbol in dynamic or static symbols
    if elf
        .dynsyms
        .iter()
        .any(|sym| elf.dynstrtab.get_at(sym.st_name) == Some(canary_symbol))
        || elf
            .syms
            .iter()
            .any(|sym| elf.strtab.get_at(sym.st_name) == Some(canary_symbol))
    {
        return ValidationResult {
            name: "Stack Canary".to_string(),
            status: true,
            description: "The ELF file uses stack canary protection, which helps prevent buffer overflow attacks by checking for stack corruption.".to_string(),
            metadata: Some(serde_json::json!({ "symbol": canary_symbol })),
        };
    }

    // Check for stack canary-related section names
    if elf.section_headers.iter().any(|sh| {
        elf.shdr_strtab
            .get_at(sh.sh_name)
            .is_some_and(|name| name.contains("stack_chk"))
    }) {
        return ValidationResult {
            name: "Stack Canary".to_string(),
            status: true,
            description: "Symbol names suggest the presence of stack canary protection, indicating measures against stack buffer overflow vulnerabilities.".to_string(),
            metadata: None,
        };
    }

    // If neither check is found, return a failure result
    ValidationResult {
        name: "Stack Canary".to_string(),
        status: false,
        description: "The ELF file lacks stack canary protection, which increases the risk of stack-based buffer overflow attacks.".to_string(),
        metadata: None,
    }
}

fn check_pie_protection(elf: &Elf) -> ValidationResult {
    if elf.header.e_type != ET_DYN {
        return ValidationResult {
            name: "PIE Protection".to_string(),
            status: false,
            description: "The ELF file is not a Position Independent Executable (PIE). A PIE is a type of executable that can be loaded at any address in memory, which enhances security by preventing attackers from predicting memory addresses.".to_string(),
            metadata: Some(serde_json::json!({
                "position_independent": false,
            })),
        };
    }

    let has_interpreter = elf.program_headers.iter().any(|ph| ph.p_type == PT_INTERP);

    if has_interpreter {
        ValidationResult {
            name: "PIE Protection".to_string(),
            status: true,
            description: "The ELF file is a Position Independent Executable (PIE). This allows the executable to be loaded at any memory address, increasing resistance against attacks such as buffer overflows.".to_string(),
            metadata: Some(serde_json::json!({
                "position_independent": true,
            })),
        }
    } else {
        ValidationResult {
            name: "PIE Protection".to_string(),
            status: false,
            description: "The ELF file is not a Position Independent Executable (PIE). Without PIE, the executable is loaded at a fixed memory address, which can make it easier for attackers to exploit vulnerabilities.".to_string(),
            metadata: Some(serde_json::json!({
                "position_independent": false,
            })),
        }
    }
}

fn check_wx_segments(elf: &Elf) -> ValidationResult {
    for ph in &elf.program_headers {
        if (ph.p_type == PT_LOAD) && (ph.p_flags & PF_W) != 0 && (ph.p_flags & PF_X) != 0 {
            return ValidationResult {
                name: "WX Segments".to_string(),
                status: false,
                description: "The ELF file contains writable and executable segments (WX segments). This configuration allows code to be both writable and executable, which is a serious security risk. It enables attackers to write and execute malicious code, potentially leading to arbitrary code execution.".to_string(),
                metadata: Some(serde_json::json!({
                    "segment_type": ph.p_type,
                    "flags": ph.p_flags
                })),
            };
        }
    }

    ValidationResult {
        name: "WX Segments".to_string(),
        status: true,
        description: "The ELF file does not contain writable and executable segments, which is a positive security measure. This prevents the risk of executing malicious code through writable areas of memory.".to_string(),
        metadata: None,
    }
}

fn check_suspicious_entry_point(elf: &Elf) -> ValidationResult {
    let is_relocatable = elf.header.e_type == ET_REL;
    let is_core = elf.header.e_type == ET_CORE;

    if (is_relocatable || is_core) && elf.header.e_entry > 0 {
        return ValidationResult {
            name: "Suspicious Entry Point".to_string(),
            status: false,
            description: "Relocatable or core dump files should not have a valid entry point. These file types are not intended to execute, and having a valid entry point could indicate a misconfiguration or tampered file.".to_string(),
            metadata: Some(serde_json::json!({
                "entry_point": elf.header.e_entry,
                "is_relocatable": is_relocatable,
                "is_core": is_core
            })),
        };
    }

    for section in &elf.section_headers {
        if let Some(name) = elf.shdr_strtab.get_at(section.sh_name) {
            if (name == ".data" || name == ".bss")
                && elf.header.e_entry >= section.sh_addr
                && elf.header.e_entry < (section.sh_addr + section.sh_size)
            {
                return ValidationResult {
                    name: "Suspicious Entry Point".to_string(),
                    status: false,
                    description: "Entry point located in a non-executable section (e.g., .data or .bss). The entry point should be located in an executable section to ensure the program can start execution.".to_string(),
                    metadata: Some(serde_json::json!({
                        "entry_point": elf.header.e_entry,
                        "section_name": name,
                        "section_address": section.sh_addr,
                        "section_size": section.sh_size
                    })),
                };
            }
        }
    }

    let in_executable_segment = elf.program_headers.iter().any(|ph| {
        ph.p_type == PT_LOAD
            && (ph.p_flags & PF_X) != 0
            && elf.header.e_entry >= ph.p_vaddr
            && elf.header.e_entry < (ph.p_vaddr + ph.p_memsz)
    });

    if in_executable_segment {
        return ValidationResult {
            name: "Suspicious Entry Point".to_string(),
            status: true,
            description: "The ELF file has a valid entry point located in an executable segment. This is expected for normal execution.".to_string(),
            metadata: Some(serde_json::json!({
                "entry_point": elf.header.e_entry
            })),
        };
    }

    ValidationResult {
        name: "Suspicious Entry Point".to_string(),
        status: false,
        description: "The entry point is outside of executable segments, which could indicate a corrupted or misconfigured ELF file.".to_string(),
        metadata: Some(serde_json::json!({
            "entry_point": elf.header.e_entry
        })),
    }
}

fn check_isolated_executable_sections(elf: &Elf) -> ValidationResult {
    let mut isolated_sections = Vec::new();

    let is_in_loadable_segment = |addr: u64, size: u64| {
        for ph in &elf.program_headers {
            if ph.p_type == PT_LOAD && addr >= ph.p_vaddr {
                if let Some(end) = addr.checked_add(size) {
                    if end <= ph.p_vaddr + ph.p_memsz {
                        return true;
                    }
                }
            }
        }
        false
    };

    for section in &elf.section_headers {
        if let Some(name) = elf.shdr_strtab.get_at(section.sh_name) {
            if (section.sh_flags & SHF_EXECINSTR as u64) != 0
                && !is_in_loadable_segment(section.sh_addr, section.sh_size)
            {
                isolated_sections.push(name.to_string());
            }
        }
    }

    if isolated_sections.is_empty() {
        ValidationResult {
            name: "Isolated Executable Sections".to_string(),
            status: true,
            description: "No isolated executable sections found. All executable sections are within loadable segments.".to_string(),
            metadata: None,
        }
    } else {
        ValidationResult {
            name: "Isolated Executable Sections".to_string(),
            status: false,
            description: format!(
                "Found {} isolated executable section(s): {:?}. These sections are marked as executable but are not located within any loadable segment, which could indicate potential issues with the file's structure or execution.",
                isolated_sections.len(),
                isolated_sections
            ),
            metadata: Some(serde_json::json!({
                "isolated_sections": isolated_sections
            })),
        }
    }
}

// ######################### ANALYSIS OF MEMORY MAPPING AND SEGMENTS #########################
fn validate_memory_mapping(elf: &Elf) -> CategoryResult {
    let mut category = CategoryResult {
        name: "Memory Mapping and Segments".to_string(),
        description: "Validates the memory mapping and segments of the ELF file.".to_string(),
        checks: Vec::new(),
    };

    let entry_point = check_entry_point(elf);
    category.checks.push(entry_point);

    let segment_alignment = check_segment_alignment(elf);
    if segment_alignment.is_empty() {
        category.checks.push(ValidationResult {
            name: "Segment Alignment".to_string(),
            status: true,
            description: "All segments are properly aligned.".to_string(),
            metadata: None,
        });
    } else {
        for result in segment_alignment {
            category.checks.push(result);
        }
    }

    let section_count = check_section_count_anomaly(elf);
    category.checks.push(section_count);

    let segment_count = check_segment_count_anomaly(elf);
    category.checks.push(segment_count);

    let upx_signature = check_upx_signature(elf);
    category.checks.push(upx_signature);

    let suspicious_section_names = check_suspicious_section_names(elf);
    category.checks.push(suspicious_section_names);

    let empty_segments = check_empty_segments(elf);
    category.checks.push(empty_segments);

    let overlapping_segments = check_overlapping_segments(elf);
    category.checks.push(overlapping_segments);

    category
}

fn check_entry_point(elf: &Elf) -> ValidationResult {
    let entry = elf.header.e_entry;

    if entry == 0 {
        return ValidationResult {
            name: "Entry Point".to_string(),
            status: false,
            description: "The ELF file has no entry point. This typically indicates an incomplete or incorrectly configured file.".to_string(),
            metadata: None,
        };
    }

    let mut in_loadable = false;
    let mut in_executable = false;
    let mut in_writable = false;

    // Check if the entry point is in a loadable segment
    for ph in elf.program_headers.iter() {
        if ph.p_type == PT_LOAD {
            let vaddr_end = ph.p_vaddr + ph.p_memsz;
            if entry >= ph.p_vaddr && entry < vaddr_end {
                in_loadable = true;
                in_executable = (ph.p_flags & PF_X) != 0;
                in_writable = (ph.p_flags & PF_W) != 0;
                break;
            }
        }
    }

    let mut section_type = None;
    // Check if the entry point is in a section
    for sh in &elf.section_headers {
        if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
            let addr_end = sh.sh_addr + sh.sh_size;
            if entry >= sh.sh_addr && entry < addr_end {
                section_type = Some(name.to_string());
                break;
            }
        }
    }

    // If entry point is not in a loadable segment
    if !in_loadable {
        ValidationResult {
            name: "Entry Point".to_string(),
            status: false,
            description: "The entry point is not located within a loadable segment. This could indicate a problem with how the file is mapped into memory.".to_string(),
            metadata: Some(serde_json::json!({
                "entry_point": entry,
                "in_loadable": in_loadable,
                "in_executable": in_executable,
                "in_writable": in_writable,
                "section_type": section_type
            })),
        }
    }
    // If entry point is in an executable and writable segment
    else if in_executable && in_writable {
        return ValidationResult {
            name: "Entry Point".to_string(),
            status: false,
            description: "The entry point is in an executable and writable segment, which is a potential security risk (W^X violation).".to_string(),
            metadata: Some(serde_json::json!({
                "entry_point": entry,
                "in_loadable": in_loadable,
                "in_executable": in_executable,
                "in_writable": in_writable,
                "section_type": section_type
            })),
        };
    }
    // If entry point is not in an executable segment
    else if !in_executable {
        return ValidationResult {
            name: "Entry Point".to_string(),
            status: false,
            description: "The entry point is not located in an executable segment, which may indicate an improperly configured ELF file.".to_string(),
            metadata: Some(serde_json::json!({
                "entry_point": entry,
                "in_loadable": in_loadable,
                "in_executable": in_executable,
                "in_writable": in_writable,
                "section_type": section_type
            })),
        };
    } else {
        // Entry point is valid
        return ValidationResult {
            name: "Entry Point".to_string(),
            status: true,
            description: "The entry point is valid and located in an executable segment."
                .to_string(),
            metadata: Some(serde_json::json!({
                "entry_point": entry,
                "in_loadable": in_loadable,
                "in_executable": in_executable,
                "in_writable": in_writable,
                "section_type": section_type
            })),
        };
    }
}

fn check_segment_alignment(elf: &Elf) -> Vec<ValidationResult> {
    let mut issues = Vec::new();

    for sh in &elf.section_headers {
        // Skip non-allocated or zero address sections
        if (sh.sh_flags & SHF_ALLOC as u64) == 0 || sh.sh_addr == 0 {
            continue;
        }

        let section_name = elf
            .shdr_strtab
            .get_at(sh.sh_name)
            .unwrap_or("<unknown>") // Default to "<unknown>" if the section name is not found
            .to_string();

        let section_start = sh.sh_addr;
        let section_end = sh.sh_addr + sh.sh_size;
        let is_executable = (sh.sh_flags & SHF_EXECINSTR as u64) != 0;
        let is_writable = (sh.sh_flags & SHF_WRITE as u64) != 0;

        let mut fully_contained = false;
        let mut partially_contained = false;
        let mut perms_mismatch = false;
        let mut matching_segment_info = None;

        // Iterate through the program headers to find matching loadable segments
        for (i, ph) in elf.program_headers.iter().enumerate() {
            if ph.p_type != PT_LOAD {
                continue;
            }

            let seg_start = ph.p_vaddr;
            let seg_end = ph.p_vaddr + ph.p_memsz;
            let seg_exec = (ph.p_flags & PF_X) != 0;
            let seg_write = (ph.p_flags & PF_W) != 0;

            let overlaps = section_start < seg_end && section_end > seg_start;

            if section_start >= seg_start && section_end <= seg_end {
                // Section is fully contained within the segment
                fully_contained = true;
                if (is_executable && !seg_exec) || (is_writable && !seg_write) {
                    // Permissions mismatch between section and segment
                    perms_mismatch = true;
                }
                matching_segment_info = Some((i, seg_start, ph.p_memsz));
                break;
            } else if overlaps {
                // Section partially overlaps with the segment
                partially_contained = true;
                matching_segment_info = Some((i, seg_start, ph.p_memsz));
            }
        }

        // If a matching segment was found, handle different cases
        if let Some((seg_idx, seg_start, seg_size)) = matching_segment_info {
            // Permissions mismatch case
            if perms_mismatch {
                issues.push(ValidationResult {
                    name: "Segment Alignment".to_string(),
                    status: false,
                    description: format!(
                        "Section '{}' is fully contained in segment {} (vaddr: {}, size: {}) with permissions mismatch.",
                        section_name, seg_idx, seg_start, seg_size
                    ),
                    metadata: Some(serde_json::json!({
                        "section_name": section_name,
                        "section_address": section_start,
                        "section_size": sh.sh_size,
                        "segment_index": seg_idx,
                        "segment_vaddr": seg_start,
                        "segment_size": seg_size,
                        "section_exec": is_executable,
                        "segment_exec": (elf.program_headers[seg_idx].p_flags & PF_X) != 0,
                        "section_write": is_writable,
                        "segment_write": (elf.program_headers[seg_idx].p_flags & PF_W) != 0
                    })),
                });
            }
            // Partially contained case
            else if partially_contained && !fully_contained {
                issues.push(ValidationResult {
                    name: "Segment Alignment".to_string(),
                    status: false,
                    description: format!(
                        "Section '{}' is partially contained in segment {} (vaddr: {}, size: {}).",
                        section_name, seg_idx, seg_start, seg_size
                    ),
                    metadata: Some(serde_json::json!({
                        "section_name": section_name,
                        "section_address": section_start,
                        "section_size": sh.sh_size,
                        "segment_index": seg_idx,
                        "segment_vaddr": seg_start,
                        "segment_size": seg_size
                    })),
                });
            }
        } else {
            // If no matching loadable segment was found
            issues.push(ValidationResult {
                name: "Segment Alignment".to_string(),
                status: false,
                description: format!(
                    "Section '{}' is not contained in any loadable segment. This could indicate misalignment or an issue with memory mapping.",
                    section_name
                ),
                metadata: Some(serde_json::json!({
                    "section_name": section_name,
                    "section_address": section_start,
                    "section_size": sh.sh_size
                })),
            });
        }
    }
    issues
}

fn check_section_count_anomaly(elf: &Elf) -> ValidationResult {
    let section_count_actual = elf.section_headers.len();
    let mut section_count_reported = elf.header.e_shnum as usize;

    // ELF spec: if e_shnum == 0, the real section count is in section[0].sh_size
    if section_count_reported == 0 && !elf.section_headers.is_empty() {
        section_count_reported = elf.section_headers[0].sh_size as usize;
    }

    // Case: No sections in ELF file
    if section_count_actual == 0 {
        return ValidationResult {
            name: "Section Count Anomaly".to_string(),
            status: false,
            description: "The ELF file has no sections.".to_string(),
            metadata: Some(serde_json::json!({
                "section_count_actual": section_count_actual,
                "section_count_reported": section_count_reported,
            })),
        };
    }

    // Case: Reported section count mismatch
    let description = if section_count_actual == section_count_reported {
        format!(
            "The ELF file has the expected number of sections ({}).",
            section_count_reported
        )
    } else {
        format!(
            "The ELF file has an unexpected number of sections ({}), expected {}.",
            section_count_actual, section_count_reported
        )
    };

    ValidationResult {
        name: "Section Count Anomaly".to_string(),
        status: section_count_actual == section_count_reported,
        description,
        metadata: Some(serde_json::json!({
            "section_count_actual": section_count_actual,
            "section_count_reported": section_count_reported,
        })),
    }
}

fn check_segment_count_anomaly(elf: &Elf) -> ValidationResult {
    let ph_count_actual = elf.program_headers.len();
    let mut ph_count_reported = elf.header.e_phnum as usize;

    // ELF spec: if e_phnum == 0xFFFF, true count may be in section[0].sh_info (rare but possible)
    if ph_count_reported == 0xFFFF && !elf.section_headers.is_empty() {
        ph_count_reported = elf.section_headers[0].sh_info as usize;
    }

    // Case: No program headers in ELF file
    if ph_count_actual == 0 {
        return ValidationResult {
            name: "Program Header Count Anomaly".to_string(),
            status: false,
            description: "The ELF file has no program headers.".to_string(),
            metadata: Some(serde_json::json!({
                "program_header_count_actual": ph_count_actual,
                "program_header_count_reported": ph_count_reported
            })),
        };
    }

    // Case: Reported program header count mismatch
    let description = if ph_count_actual == ph_count_reported {
        format!(
            "The ELF file has the expected number of program headers ({}).",
            ph_count_reported
        )
    } else {
        format!(
            "The ELF file has an unexpected number of program headers ({}), expected {}.",
            ph_count_actual, ph_count_reported
        )
    };

    ValidationResult {
        name: "Program Header Count Anomaly".to_string(),
        status: ph_count_actual == ph_count_reported,
        description,
        metadata: Some(serde_json::json!({
            "program_header_count_actual": ph_count_actual,
            "program_header_count_reported": ph_count_reported
        })),
    }
}

fn check_upx_signature(elf: &Elf) -> ValidationResult {
    // 1. Controlla i nomi delle sezioni
    for sh in &elf.section_headers {
        if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
            if name.to_lowercase().contains("upx") {
                return ValidationResult {
                    name: "UPX Signature".to_string(),
                    status: true,
                    description: "Found section with UPX-like name.".to_string(),
                    metadata: Some(serde_json::json!({
                        "section_name": name,
                        "section_address": sh.sh_addr,
                        "section_size": sh.sh_size
                    })),
                };
            }
        }
    }

    // 2. Controlla nelle stringhe dinamiche
    for i in 0..elf.dynstrtab.len() {
        if let Some(s) = elf.dynstrtab.get_at(i) {
            if s.to_lowercase().contains("upx") {
                return ValidationResult {
                    name: "UPX Signature".to_string(),
                    status: true,
                    description: "Found UPX reference in dynamic string table.".to_string(),
                    metadata: Some(serde_json::json!({
                        "string_index": i,
                        "string_value": s
                    })),
                };
            }
        }
    }

    // 3. Euristica: pochi segmenti, nessuna sezione, segmento LOAD gigante con p_filesz << p_memsz
    if elf.section_headers.is_empty() && elf.program_headers.len() <= 3 {
        for ph in &elf.program_headers {
            if ph.p_type == PT_LOAD && ph.p_memsz > 8 * 1024 * 1024 {
                let is_suspect_layout = ph.p_filesz < (ph.p_memsz / 4); // Molta memoria allocata ma pochi dati
                if is_suspect_layout {
                    return ValidationResult {
                        name: "UPX Signature".to_string(),
                        status: true,
                        description:
                            "Suspect UPX layout: large LOAD segment with minimal file data."
                                .to_string(),
                        metadata: Some(serde_json::json!({
                            "segment_type": ph.p_type,
                            "segment_mem_size": ph.p_memsz,
                            "segment_file_size": ph.p_filesz,
                            "segment_offset": ph.p_offset
                        })),
                    };
                }
            }
        }
    }

    // Nessuna firma rilevata
    ValidationResult {
        name: "UPX Signature".to_string(),
        status: false,
        description: "No UPX indicators found.".to_string(),
        metadata: None,
    }
}

fn check_suspicious_section_names(elf: &Elf) -> ValidationResult {
    let mut suspicious_sections = Vec::new();
    let common_section_names: Vec<&str> = vec![
        ".text",
        ".data",
        ".bss",
        ".rodata",
        ".comment",
        ".note",
        ".init",
        ".fini",
        ".plt",
        ".got",
        ".dynsym",
        ".dynstr",
        ".rela",
        ".rel",
        ".eh_frame",
        ".symtab",
        ".strtab",
        ".shstrtab",
        ".gcc_except_table",
        ".tbss",
        ".tdata",
        ".init_array",
        ".fini_array",
        ".ctors",
        ".dtors",
        ".jcr",
        ".dynamic",
        ".interp",
        ".hash",
        ".gnu",
        ".debug",
        ".gnu.version",
        ".gnu.hash",
        ".gnu.attributes",
        ".gnu.version_r",
        ".gnu.warning",
        ".gnu.linkonce",
    ];

    let is_common_variation = |name: &str| -> bool {
        for common in &common_section_names {
            if name.starts_with(common)
                || name.ends_with(common)
                || name.contains(&format!("{}_", common))
                || name.contains(&format!("_{}", common))
            {
                return true;
            }
        }
        false
    };

    for sh in &elf.section_headers {
        if let Some(name) = elf.shdr_strtab.get_at(sh.sh_name) {
            let mut reasons = Vec::new();

            if !name.is_ascii() {
                reasons.push("non_ascii");
            }
            if !common_section_names.contains(&name) && !is_common_variation(name) {
                if name.len() > 20 {
                    reasons.push("too_long");
                }
                if name
                    .chars()
                    .any(|c| !c.is_ascii_alphanumeric() && c != '.' && c != '_' && c != '-')
                {
                    reasons.push("unusual_chars");
                }
            }

            if !reasons.is_empty() {
                suspicious_sections.push(serde_json::json!({
                    "section_name": name,
                    "anomalies": reasons,
                    "address": sh.sh_addr,
                    "size": sh.sh_size
                }));
            }
        }
    }

    if suspicious_sections.is_empty() {
        ValidationResult {
            name: "Suspicious Section Names".to_string(),
            status: true,
            description: "No suspicious section names found.".to_string(),
            metadata: None,
        }
    } else {
        ValidationResult {
            name: "Suspicious Section Names".to_string(),
            status: false,
            description: format!(
                "{} suspicious section(s) with naming anomalies detected.",
                suspicious_sections.len()
            ),
            metadata: Some(serde_json::json!({
                "suspicious_sections": suspicious_sections
            })),
        }
    }
}

fn check_empty_segments(elf: &Elf) -> ValidationResult {
    let mut empty_segments = Vec::new();

    for (i, ph) in elf.program_headers.iter().enumerate() {
        if ph.p_type == PT_LOAD && ph.p_filesz == 0 && ph.p_memsz > 0 {
            empty_segments.push(format!(
                "Segment {}: p_vaddr = {:#x}, p_memsz = {}, p_offset = {}",
                i, ph.p_vaddr, ph.p_memsz, ph.p_offset
            ));
        }
    }

    if empty_segments.is_empty() {
        ValidationResult {
            name: "Empty Segments".to_string(),
            status: true,
            description: "No empty segments found.".to_string(),
            metadata: None,
        }
    } else {
        ValidationResult {
            name: "Empty Segments".to_string(),
            status: false,
            description: format!(
                "{} empty segment(s) found. These segments have a non-zero memory size (p_memsz) but no associated file size (p_filesz).",
                empty_segments.len()
            ),
            metadata: Some(serde_json::json!({
                "empty_segments": empty_segments
            })),
        }
    }
}

fn check_overlapping_segments(elf: &Elf) -> ValidationResult {
    let mut overlapping_segments = Vec::new();

    let mut load_segments: Vec<(
        usize, // index
        u64,   // vaddr_start
        u64,   // vaddr_end
        u32,   // flags
        u64,   // file offset
    )> = Vec::new();

    for (i, ph) in elf.program_headers.iter().enumerate() {
        if ph.p_type == PT_LOAD {
            load_segments.push((
                i,
                ph.p_vaddr,
                ph.p_vaddr + ph.p_memsz,
                ph.p_flags,
                ph.p_offset,
            ));
        }
    }

    for i in 0..load_segments.len() {
        for j in (i + 1)..load_segments.len() {
            let (idx1, start1, end1, flags1, offset1) = load_segments[i];
            let (idx2, start2, end2, flags2, offset2) = load_segments[j];

            if start1 < end2 && start2 < end1 {
                let overlap_start = start1.max(start2);
                let overlap_end = end1.min(end2);
                let overlap_size = overlap_end - overlap_start;

                overlapping_segments.push(serde_json::json!({
                    "segment1_index": idx1,
                    "segment2_index": idx2,
                    "overlap_range": format!("{:#x} - {:#x}", overlap_start, overlap_end),
                    "overlap_size": overlap_size,
                    "segment1": {
                        "vaddr_start": start1,
                        "vaddr_end": end1,
                        "flags": format!("{:#x}", flags1),
                        "offset": offset1
                    },
                    "segment2": {
                        "vaddr_start": start2,
                        "vaddr_end": end2,
                        "flags": format!("{:#x}", flags2),
                        "offset": offset2
                    }
                }));
            }
        }
    }

    if overlapping_segments.is_empty() {
        ValidationResult {
            name: "Overlapping Segments".to_string(),
            status: true,
            description: "No overlapping segments found.".to_string(),
            metadata: None,
        }
    } else {
        ValidationResult {
            name: "Overlapping Segments".to_string(),
            status: false,
            description: format!(
                "{} overlapping segment pair(s) found. Overlapping segments can lead to unexpected behavior and should be inspected.",
                overlapping_segments.len()
            ),
            metadata: Some(serde_json::json!({
                "overlapping_segments": overlapping_segments
            })),
        }
    }
}

// ######################### DEPENDENCIES AND INTERACTION WITH THE ENVIRONMENT #########################
fn validate_dependencies(elf: &Elf, file_path: &Path) -> CategoryResult {
    let mut category = CategoryResult {
        name: "Dependencies and Interaction with Environment".to_string(),
        description: "Validates dependencies and environment interactions of the ELF file."
            .to_string(),
        checks: Vec::new(),
    };

    let data = read_file(file_path);

    let interp = check_interpreter(elf, &data);
    category.checks.push(interp);

    let debug_info = check_debug_sections(elf);
    category.checks.push(debug_info);

    let rpath = check_rpath_runpath(elf);
    category.checks.push(rpath);

    let stripped = check_stripped(elf);
    category.checks.push(stripped);

    let dependency = check_dependency_count(elf);
    category.checks.push(dependency);

    category
}

fn check_interpreter(elf: &Elf, data: &[u8]) -> ValidationResult {
    let standard_interpreters = [
        "/lib64/ld-linux-x86-64.so.2",
        "/lib/ld-linux.so.2",
        "/lib/ld-linux-aarch64.so.1",
        "/lib/ld-linux-armhf.so.3",
        "/libx32/ld-linux-x32.so.2",
        "/lib/ld-musl-x86_64.so.1",
        "/lib64/ld-linux-riscv64-lp64d.so.1",
    ];

    let interpreter = elf.program_headers.iter().find_map(|ph| {
        if ph.p_type == PT_INTERP {
            let start = ph.p_offset as usize;
            let end = (ph.p_offset + ph.p_filesz) as usize;
            data.get(start..end)
                .and_then(|bytes| std::str::from_utf8(bytes).ok())
                .map(|s| s.trim_end_matches('\0').to_string())
        } else {
            None
        }
    });

    match interpreter {
        Some(interp) => {
            let is_standard = standard_interpreters
                .iter()
                .any(|std_interp| &interp == std_interp);

            ValidationResult {
                name: "Dynamic Interpreter".to_string(),
                status: is_standard,
                description: if is_standard {
                    "The binary uses a standard interpreter.".to_string()
                } else {
                    "The binary uses a non-standard, potentially suspect interpreter. This could indicate the use of a custom or modified interpreter.".to_string()
                },
                metadata: Some(serde_json::json!({
                    "interpreter": interp,
                    "is_standard": is_standard
                })),
            }
        }
        None => ValidationResult {
            name: "Dynamic Interpreter".to_string(),
            status: true,
            description: "The binary is statically compiled or does not require an interpreter. This means it is fully self-contained."
                .to_string(),
            metadata: Some(serde_json::json!({
                "interpreter": null
            })),
        },
    }
}

fn check_debug_sections(elf: &Elf) -> ValidationResult {
    let mut debug_related_sections = Vec::new();

    for section in &elf.section_headers {
        if let Some(name) = elf.shdr_strtab.get_at(section.sh_name) {
            if name.starts_with(".debug") || name == ".symtab" || name == ".strtab" {
                debug_related_sections.push(name.to_string());
            }
        }
    }

    if debug_related_sections.is_empty() {
        ValidationResult {
            name: "Debugging Indications".to_string(),
            status: true,
            description: "The binary has no sections or symbols indicative of a debug build."
                .to_string(),
            metadata: None,
        }
    } else {
        ValidationResult {
            name: "Debugging Indications".to_string(),
            status: false,
            description: format!(
                "The binary contains {} section(s) indicating the presence of symbols or debugging data. Presence of debug symbols can indicate that the binary was compiled with debugging information, potentially revealing sensitive data or implementation details.",
                debug_related_sections.len()
            ),
            metadata: Some(serde_json::json!({
                "debug_sections": debug_related_sections
            })),
        }
    }
}

fn check_rpath_runpath(elf: &Elf) -> ValidationResult {
    let mut entries = Vec::new();

    if let Some(dynamic) = &elf.dynamic {
        for dyn_entry in &dynamic.dyns {
            match dyn_entry.d_tag {
                DT_RPATH => {
                    if let Some(path) = elf.dynstrtab.get_at(dyn_entry.d_val as usize) {
                        entries.push(("RPATH", path.to_string()));
                    }
                }
                DT_RUNPATH => {
                    if let Some(path) = elf.dynstrtab.get_at(dyn_entry.d_val as usize) {
                        entries.push(("RUNPATH", path.to_string()));
                    }
                }
                _ => {}
            }
        }
    }

    if entries.is_empty() {
        return ValidationResult {
            name: "RPATH / RUNPATH".to_string(),
            status: true,
            description: "The binary does not use RPATH or RUNPATH.".to_string(),
            metadata: None,
        };
    }

    let mut metadata = Vec::new();
    let mut risky = false;

    for (kind, path) in &entries {
        let mut details = format!("{}: {}", kind, path);
        if path.contains("./") || path.contains("../") || path.contains("$ORIGIN") {
            details.push_str(" (⚠️ Contains relative path or environment variable)");
            risky = true;
        }
        if !path.starts_with('/') {
            details.push_str(" (⚠️ Non-absolute path)");
            risky = true;
        }
        metadata.push(details);
    }

    ValidationResult {
        name: "RPATH / RUNPATH".to_string(),
        status: !risky,
        description: if risky {
            "Potentially dangerous RPATH/RUNPATH entries found. These may allow loading shared libraries from untrusted or relative locations, which can be exploited for code injection.".to_string()
        } else {
            "RPATH/RUNPATH entries were found, but they use absolute and safe paths. These do not appear to pose a security risk.".to_string()
        },
        metadata: Some(metadata.into()),
    }
}

fn check_stripped(elf: &Elf) -> ValidationResult {
    let mut has_symtab = false;
    let mut has_strtab = false;
    let mut debug_sections = Vec::new();

    for section in &elf.section_headers {
        if let Some(name) = elf.shdr_strtab.get_at(section.sh_name) {
            match name {
                ".symtab" => has_symtab = true,
                ".strtab" => has_strtab = true,
                _ if name.starts_with(".debug") => debug_sections.push(name.to_string()),
                _ => {}
            }
        }
    }

    let stripped_level = match (has_symtab, has_strtab, debug_sections.is_empty()) {
        (true, _, _) => "Not stripped",
        (false, true, false) => "Partially stripped",
        (false, _, true) => "Stripped",
        _ => "Partially stripped",
    };

    let (status, description) = match stripped_level {
        "Not stripped" => (
            false,
            "The binary contains a full symbol table and has not been stripped. This may expose internal details useful for reverse engineering.",
        ),
        "Partially stripped" => (
            false,
            "The binary is partially stripped: some debug or symbolic information remains. This could still leak internal structure or behavior.",
        ),
        "Stripped" => (
            true,
            "The binary appears to be stripped: no static symbols or debug sections are present.",
        ),
        _ => (true, "Unable to conclusively determine stripping level, but no major debug symbols found."),
    };

    let mut metadata = Vec::new();
    metadata.push(format!("Level: {}", stripped_level));
    metadata.push(format!(".symtab present: {}", has_symtab));
    metadata.push(format!(".strtab present: {}", has_strtab));
    if !debug_sections.is_empty() {
        metadata.push(format!("debug_sections: {}", debug_sections.join(", ")));
    }

    ValidationResult {
        name: "Binary Stripping".to_string(),
        status,
        description: description.to_string(),
        metadata: Some(metadata.into()),
    }
}

fn check_dependency_count(elf: &Elf) -> ValidationResult {
    let mut needed_libs = Vec::new();

    if let Some(dynamic) = &elf.dynamic {
        for dyn_entry in &dynamic.dyns {
            if dyn_entry.d_tag == DT_NEEDED {
                if let Some(name) = elf.dynstrtab.get_at(dyn_entry.d_val as usize) {
                    needed_libs.push(name.to_string());
                }
            }
        }
    }

    let needed_libs_count = needed_libs.len();

    let classification = match needed_libs_count {
        0 => "No dependencies (statically linked or minimal)",
        1..=5 => "Very lightweight",
        6..=15 => "Moderately dependent",
        16..=25 => "Heavy",
        _ => "Very heavy / possibly suspicious",
    };

    let description = format!(
        "The binary declares {} dynamic dependencies. Classification: {}.",
        needed_libs_count, classification
    );

    let is_suspicious = needed_libs_count > 20;

    let enriched_description = if is_suspicious {
        format!(
            "{} This high number of dependencies may indicate unusual complexity, modularity, or inclusion of third-party or bundled components, which can be a red flag in security-sensitive contexts.",
            description
        )
    } else {
        description
    };

    ValidationResult {
        name: "Number of Dependencies".to_string(),
        status: !is_suspicious,
        description: enriched_description,
        metadata: Some(serde_json::json!({
            "dependency_count": needed_libs_count,
            "classification": classification,
            "needed_libraries": needed_libs
        })),
    }
}
