pub mod analyse;
pub mod detect;
pub mod digest;
pub mod entry;
pub mod error;
pub mod inspect;

/// Represents the basic metadata extracted from an ELF binary.
///
/// # Overview
/// The `BasicInfo` structure contains essential information about an ELF binary,
/// such as its name, type, architecture, size, and entry point.
/// This structure is typically used to summarise key properties of a binary in a readable format.
///
/// # Fields
/// - `file_name`: The name of the ELF file.
/// - `file_type`: The type of the ELF file (e.g., `Executable`, `Shared Object`).
/// - `file_size`: The size of the ELF file in bytes.
/// - `arch`: The architecture of the binary (e.g., `x86_64`, `ARM`).
/// - `pie`: Indicates whether the binary is position-independent (true for PIE binaries).
/// - `stripped`: Indicates whether the binary lacks debug symbols.
/// - `static_linking`: A string indicating if the binary is statically or dynamically linked.
/// - `language`: The programming language used to write the binary (e.g., `C`, `C++`).
/// - `entry_point`: The address of the entry point in the binary.
///
/// # Usage
/// This structure is used in [`inspect_binary`](crate::inspect::inspect_binary)
/// to save the extracted information and return it to the caller.
///
/// # See also
/// - [`inspect_binary`](crate::inspect::inspect_binary): Uses this structure to encapsulate extracted binary data.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BasicInfo<'a> {
    pub file_name: &'a str,
    pub file_type: &'a str,
    pub file_size: u64,
    pub arch: &'a str,
    pub pie: bool,
    pub stripped: bool,
    pub static_linking: &'a str,
    pub language: String,
    pub entry_point: u64,
}
impl<'a> BasicInfo<'a> {
    /// Creates a new `BasicInfo` instance with the provided file name and file type.
    ///
    /// # Arguments
    /// - `file_name`: The name of the ELF file.
    /// - `file_type`: The type of the ELF file.
    ///
    /// # Returns
    /// A new `BasicInfo` instance with default values for all other fields.
    ///
    /// # Example
    /// ```
    /// use manifest_producer_backend::BasicInfo;
    ///
    /// let info = BasicInfo::new("example.elf", "Executable");
    /// assert_eq!(info.file_name, "example.elf");
    /// assert_eq!(info.file_type, "Executable");
    /// ```
    #[must_use]
    pub fn new(file_name: &'a str, file_type: &'a str) -> Self {
        Self {
            file_name,
            file_type,
            file_size: 0,
            arch: "",
            pie: false,
            stripped: false,
            static_linking: "",
            language: String::new(),
            entry_point: 0,
        }
    }

    /// Sets the file size of the binary.
    ///
    /// # Arguments
    /// - `file_size`: The size of the binary in bytes.
    ///
    /// # Returns
    /// A new `BasicInfo` instance with the updated file size.
    ///
    /// # Example
    /// ```
    /// use manifest_producer_backend::BasicInfo;
    ///
    /// let info = BasicInfo::new("example.elf", "Executable").file_size(1024);
    /// assert_eq!(info.file_size, 1024);
    /// ```
    #[must_use]
    pub fn file_size(self, file_size: u64) -> Self {
        Self { file_size, ..self }
    }

    /// Sets the architecture of the binary.
    ///
    /// # Arguments
    /// - `arch`: The architecture string (e.g., `x86_64`).
    ///
    /// # Returns
    /// A new `BasicInfo` instance with the updated architecture.
    #[must_use]
    pub fn arch(self, arch: &'a str) -> Self {
        Self { arch, ..self }
    }

    /// Sets whether the binary is position-independent (PIE).
    ///
    /// # Arguments
    /// - `pie`: A boolean indicating whether the binary is PIE.
    ///
    /// # Returns
    /// A new `BasicInfo` instance with the updated PIE status.
    #[must_use]
    pub fn pie(self, pie: bool) -> Self {
        Self { pie, ..self }
    }

    /// Sets the static or dynamic linking type of the binary.
    ///
    /// # Arguments
    /// - `static_linking`: A string indicating the linking type.
    ///
    /// # Returns
    /// A new `BasicInfo` instance with the updated linking type.
    #[must_use]
    pub fn static_linking(self, static_linking: &'a str) -> Self {
        Self {
            static_linking,
            ..self
        }
    }

