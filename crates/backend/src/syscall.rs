use std::collections::HashMap;
use std::fs;

use crate::{error::Result, FunctionNode, SyscallInfo};

use regex::Regex;

type RegisterMap = HashMap<String, u64>;
type SyscallSet = Vec<u64>;
type HandlerFn = Box<dyn Fn(&str, &mut RegisterMap, &mut SyscallSet)>;

/// Detects system calls within the provided functions and updates their information.
///
/// # Arguments
///
/// - `functions`: A mutable reference to a `HashMap` containing function names as keys and `FunctionNode` as values.
///
/// # Returns
///
/// - A `Result` indicating success or failure.
///
/// # Errors
///
/// - Returns errors if loading the syscall table or extracting syscall numbers fails.
///
/// # Feature Flags
///
/// - `progress_bar`: If enabled, displays a progress bar indicating the progress of syscall detection.
pub fn detect_syscalls<S: ::std::hash::BuildHasher>(
    functions: &mut HashMap<String, FunctionNode, S>,
) -> Result<()> {
    #[cfg(feature = "progress_bar")]
    let pb = {
        let pb = indicatif::ProgressBar::new(functions.len() as u64);
        pb.set_message("System Call detection:".to_string());
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{msg}\n{wide_bar} {pos}/{len} [{elapsed_precise}]")?,
        );
        pb
    };

    let syscall_table = load_syscall_table()?;
    for func_node in functions.values_mut() {
        if func_node.syscall {
            if let Some(ref disasm) = func_node.disassembly {
                let syscall_numbers = extract_syscall_numbers(disasm)?;
                for number in syscall_numbers {
                    if let Some(info) = syscall_table.get(&number) {
                        func_node.set_syscall_info(info.clone());
                    }
                }
            }
        }
        #[cfg(feature = "progress_bar")]
        pb.inc(1);
    }

    #[cfg(feature = "progress_bar")]
    pb.finish_with_message("System calls collected!");
    Ok(())
}

fn load_syscall_table() -> Result<HashMap<u64, SyscallInfo>> {
    // let data = fs::read_to_string("./data/syscall_tab.json")?;
    let data = fs::read_to_string("syscall_tab.json")?;

    let syscalls: Vec<SyscallInfo> = serde_json::from_str(&data)?;
    Ok(syscalls.into_iter().map(|info| (info.id, info)).collect())
}

fn extract_syscall_numbers(disassembly: &str) -> Result<Vec<u64>> {
    let mut syscall_numbers: Vec<u64> = Vec::new();
    let mut registers: HashMap<String, u64> = HashMap::new();

    let patterns = get_patterns()?;

    for line in disassembly.lines() {
        if let Some(handler) = patterns.iter().find_map(|(re, handler)| {
            if re.is_match(line) {
                Some(handler)
            } else {
                None
            }
        }) {
            handler(line, &mut registers, &mut syscall_numbers);
        }
    }

    Ok(syscall_numbers)
}

fn get_patterns() -> Result<Vec<(Regex, HandlerFn)>> {
    let mov_imm_to_reg = Regex::new(r"mov\s+\$([a-fA-F0-9x]+),\s+%(\w+)")?;
    let mov_reg_to_reg = Regex::new(r"mov\s+%(\w+),\s*%(\w+)")?;
    let xor_reg_to_self = Regex::new(r"xor\s+%(\w+),\s*%(\w+)")?;
    let syscall_re = Regex::new(r"syscall")?;

    Ok(vec![
        (
            mov_imm_to_reg.clone(),
            Box::new(move |line, registers, _| {
                if let Some(caps) = mov_imm_to_reg.captures(line) {
                    let value_str = caps.get(1).unwrap().as_str();
                    let register = caps.get(2).unwrap().as_str().to_string();
                    let value = if let Some(value_str) = value_str.strip_prefix("0x") {
                        u64::from_str_radix(value_str, 16).unwrap()
                    } else {
                        value_str.parse::<u64>().unwrap()
                    };
                    registers.insert(register, value);
                }
            }),
        ),
        (
            mov_reg_to_reg.clone(),
            Box::new(move |line, registers, _| {
                if let Some(caps) = mov_reg_to_reg.captures(line) {
                    let src = caps.get(1).unwrap().as_str();
                    let dest = caps.get(2).unwrap().as_str().to_string();
                    if let Some(&value) = registers.get(src) {
                        registers.insert(dest, value);
                    }
                }
            }),
        ),
        (
            xor_reg_to_self.clone(),
            Box::new(move |line, registers, _| {
                if let Some(caps) = xor_reg_to_self.captures(line) {
                    let reg1 = caps.get(1).unwrap().as_str();
                    let reg2 = caps.get(2).unwrap().as_str();
                    if reg1 == reg2 {
                        registers.insert(reg1.to_string(), 0);
                    }
                }
            }),
        ),
        (
            syscall_re.clone(),
            Box::new(move |_, registers, syscall_numbers| {
                if let Some(&value) = registers.get("eax") {
                    syscall_numbers.push(value);
                }
            }),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_detect_syscalls() {
        let mut functions = HashMap::new();
        let mut func_node = FunctionNode::new("mock_function".to_string(), 0x1000, 0x2000);
        func_node.syscall = true;
        func_node.disassembly = Some("mov $0x1, %eax\nsyscall".to_string());
        functions.insert("mock_function".to_string(), func_node);

        let result = detect_syscalls(&mut functions);

        assert!(result.is_ok());

        let func_node = functions.get("mock_function").unwrap();
        assert!(func_node.syscall_info.is_some());
    }

    #[test]
    #[cfg(feature = "progress_bar")]
    fn test_detect_syscalls_with_progress_bar() {
        let mut functions = HashMap::new();
        let mut func_node = FunctionNode::new("mock_function".to_string(), 0x1000, 0x2000);
        func_node.syscall = true;
        func_node.disassembly = Some("mov $0x1, %eax\nsyscall".to_string());
        functions.insert("mock_function".to_string(), func_node);

        let result = detect_syscalls(&mut functions);

        assert!(result.is_ok());

        let func_node = functions.get("mock_function").unwrap();
        assert!(func_node.syscall_info.is_some());
    }
}
