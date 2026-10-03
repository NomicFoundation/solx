//!
//! The `solc --standard-json` expected output selector.
//!

///
/// The `solc --standard-json` expected output selector.
///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Selector {
    /// The AST JSON.
    #[serde(rename = "ast")]
    AST,
    /// The symbol table for debuggers and stack tracers.
    #[serde(rename = "debugSymbols")]
    DebugSymbols,
    /// The ABI JSON.
    #[serde(rename = "abi")]
    ABI,
    /// The metadata.
    #[serde(rename = "metadata")]
    Metadata,
    /// The developer documentation.
    #[serde(rename = "devdoc")]
    DeveloperDocumentation,
    /// The user documentation.
    #[serde(rename = "userdoc")]
    UserDocumentation,
    /// The storage layout.
    #[serde(rename = "storageLayout")]
    StorageLayout,
    /// The transient storage layout.
    #[serde(rename = "transientStorageLayout")]
    TransientStorageLayout,
    /// The function signature hashes JSON.
    #[serde(rename = "evm.methodIdentifiers")]
    MethodIdentifiers,
    /// The compilation pipeline benchmarks.
    #[serde(rename = "benchmarks")]
    Benchmarks,
    /// The MLIR source code (LLVM dialect, solx intermediate representation).
    #[serde(rename = "mlir")]
    MLIR,

    /// All EVM data.
    #[serde(rename = "evm")]
    EVM,
    /// The deploy bytecode.
    #[serde(rename = "evm.bytecode")]
    Bytecode,
    /// The deploy bytecode object.
    #[serde(rename = "evm.bytecode.object")]
    BytecodeObject,
    /// The deploy unoptimized LLVM IR (solx internal representation).
    #[serde(rename = "evm.bytecode.llvmIrUnoptimized")]
    BytecodeLLVMIRUnoptimized,
    /// The deploy LLVM IR (solx internal representation).
    #[serde(rename = "evm.bytecode.llvmIr")]
    BytecodeLLVMIR,
    /// The deploy LLVM assembly.
    #[serde(rename = "evm.bytecode.llvmAssembly")]
    BytecodeLLVMAssembly,
    /// The deploy bytecode opcodes.
    #[serde(rename = "evm.bytecode.opcodes")]
    BytecodeOpcodes,
    /// The deploy bytecode link references.
    #[serde(rename = "evm.bytecode.linkReferences")]
    BytecodeLinkReferences,
    /// The deploy bytecode source maps (solc-style, unused).
    #[serde(rename = "evm.bytecode.sourceMap")]
    BytecodeSourceMap,
    /// The deploy bytecode debug info (DWARF).
    #[serde(rename = "evm.bytecode.debugInfo")]
    BytecodeDebugInfo,
    /// The deploy bytecode function debug data.
    #[serde(rename = "evm.bytecode.functionDebugData")]
    BytecodeFunctionDebugData,
    /// The deploy bytecode generated sources.
    #[serde(rename = "evm.bytecode.generatedSources")]
    BytecodeGeneratedSources,
    /// The runtime bytecode.
    #[serde(rename = "evm.deployedBytecode")]
    RuntimeBytecode,
    /// The runtime bytecode object.
    #[serde(rename = "evm.deployedBytecode.object")]
    RuntimeBytecodeObject,
    /// The runtime unoptimized LLVM IR (solx internal representation).
    #[serde(rename = "evm.deployedBytecode.llvmIrUnoptimized")]
    RuntimeBytecodeLLVMIRUnoptimized,
    /// The runtime LLVM IR (solx internal representation).
    #[serde(rename = "evm.deployedBytecode.llvmIr")]
    RuntimeBytecodeLLVMIR,
    /// The runtime LLVM assembly.
    #[serde(rename = "evm.deployedBytecode.llvmAssembly")]
    RuntimeBytecodeLLVMAssembly,
    /// The runtime bytecode opcodes.
    #[serde(rename = "evm.deployedBytecode.opcodes")]
    RuntimeBytecodeOpcodes,
    /// The runtime bytecode link references.
    #[serde(rename = "evm.deployedBytecode.linkReferences")]
    RuntimeBytecodeLinkReferences,
    /// The runtime bytecode immutable references.
    #[serde(rename = "evm.deployedBytecode.immutableReferences")]
    RuntimeBytecodeImmutableReferences,
    /// The runtime bytecode source maps (solc-style, unused).
    #[serde(rename = "evm.deployedBytecode.sourceMap")]
    RuntimeBytecodeSourceMap,
    /// The runtime bytecode debug info (DWARF).
    #[serde(rename = "evm.deployedBytecode.debugInfo")]
    RuntimeBytecodeDebugInfo,
    /// The runtime bytecode function debug data.
    #[serde(rename = "evm.deployedBytecode.functionDebugData")]
    RuntimeBytecodeFunctionDebugData,
    /// The runtime bytecode generated sources.
    #[serde(rename = "evm.deployedBytecode.generatedSources")]
    RuntimeBytecodeGeneratedSources,
    /// The gas estimates.
    #[serde(rename = "evm.gasEstimates")]
    GasEstimates,

    /// The wildcard variant that selects everything.
    #[serde(rename = "*")]
    Any,
}
