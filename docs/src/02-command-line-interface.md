# Command Line Interface (CLI)

The CLI of **solx** is designed to mimic that of **solc**. There are several main input/output (I/O) modes in the **solx** interface:

- [Basic CLI](#basic-cli)
- [Standard JSON](./03-standard-json.md)

The basic CLI is simpler and suitable for using from the shell. The standard JSON mode is similar to client-server interaction, thus more suitable for using from other applications.

> All toolkits using **solx** must be operating in standard JSON mode and follow [its specification](./03-standard-json.md).
> It will make the toolkits more robust and future-proof, as the standard JSON mode is the most versatile and used for the majority of popular projects.

This page focuses on the basic CLI mode. For more information on the standard JSON mode, see [this page](./03-standard-json.md).



## Basic CLI

Basic CLI mode is the simplest way to compile a file with the source code.

To compile a basic Solidity contract, run the simple example from [the *--bin* section](#--bin).

The rest of this section describes the available CLI options and their usage. You may also check out `solx --help` for a quick reference.



### `--bin`

Emits the full bytecode.

```bash
solx 'Simple.sol' --bin
```

Output:

```text
======= Simple.sol:Simple =======
Binary:
5b60806040525f341415601c5763...
```



### `--bin`

Emits the runtime part of the bytecode.

```bash
solx 'Simple.sol' --bin-runtime
```

Output:

```text
======= Simple.sol:Simple =======
Binary of the runtime part:
5b60806040525f34141560145760...
```



### `--asm`

Emits the text assembly produced by LLVM.

```bash
solx 'Simple.sol' --asm
```

Output:

```text
======= Simple.sol:Simple =======
Deploy LLVM EVM assembly:
        .text
        .file   "Simple.sol:Simple"
main:
.func_begin0:
        JUMPDEST
        PUSH1 128
        PUSH1 64
...

Runtime LLVM EVM assembly:
        .text
        .file   "Simple.sol:Simple.runtime"
main:
.func_begin0:
        JUMPDEST
        PUSH1 128
        PUSH1 64
...
```



### `--metadata`

Emits the contract metadata. The metadata is a JSON object that contains information about the contract, such as its name, source code hash, the list of dependencies, compiler versions, and so on.

The **solx** metadata format is compatible with the [Solidity metadata format](https://docs.soliditylang.org/en/latest/metadata.html#contract-metadata). This means that the metadata output can be used with other tools that support Solidity metadata. Extra **solx** data is inserted into **solc** metadata with this JSON object:

```javascript
{
  "solx": {
    "llvm_options": [],
    "optimizer_settings": {
      "is_debug_logging_enabled": false,
      "is_fallback_to_size_enabled": false,
      "is_verify_each_enabled": false,
      "level_back_end": "Aggressive",
      "level_middle_end": "Aggressive",
      "level_middle_end_size": "Zero"
    },
    // Optional: only set for Solidity and Yul contracts.
    "solc_llvm_revision": "1.0.2",
    // Optional: only set for Solidity and Yul contracts.
    "solc_version": "0.8.33",
    // Mandatory: current version of solx.
    "solx_version": "0.1.3"
  }
}
```

Usage:

```bash
solx 'Simple.sol' --metadata
```

Output:

```text
======= Simple.sol:Simple =======
Metadata:
{"compiler":{"version":"0.8.33+commit.64118f21"},"language":"Solidity","output":{"abi":[{"inputs":[],"name":"first","outputs":[{"internalType":"uint64","name":"","type":"uint64"}],"stateMutability":"pure","type":"function"},{"inputs":[],"name":"second","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"pure","type":"function"}],"devdoc":{"kind":"dev","methods":{},"version":1},"userdoc":{"kind":"user","methods":{},"version":1}},"settings":{"compilationTarget":{"Simple.sol":"Simple"},"evmVersion":"cancun","libraries":{},"metadata":{"bytecodeHash":"ipfs"},"optimizer":{"enabled":true,"runs":200},"remappings":[]},"solx":{"llvm_options":[],"optimizer_settings":{"is_debug_logging_enabled":false,"is_fallback_to_size_enabled":false,"is_verify_each_enabled":false,"level_back_end":"Aggressive","level_middle_end":"Aggressive","level_middle_end_size":"Zero"},"solc_llvm_revision":"1.0.2","solc_version":"0.8.33","solx_version":"0.1.0"},"sources":{"Simple.sol":{"keccak256":"0x1145e81d58e9fd0859036aac4ba16cfcfbe11045e3dfd5105a2dca469f31db89","license":"MIT","urls":["bzz-raw://9d97789b5c14a95fac1e7586de6712119f4606f79d6771324c9d24417ebab0db","dweb:/ipfs/QmSZ3HNGZom6N6eb8d74Y7UQAKAGRkXgbinwVVLaiuGb3S"]}},"version":1}
```



### `--ast-json`

Emits the AST of each Solidity file.

```bash
solx 'Simple.sol' --ast-json
```

Output:

```text
======= Simple.sol:Simple =======
JSON AST:
{"absolutePath":".../Simple.sol","exportedSymbols":{"Test":[26]},"id":27,"license":"MIT","nodeType":"SourceUnit","nodes":[ ... ],"src":"506:265:0"}
```

> Since **solx** communicates with **solc** only via standard JSON under the hood, full JSON AST instead is emitted instead of the compact one.



### `--abi`

Emits the contract ABI specification.

```bash
solx 'Simple.sol' --abi
```

Output:

```text
======= Simple.sol:Simple =======
Contract JSON ABI:
[{"inputs":[],"name":"first","outputs":[{"internalType":"uint64","name":"","type":"uint64"}],"stateMutability":"pure","type":"function"},{"inputs":[],"name":"second","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"pure","type":"function"}]
```



### `--hashes`

Emits the contract function signatures.

```bash
solx 'Simple.sol' --hashes
```

Output:

```text
======= Simple.sol:Simple =======
Function signatures:
3df4ddf4: first()
5a8ac02d: second()
```



### `--storage-layout`

Emits the contract storage layout.

```bash
solx 'Simple.sol' --storage-layout
```

Output:

```text
======= Simple.sol:Simple =======
Contract Storage Layout:
{"storage":[{"astId":3,"contract":"Simple.sol:Simple","label":"field_1","offset":0,"slot":"0","type":"t_uint256"},{"astId":5,"contract":"Simple.sol:Simple","label":"field_2","offset":0,"slot":"1","type":"t_uint256"},{"astId":7,"contract":"Simple.sol:Simple","label":"field_3","offset":0,"slot":"2","type":"t_uint256"}],"types":{"t_uint256":{"encoding":"inplace","label":"uint256","numberOfBytes":"32"}}}
```



### `--transient-storage-layout`

Emits the contract transient storage layout.

```bash
solx 'Simple.sol' --transient-storage-layout
```

Output:

```text
======= Simple.sol:Simple =======
Contract Transient Storage Layout:
{"storage":[{"astId":3,"contract":"Simple.sol:Simple","label":"field_1","offset":0,"slot":"0","type":"t_uint256"},{"astId":5,"contract":"Simple.sol:Simple","label":"field_2","offset":0,"slot":"1","type":"t_uint256"},{"astId":7,"contract":"Simple.sol:Simple","label":"field_3","offset":0,"slot":"2","type":"t_uint256"}],"types":{"t_uint256":{"encoding":"inplace","label":"uint256","numberOfBytes":"32"}}}
```



### `--userdoc`

Emits the contract user documentation.

```bash
solx 'Simple.sol' --userdoc
```

Output:

```text
======= Simple.sol:Simple =======
User Documentation:
{"kind":"user","methods":{ ... },"version":1}
```



### `--devdoc`

Emits the contract developer documentation.

```bash
solx 'Simple.sol' --devdoc
```

Output:

```text
======= Simple.sol:Simple =======
Developer Documentation:
{"kind":"dev","methods":{ ... },"version":1}
```



### `--asm-solc-json`

Emits the **solc** EVM assembly in JSON format.

```bash
solx 'Simple.sol' --asm-solc-json
```

Output:

```text
======= Simple.sol:Simple =======
EVM assembly:
{".auxdata":null,".code":[ ... ],".data":{"0":{".auxdata":"a26469706673582212202644e52ba9ffa2e1d55713f314f19bc59467d1342b170ca4ce0e2d6d0e7afda664736f6c634300081e0033",".code":[ ... ],".data":null}},"full_path":"Simple.sol:Simple"}
```

> This is the **solc** EVM assembly output that is translated to LLVM IR by **solx**. For **solx**'s own EVM assembly output emitted by LLVM, use the [`--asm`](#--asm) option instead.



### `--ir` (or `--ir-optimized`)

Emits the **solc** Yul IR.

> **solx** does not use the Yul optimizer anymore, so the Yul IR is always unoptimized, and it is not possible to emit solc-optimized Yul IR with **solx**.

```bash
solx 'Simple.sol' --ir
```

Output:

```text
======= Simple.sol:Simple =======
Optimized IR:
/// @use-src 0:"Simple.sol:Simple"
object "Test_26" {
    code {
        {
            ...
        }
    }
    /// @use-src 0:"Simple.sol:Simple"
    object "Test_26_deployed" {
        code {
            {
                ...
            }
        }
        data ".metadata" hex"a2646970667358221220ccbacddd0734a08eedf3a12f841c7e8c7127e2457a91f6e68f5fac6df7b9c88a64736f6c634300081e0033"
    }
}
```



### `--debug-info`

Emits the ELF-wrapped DWARF debug info of the deploy code.

```bash
solx 'Simple.sol' --debug-info
```

Output:

```text
======= Simple.sol:Simple =======
Debug info:
7f454c46010201ff...
```



### `--debug-info-runtime`

Emits the ELF-wrapped DWARF debug info of the runtime code.

```bash
solx 'Simple.sol' --debug-info-runtime
```

Output:

```text
======= Simple.sol:Simple =======
Debug info of the runtime part:
7f454c46010201ff
```



### `--benchmarks`

Emits benchmarks of the **solx** LLVM-based pipeline and its underlying call to **solc**.

```bash
solx 'Simple.sol' --benchmarks
```

Output:

```text
Benchmarks:
solc_Solidity_Standard_JSON: 6ms
solx_Solidity_IR_Analysis: 0ms
solx_Compilation: 75ms

======= Simple.sol:Simple =======
Benchmarks:
    Simple.sol:Simple:deploy/EVMAssemblyToLLVMIR/M3B3/SpillArea(0): 0ms
    Simple.sol:Simple:deploy/InitVerify/M3B3/SpillArea(0): 0ms
    Simple.sol:Simple:deploy/OptimizeVerify/M3B3/SpillArea(0): 1ms
    Simple.sol:Simple:runtime/EVMAssemblyToLLVMIR/M3B3/SpillArea(0): 0ms
    Simple.sol:Simple.runtime:runtime/InitVerify/M3B3/SpillArea(0): 0ms
    Simple.sol:Simple.runtime:runtime/OptimizeVerify/M3B3/SpillArea(0): 5ms
```



### Input Files

**solx** supports multiple input files. The following command compiles two Solidity files and prints the bytecode:

```bash
solx 'Simple.sol' 'Complex.sol' --bin
```

[Solidity import remappings](https://docs.soliditylang.org/en/latest/path-resolution.html#import-remapping) are passed in the way as input files, but they are distinguished by a `=` symbol between source and destination. The following command compiles a Solidity file with a remapping and prints the bytecode:

```bash
solx 'Simple.sol' 'github.com/ethereum/dapp-bin/=/usr/local/lib/dapp-bin/' --bin
```

**solx** does not handle remappings itself, but only passes them through to *solc*.
Visit [the **solc** documentation](https://docs.soliditylang.org/en/latest/using-the-compiler.html#base-path-and-import-remapping) to learn more about the processing of remappings.



### `--libraries`

Specifies the libraries to link with compiled contracts. The option accepts multiple string arguments. The safest way is to wrap each argument in single quotes, and separate them with a space.

The specifier has the following format: `<ContractPath>:<ContractName>=<LibraryAddress>`.

Usage:

```bash
solx 'Simple.sol' --bin --libraries 'Simple.sol:Test=0x1234567890abcdef1234567890abcdef12345678'
```



### `--base-path`, `--include-path`, `--allow-paths`

These options are used to specify Solidity import resolution settings. They are not used by **solx** and only passed through to **solc** like import remappings.

Visit [the **solc** documentation](https://docs.soliditylang.org/en/latest/path-resolution.html) to learn more about the processing of these options.



### `--output-dir`

Specifies the output directory for build artifacts. Can only be used in [basic CLI](#basic-cli) mode.

Usage in basic CLI mode:

```bash
solx 'Simple.sol' --bin --asm --metadata --output-dir './build/'
ls './build/Simple.sol'
```

Output:

```text
Compiler run successful. Artifact(s) can be found in directory "build".
...
Test.zasm       Test.zbin       Test_meta.json
```



### `--overwrite`

Overwrites the output files if they already exist in the output directory. By default, **solx** does not overwrite existing files.

Can only be used in combination with the [`--output-dir`](#--output-dir) option.

Usage:

```bash
solx 'Simple.sol' --bin --output-dir './build/' --overwrite
```

If the `--overwrite` option is not specified and the output files already exist, **solx** will print an error message and exit:

```text
Error: Refusing to overwrite an existing file "./build/Simple.sol/Test.bin" (use --overwrite to force).
```



### `--version`

Prints the version of **solx** and the hash of the LLVM commit it was built with.

Usage:

```bash
solx --version
```



### `--help`

Prints the help message.

Usage:

```bash
solx --help
```



## Other I/O Modes

The mode-altering CLI options are mutually exclusive. This means that only one of the options below can be enabled at a time:

- [`--standard-json`](#--standard-json)
- [`--yul`](#--yul)
- [`--llvm-ir`](#--llvm-ir)



### `--standard-json`

For the standard JSON mode usage, see the [Standard JSON](./03-standard-json.md) page.



## **solx** Compilation Settings

The options in this section are only configuring the **solx** compiler and do not affect the underlying **solc** compiler.



### `--optimization / -O`

Sets the optimization level of the LLVM optimizer. Available values are:

| Level | Meaning                      | Hints                                            |
|:------|:-----------------------------|:-------------------------------------------------|
| 0     | No optimization              | For fast compilation during development (unsupported)
| 1     | Performance: basic           | For optimization research
| 2     | Performance: default         | For optimization research
| 3     | Performance: aggressive      | Best performance for production
| s     | Size: default                | For optimization research
| z     | Size: aggressive             | Best size for contracts with size constraints

For most cases, it is fine to keep the default value of `3`. You should only use the level `z` if you are ready to deliberately sacrifice performance and optimize for size.

> Large contracts may hit the EVM bytecode size limit. In this case, it is recommended to use the [`--optimization-size-fallback`](#--optimization-size-fallback) option rather than setting the level to `z`.

Usage:

```bash
solx 'Simple.sol' --bin -O3
```

This option can also be set with an environment variable `SOLX_OPTIMIZATION`, which is useful for toolkits
where arbitrary solx-specific options are not supported:

```bash
SOLX_OPTIMIZATION='3' solx 'Simple.sol' --bin
```



### `--optimization-size-fallback`

Sets the optimization level to `z` for contracts that failed to compile due to overrunning the bytecode size constraints.

Under the hood, this option automatically triggers recompilation of contracts with level `z`. Contracts that were successfully compiled with [the original `--optimization` setting](#--optimization---o) are not recompiled.

> For deployment, it is recommended to have this option enabled in order to mitigate potential issues with EVM bytecode size constraints on a per-contract basis.
> If your environment does not have bytecode size limitations, it is better to disable it to prevent unnecessary recompilations. A good example is running `forge test`.

Usage:

```bash
solx 'Simple.sol' --bin -O3 --optimization-size-fallback
```

This option can also be set with an environment variable `SOLX_OPTIMIZATION_SIZE_FALLBACK`, which is useful for toolkits
where arbitrary solx-specific options are not supported:

```bash
SOLX_OPTIMIZATION_SIZE_FALLBACK= solx 'Simple.sol' --bin -O3
```



### `--metadata-hash`

Specifies the hash format used for contract metadata.

Usage with `ipfs`:

```bash
solx 'Simple.sol' --bin --metadata-hash 'ipfs'
```

Output with `ipfs`:

```text
======= Simple.sol:Simple =======
Binary:
5b60806040525f341415601c5763000000488063000000245f395ff35b5f80...
a2646970667358221220ba14ea4e52366f139a845913d41e98933393bd1c1126331611687003d4aa92de64736f6c6378247a6b736f6c633a312e352e31333b736f6c633a302e382e32393b6c6c766d3a312e302e310055
```

The byte array starting with `a2` at the end of the bytecode is a CBOR-encoded compiler version data and an optional metadata hash.

The last two bytes of the metadata (`0x0055`) are not a part of the CBOR payload, but the length of it, which must be known to correctly decode the payload. 

JSON representation of the CBOR payload:

```javascript
{
    // Optional: included if `--metadata-hash` is set to `ipfs`.
    "ipfs": "1220ba14ea4e52366f139a845913d41e98933393bd1c1126331611687003d4aa92de",

    // Required: consists of semicolon-separated pairs of colon-separated compiler names and versions.
    // `solx:<version>` is always included.
    // `solc:<version>;llvm:<version>` is only included for Solidity and Yul contracts, but not included for LLVM IR ones.
    // `llvm` stands for the revision of Matter Labs fork of solc, that solx is statically linked with.
    "solc": "solx:0.1.3;solc:0.8.33;llvm:1.0.2"
}
```

For more information on these formats, see the [CBOR](https://cbor.io/) and [IPFS](https://docs.ipfs.tech/) documentation.



### `--no-cbor-metadata`

Disables the CBOR metadata that is appended at the end of bytecode. This option is useful for debugging and research purposes.

> It is not recommended to use this option in production, as it is not possible to verify contracts deployed without metadata.

Usage:

```shell
solx 'Simple.sol' --no-cbor-metadata
```



### `--llvm-options`

Specifies additional options for the LLVM framework. The argument must be a single quoted string following a `=` separator.

Usage:

```bash
solx 'Simple.sol' --bin --llvm-options='-key=value'
```

> The `--llvm-options` option is experimental and must only be used by experienced users. All supported options will be documented in the future.



## **solc** Compilation Settings

The options in this section are only configuring **solc**, so they are passed directly to its child process, and do not affect the **solx** compiler.



### `--via-ir`

Switches the **solc** codegen to Yul a.k.a. IR.

Usage:

```bash
solx 'Simple.sol' --bin --via-ir
```



### `--evm-version`

Specifies the EVM version **solx** will produce bytecode for. For instance, with version *osaka*, **solx** will be producing `clz` instructions, whereas for older EVM versions it will not.

Only the following EVM versions are supported:

- cancun
- prague
- osaka

Usage:

```bash
solx 'Simple.sol' --bin --evm-version 'osaka'
```



### `--metadata-literal`

Tells **solc** to store referenced sources as literal data in the metadata output.

> This option only affects the contract metadata output produced by **solc**, and does not affect artifacts produced by **solx**.

Usage:

```bash
solx 'Simple.sol' --bin --metadata --metadata-literal
```



### `--no-import-callback`

Disables the default import resolution callback in **solc**.

> This parameter is used by some tooling that resolves all imports by itself, such as Hardhat.

Usage:

```shell
solx 'Simple.sol' --no-import-callback
```



## Multi-Language Support

**solx** supports input in multiple programming languages:

- [Solidity](https://soliditylang.org/)
- [Yul](https://docs.soliditylang.org/en/latest/yul.html)
- [LLVM IR](https://llvm.org/docs/LangRef.html)

The following sections outline how to use **solx** with these languages.



### `--yul` (or `--strict-assembly`)

Enables the Yul mode. In this mode, input is expected to be in the Yul language. The output works the same way as with Solidity input.

Usage:

```bash
solx --yul 'Simple.yul' --bin
```

Output:

```text
======= Simple.yul =======
Binary:
5b60806040525f341415601c5763...
```



### `--llvm-ir`

Enables the LLVM IR mode. In this mode, input is expected to be in the LLVM IR language. The output works the same way as with Solidity input.

> In this mode, every input file is treated as runtime code, while deploy code will be generated automatically by **solx**.
> It is not possible to write deploy code manually yet, but it will be supported in the future.

Unlike **solc**, **solx** is an LLVM-based compiler toolchain, so it uses LLVM IR as an intermediate representation. It is not recommended to write LLVM IR manually, but it can be useful for debugging and optimization purposes. LLVM IR is more low-level than Yul and EVM assembly in the **solx** IR hierarchy.

Usage:

```bash
solx --llvm-ir 'Simple.ll' --bin
```

Output:

```text
======= Simple.ll =======
Binary:
5b60806040525f341415601c5763...
```



## Debugging



### `--debug-output-dir`

Specifies the directory to store intermediate build artifacts. The artifacts can be useful for debugging and research.

The directory is created if it does not exist. If artifacts are already present in the directory, they are overwritten.

The intermediate build artifacts can be:

| Name          | Extension   |
|:--------------|:------------|
| EVM Assembly  | *evmla*     |
| EthIR         | *ethir*     |  
| Yul           | *yul*       |
| LLVM IR       | *ll*        |

Usage:

```bash
solx 'Simple.sol' --bin --debug-output-dir './debug/'
ls './debug/'
```

This option can also be set with an environment variable `SOLX_DEBUG_OUTPUT_DIR`, which is useful for toolkits
where arbitrary solx-specific options are not supported:

```bash
SOLX_DEBUG_OUTPUT_DIR='./debug/' solx 'Simple.sol' --bin
ls './debug/'
```

Additionally, it is possible to dump the standard JSON input file with the `SOLX_STANDARD_JSON_DEBUG` environment variable:

```bash
SOLX_STANDARD_JSON_DEBUG='./debug/input.json' solx 'Simple.sol' --bin
cat './debug/input.json' | jq .
```

Output:

```text
Compiler run successful. No output generated.
...
Simple.sol_Test.evmla
Simple.sol_Test.ethir
Simple.sol_Test.unoptimized.ll
Simple.sol_Test.optimized.ll
Simple.sol_Test.asm
Simple.sol_Test.runtime.evmla
Simple.sol_Test.runtime.ethir
Simple.sol_Test.runtime.unoptimized.ll
Simple.sol_Test.runtime.optimized.ll
Simple.sol_Test.runtime.asm
```

The output file name is constructed as follows: `<ContractPath>_<ContractName>.<Modifiers>.<Extension>`.



### `--llvm-verify-each`

Enables the verification of the LLVM IR after each optimization pass. This option is useful for debugging and research purposes.

Usage:

```bash
solx 'Simple.sol' --bin --llvm-verify-each
```



### `--llvm-debug-logging`

Enables the debug logging of the LLVM IR optimization passes. This option is useful for debugging and research purposes.

Usage:

```bash
solx 'Simple.sol' --bin --llvm-debug-logging
```