    /// Sets the programming language of the binary.
    ///
    /// # Arguments
    /// - `language`: A string indicating the language.
    ///
    /// # Returns
    /// A new `BasicInfo` instance with the updated language.
    #[must_use]
    pub fn language(self, language: String) -> Self {
        Self { language, ..self }
    }

    /// Sets the entry point address of the binary.
    ///
    /// # Arguments
    /// - `entry_point`: The address of the entry point.
    ///
    /// # Returns
    /// A new `BasicInfo` instance with the updated entry point.
    #[must_use]
    pub fn entry_point(self, entry_point: u64) -> Self {
        Self {
            entry_point,
            ..self
        }
    }

    /// Sets whether the binary is stripped.
    ///
    /// # Arguments
    /// - `stripped`: A boolean indicating whether the binary is stripped.
    ///
    /// # Returns
    /// A new `BasicInfo` instance with the updated stripped status.
    #[must_use]
    pub fn stripped(self, stripped: bool) -> Self {
        Self { stripped, ..self }
    }
}

/// Represents a node in the call tree of a binary's functions.
///
/// # Overview
/// `FunctionNode` captures details about an individual function in an ELF binary,
/// including its name, address range, invocation count, and any child functions
/// it may call (based on disassembly).
///
/// This structure is central to analysing and representing the relationships between functions.
///
/// # Fields
/// - `name`: The demangled name of the function.
/// - `start_addr`: The start address of the function in the binary.
/// - `end_addr`: The end address of the function in the binary.
/// - `invocation_entry`: The number of times this function is invoked by another function in the binary.
/// - `jmp`:  The number of times the function is identified for the construction of its subtree.
/// - `children`: A list of function names that are called by this function.
/// - `disassembly`: An optional field containing the disassembled machine code for the function.
///
/// # See also
/// - [`analyse_functions`](crate::analyse::analyse_functions): Uses `FunctionNode` to build the call graph.
/// - [`function_detection`](crate::detect::function_detection): Creates initial `FunctionNode` objects.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FunctionNode {
    pub name: String,
    pub start_addr: u64,
    pub end_addr: u64,
    pub invocation_entry: usize,
    pub jmp: usize,
    pub children: Vec<String>,
    pub disassembly: Option<String>,
}
impl FunctionNode {
    /// Creates a new `FunctionNode`.
    ///
    /// # Arguments
    /// - `name`: The name of the function.
    /// - `start_addr`: The starting memory address of the function in the binary.
    /// - `end_addr`: The ending memory address of the function in the binary.
    ///
    /// # Returns
    /// - A new instance of `FunctionNode` with the provided name, address range, and default values for other fields.
    ///
    /// # Example
    /// ```
    /// use manifest_producer_backend::FunctionNode;
    ///
    /// let func_node = FunctionNode::new(
    ///     "example_function".to_string(),
    ///     0x1000,
    ///     0x2000,
    /// );
    /// assert_eq!(func_node.name, "example_function");
    /// ```
    #[must_use]
    pub fn new(name: String, start_addr: u64, end_addr: u64) -> Self {
        Self {
            name,
            start_addr,
            end_addr,
            invocation_entry: 0,
            jmp: 0,
            children: Vec::new(),
            disassembly: None,
        }
    }

    /// Sets the disassembly for the function.
    ///
    /// # Arguments
    /// - `disassembly`: A string containing the disassembled machine code for the function.
    ///
    /// # Example
    /// ```
    /// use manifest_producer_backend::FunctionNode;
    ///
    /// let mut func_node = FunctionNode::new(
    ///     "example_function".to_string(),
    ///     0x1000,
    ///     0x2000,
    /// );
    /// func_node.set_disassembly("MOV RAX, RBX\nCALL 0x1020".to_string());
    /// assert!(func_node.disassembly.is_some());
    /// ```
    pub fn set_disassembly(&mut self, disassembly: String) {
        self.disassembly = Some(disassembly);
    }
}
