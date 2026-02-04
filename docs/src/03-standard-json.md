# Standard JSON

Standard JSON is a protocol for interaction with the **solx** and **solc** compilers. This protocol must be implemented by toolkits such as Hardhat.

The protocol uses two data formats for communication: [input JSON](#input-json) and [output JSON](#output-json).



## Usage

Input JSON can be provided by-value via the `--standard-json` option:

```shell
solx --standard-json './input.json'
```

Alternatively, the input JSON can be fed to **solx** via *stdin*:

```shell
cat './input.json' | solx --standard-json
```

You can also insert your standard JSON input directly into the command line:

```shell
solx --standard-json

<paste into stdin here and press Ctrl-D>
```

> For the sake of interface unification, **solx** will always return with exit code 0 and have its standard JSON output printed to *stdout*.
> It differs from **solc** that may return with exit code 1 and a free-formed error in some cases, such as when the standard JSON input file is missing, even though [the **solc** documentation claims otherwise](https://docs.soliditylang.org/en/latest/using-the-compiler.html#compiler-input-and-output-json-description).



## Input JSON

The input JSON provides the compiler with the source code and settings for the compilation. The example below serves as the specification of the input JSON format.

This format introduces several **solx**-specific parameters such as `settings.optimizer.sizeFallback`. These parameters are marked as `solx-only`.

On the other hand, parameters that are not mentioned here but are parts of **solc** standard JSON protocol have no effect in **solx**.

```javascript
{
  // Required: Source code language.
  // Currently supported: "Solidity", "Yul", "LLVM IR".
  "language": "Solidity",
  // Required: Source code files to compile.
  // The keys here are the "global" names of the source files. Imports can be using other file paths via remappings.
  "sources": {
    // In source file entry, either but not both "urls" and "content" must be specified.
    "myFile.sol": {
      // Required (unless "content" is used): URL(s) to the source file.
      "urls": [
        // In Solidity mode, directories must be added to the command-line via "--allow-paths <path>" for imports to work.
        // It is possible to specify multiple URLs for a single source file. In this case the first successfully resolved URL will be used.
        "/tmp/path/to/file.sol"
      ],
      // Required (unless "urls" is used): Literal contents of the source file.
      "content": "contract settable is owned { uint256 private x = 0; function set(uint256 _x) public { if (msg.sender == owner) x = _x; } }"
    }
  },

  // Required: Compilation settings.
  "settings": {
    // Optional: Optimizer settings.
    "optimizer": {
      // Optional, solx-only: Set the LLVM optimizer level.
      // Available options:
      // -0: do not optimize (unsupported)
      // -1: basic optimizations for gas usage
      // -2: advanced optimizations for gas usage
      // -3: all optimizations for gas usage
      // -s: basic optimizations for bytecode size
      // -z: all optimizations for bytecode size
      // Default: 3.
      "mode": "3",
      // Optional, solx-only: Re-run the compilation with "mode": "z" if the initial compilation exceeds the EVM bytecode size limit.
      // Used on a per-contract basis and applied automatically, so some contracts will end up compiled in the initial mode, and others with "mode": "z".
      // Only activated if "mode" is set to "3", which is the default optimization mode.
      // Default: false.
      "sizeFallback": false
    },

    // Optional: Sorted list of remappings.
    // Important: Only used with Solidity input.
    "remappings": [ ":g=/dir" ],
    // Optional: Addresses of the libraries.
    // If not all library addresses are provided here, it will result in unlinked bytecode files that will require post-compile-time linking before deployment.
    // Important: Only used with Solidity, Yul, and LLVM IR input.
    "libraries": {
      // The top level key is the name of the source file where the library is used.
      // If remappings are used, this source file should match the global path after remappings were applied.
      "myFile.sol": {
        // Source code library name and address where it is deployed.
        "MyLib": "0x123123..."
      }
    },

    // Optional: Version of EVM solx will produce bytecode for.
    // Supported EVM versions: "cancun", "prague", "osaka".
    // For instance, with version "osaka", solx will be producing `clz` instructions, whereas for older EVM versions it will not.
    // The oldest supported EVM version is "cancun".
    // Default: "osaka".
    "evmVersion": "osaka",
    // Optional: Select the desired output.
    // Default: no flags are selected, and no output is generated.
    "outputSelection": {
      "<path>": {
        // Available file-level options, must be listed under "<path>"."":
        "": [
          // AST of all source files.
          "ast",
          // Benchmarks of the solx LLVM-based compilation pipeline and its underlying call to solc.
          "benchmarks"
        ],
        // Available contract-level options, must be listed under "<path>"."<name>":
        "<name>": [
          // Solidity ABI.
          "abi",
          // Metadata.
          "metadata",
          // Developer documentation (natspec).
          "devdoc",
          // User documentation (natspec).
          "userdoc",
          // Slots, offsets and types of the contract's state variables in storage.
          "storageLayout",
          // Slots, offsets and types of the contract's state variables in transient storage.
          "transientStorageLayout",
          // Yul produced by solc.
          // An alias "irOptimized" is supported for compatibility, but it will request unoptimized Yul IR anyway.
          "ir",
          // Everything of the below.
          "evm",
          // Solidity function hashes.
          "evm.methodIdentifiers",
          // EVM assembly produced by solc.
          "evm.legacyAssembly",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.gasEstimates",
          // Everything that starts with "evm.bytecode".
          "evm.bytecode",
          // Deploy bytecode produced by solx/LLVM.
          // As long as the solx bytecode linker is in experimental stage, all contracts will be compiled if this key is enabled for at least one contract.
          "evm.bytecode.object",
          // Deploy code assembly produced by solx/LLVM.
          "evm.bytecode.llvmAssembly",
          // solx-only: EVM legacy assembly IR (internal representation). Only available for non-viaIR mode.
          "evm.bytecode.evmla",
          // solx-only: Ethereal IR (internal representation). Only available for non-viaIR mode.
          "evm.bytecode.ethir",
          // solx-only: Unoptimized LLVM IR (internal representation).
          "evm.bytecode.llvmIrUnoptimized",
          // solx-only: Optimized LLVM IR (internal representation).
          "evm.bytecode.llvmIr",
          // ELF-wrapped DWARF debug info produced by solx/LLVM. Only available for Solidity source code input.
          "evm.bytecode.debugInfo",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.bytecode.opcodes",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.bytecode.sourceMap",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.bytecode.functionDebugData",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.bytecode.generatedSources",
          // Everything that starts with "evm.deployedBytecode".
          "evm.deployedBytecode",
          // Runtime bytecode produced by solx/LLVM.
          // As long as the solx bytecode linker is in experimental stage, all contracts will be compiled if this key is enabled for at least one contract.
          "evm.deployedBytecode.object",
          // Runtime code assembly produced by solx/LLVM.
          "evm.deployedBytecode.llvmAssembly",
          // solx-only: EVM legacy assembly IR (internal representation). Only available for non-viaIR mode.
          "evm.deployedBytecode.evmla",
          // solx-only: Ethereal IR (internal representation). Only available for non-viaIR mode.
          "evm.deployedBytecode.ethir",
          // solx-only: Unoptimized LLVM IR (internal representation).
          "evm.deployedBytecode.llvmIrUnoptimized",
          // solx-only: Optimized LLVM IR (internal representation).
          "evm.deployedBytecode.llvmIr",
          // Link references for linkers that are to resolve library addresses at deploy time.
          "evm.deployedBytecode.linkReferences",
          // Resolved automatically by solx/LLVM, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.deployedBytecode.immutableReferences",
          // ELF-wrapped DWARF debug info produced by solx/LLVM. Only available for Solidity source code input.
          "evm.deployedBytecode.debugInfo",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.deployedBytecode.opcodes",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.deployedBytecode.sourceMap",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.deployedBytecode.functionDebugData",
          // Unsupported, but emitted as an empty object to preserve compatibility with some toolkits.
          "evm.deployedBytecode.generatedSources"
        ]
      }
    },
    // Optional: Metadata settings.
    "metadata": {
      // Optional: Use the given hash method for the metadata hash that is appended to the bytecode.
      // Available options: "none", "ipfs".
      // Default: "ipfs".
      "bytecodeHash": "ipfs",
      // Optional: Use only literal content and not URLs.
      // Default: false.
      "useLiteralContent": true,
      // Optional: Whether to include CBOR-encoded metadata at the end of bytecode.
      // Default: true.
      "appendCBOR": true
    },
    // Optional: Enables the IR codegen in solc.
    "viaIR": true,

    // Optional, solx: Extra LLVM settings.
    "llvmOptions": [
      "-key", "value"
    ]
  }
}
```



## Output JSON

The output JSON contains all artifacts produced by **solx** and **solc** together. The example below serves as the specification of the output JSON format.

```javascript
{
  // Required: File-level outputs.
  "sources": {
    "sourceFile.sol": {
      // Required: Identifier of the source.
      "id": 1,
      // Optional: The AST object.
      // Corresponds to "ast" in the outputSelection settings.
      "ast": {/* ... */}
    }
  },

  // Required: Contract-level outputs.
  "contracts": {
    // The source name.
    "sourceFile.sol": {
      // The contract name.
      // If the language only supports one contract per file, this field equals to the source name.
      "ContractName": {
        // Optional: The Ethereum Contract ABI (object).
        // See https://docs.soliditylang.org/en/develop/abi-spec.html.
        // Corresponds to "abi" in the outputSelection settings.
        "abi": [/* ... */],
        // Optional: Storage layout (object).
        // Corresponds to "storageLayout" in the outputSelection settings.
        "storageLayout": {/* ... */},
        // Optional: Transient storage layout (object).
        // Corresponds to "transientStorageLayout" in the outputSelection settings.
        "transientStorageLayout": {/* ... */},
        // Optional: Contract metadata (string).
        // Corresponds to "metadata" in the outputSelection settings.
        "metadata": "/* ... */",
        // Optional: Developer documentation (natspec object).
        // Corresponds to "devdoc" in the outputSelection settings.
        "devdoc": {/* ... */},
        // Optional: User documentation (natspec object).
        // Corresponds to "userdoc" in the outputSelection settings.
        "userdoc": {/* ... */},
        // Optional: Yul produced by solc (string).
        // Corresponds to "ir" in the outputSelection settings.
        "ir": "/* ... */",
        // Optional: EVM target outputs.
        // Corresponds to "evm" in the outputSelection settings.
        "evm": {
          // Optional: EVM assembly produced by solc (object).
          // Corresponds to "evm.legacyAssembly" in the outputSelection settings.
          "legacyAssembly": {/* ... */},
          // Optional: List of function hashes (object).
          // Corresponds to "evm.methodIdentifiers" in the outputSelection settings.
          "methodIdentifiers": {
            // Mapping between the function signature and its hash.
            "delegate(address)": "5c19a95c"
          },
          // Optional: Always empty, Included only to preserve compatibility with some toolkits (object).
          // Corresponds to "evm.gasEstimates" in the outputSelection settings.
          "gasEstimates": {},
          // Optional: Deploy EVM bytecode.
          // Corresponds to "evm.bytecode" in the outputSelection settings.
          "bytecode": {
            // Optional: Bytecode (string).
            // Corresponds to "evm.bytecode.object" in the outputSelection settings.
            "object": "5b60806040525f341415601c5763...",
            // Optional: LLVM text assembly (string).
            // Corresponds to "evm.bytecode.llvmAssembly" in the outputSelection settings.
            "llvmAssembly": "/* ... */",
            // Optional, solx-only: EVM legacy assembly IR (string). Only available for non-viaIR mode.
            // Corresponds to "evm.bytecode.evmla" in the outputSelection settings.
            "evmla": "/* ... */",
            // Optional, solx-only: Ethereal IR (string). Only available for non-viaIR mode.
            // Corresponds to "evm.bytecode.ethir" in the outputSelection settings.
            "ethir": "/* ... */",
            // Optional, solx-only: Unoptimized LLVM IR (string).
            // Corresponds to "evm.bytecode.llvmIrUnoptimized" in the outputSelection settings.
            "llvmIrUnoptimized": "/* ... */",
            // Optional, solx-only: Optimized LLVM IR (string).
            // Corresponds to "evm.bytecode.llvmIr" in the outputSelection settings.
            "llvmIr": "/* ... */",
            // Optional: ELF-wrapped DWARF debug info (string).
            // Corresponds to "evm.bytecode.debugInfo" in the outputSelection settings.
            "debugInfo": "/* ... */",
            // Optional: Link references for linkers that are to resolve library addresses at deploy time (object).
            // Corresponds to "evm.bytecode.linkReferences" in the outputSelection settings.
            "linkReferences": {/* ... */},
            // Optional: Benchmarks of each stage of the compilation on a per-translation unit basis.
            // Corresponds to "benchmarks" in the outputSelection settings.
            "benchmarks": {/* ... */},
            // Optional: Always empty, included only to preserve compatibility with some toolkits (string).
            // Corresponds to "evm.bytecode.immutableReferences" in the outputSelection settings.
            "opcodes": {},
            // Optional: Always empty, Included only to preserve compatibility with some toolkits (string).
            // Corresponds to "evm.bytecode.sourceMap" in the outputSelection settings.
            "sourceMap": {},
            // Optional: Always empty, Included only to preserve compatibility with some toolkits (array).
            // Corresponds to "evm.bytecode.functionDebugData" in the outputSelection settings.
            "functionDebugData": {},
            // Optional: Always empty, Included only to preserve compatibility with some toolkits (object).
            // Corresponds to "evm.bytecode.generatedSources" in the outputSelection settings.
            "generatedSources": {}
          },
          // Optional: Runtime EVM bytecode.
          // Corresponds to "evm.deployedBytecode" in the outputSelection settings.
          "deployedBytecode": {
            // Optional: Bytecode (string).
            // Corresponds to "evm.deployedBytecode.object" in the outputSelection settings.
            "object": "5b60806040525f34141560145760...",
            // Optional: LLVM text assembly (string).
            // Corresponds to "evm.deployedBytecode.llvmAssembly" in the outputSelection settings.
            "llvmAssembly": "/* ... */",
            // Optional, solx-only: EVM legacy assembly IR (string). Only available for non-viaIR mode.
            // Corresponds to "evm.deployedBytecode.evmla" in the outputSelection settings.
            "evmla": "/* ... */",
            // Optional, solx-only: Ethereal IR (string). Only available for non-viaIR mode.
            // Corresponds to "evm.deployedBytecode.ethir" in the outputSelection settings.
            "ethir": "/* ... */",
            // Optional, solx-only: Unoptimized LLVM IR (string).
            // Corresponds to "evm.deployedBytecode.llvmIrUnoptimized" in the outputSelection settings.
            "llvmIrUnoptimized": "/* ... */",
            // Optional, solx-only: Optimized LLVM IR (string).
            // Corresponds to "evm.deployedBytecode.llvmIr" in the outputSelection settings.
            "llvmIr": "/* ... */",
            // Optional: ELF-wrapped DWARF debug info (string).
            // Corresponds to "evm.deployedBytecode.debugInfo" in the outputSelection settings.
            "debugInfo": "/* ... */",
            // Optional: Link references for linkers that are to resolve library addresses at deploy time (object).
            // Corresponds to "evm.deployedBytecode.linkReferences" in the outputSelection settings.
            "linkReferences": {/* ... */},
            // Optional: Benchmarks of each stage of the compilation on a per-translation unit basis.
            // Corresponds to "benchmarks" in the outputSelection settings.
            "benchmarks": {/* ... */},
            // Optional: Resolved by LLVM automatically, so always returned as an empty object (object).
            // Included only to preserve compatibility with some toolkits.
            // Corresponds to "evm.deployedBytecode.immutableReferences" in the outputSelection settings.
            "immutableReferences": {},
            // Optional: Always empty, included only to preserve compatibility with some toolkits (string).
            // Corresponds to "evm.deployedBytecode.opcodes" in the outputSelection settings.
            "opcodes": {},
            // Optional: Always empty, Included only to preserve compatibility with some toolkits (string).
            // Corresponds to "evm.deployedBytecode.sourceMap" in the outputSelection settings.
            "sourceMap": {},
            // Optional: Always empty, Included only to preserve compatibility with some toolkits (array).
            // Corresponds to "evm.deployedBytecode.functionDebugData" in the outputSelection settings.
            "functionDebugData": {},
            // Optional: Always empty, Included only to preserve compatibility with some toolkits (object).
            // Corresponds to "evm.deployedBytecode.generatedSources" in the outputSelection settings.
            "generatedSources": {}
          }
        },
        // Optional: Benchmarks of the solx LLVM-based compilation pipeline and its underlying call to solc (object).
        // Corresponds to "benchmarks" in the outputSelection settings.
        "benchmarks": {/* ... */}
      }
    }
  },

  // Optional: Unset if no messages were emitted.
  "errors": [
    {
      // Optional: Location within the source file.
      // Unset if the error is unrelated to input sources.
      "sourceLocation": {
        /// Required: The source path.
        "file": "sourceFile.sol",
        /// Required: The source location start. Equals -1 if unknown.
        "start": 0,
        /// Required: The source location end. Equals -1 if unknown.
        "end": 100
      },
      // Required: Message type.
      // solc errors are listed at https://docs.soliditylang.org/en/latest/using-the-compiler.html#error-types.
      "type": "Error",
      // Required: Component the error originates from.
      "component": "general",
      // Required: Message severity.
      // Possible values: "error", "warning", "info".
      "severity": "error",
      // Optional: Unique code for the cause of the error.
      // Only solc produces error codes for now.
      // solx currently emits errors without codes, but they will be introduced soon.
      "errorCode": "3141",
      // Required: Message.
      "message": "Invalid keyword",
      // Required: Message formatted using the source location.
      "formattedMessage": "sourceFile.sol:100: Invalid keyword"
    }
  ]
}
```
