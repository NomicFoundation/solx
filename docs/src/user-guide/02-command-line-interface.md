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

All examples on this page are executable and verified against the compiler on every change. Unless stated otherwise, they compile this contract, `Simple.sol`:

```solidity
{{#include 02-command-line-interface.in/Simple.sol}}
```

The examples use the following notation:

- `$` precedes the command being run; the lines below it show the command's output.
- `[..]` matches any text within a line — used for values that differ between runs or environments, such as timings and hashes.
- `...` on its own line elides a run of output lines.
- `? failed` marks a command that exits with a non-zero code.

All commands on this page run in one shared working directory, so sections that emit files use per-section output directories (`./build-llvm-ir/`, `./build-asm/`, …) to keep each listing scoped to its own artifacts.



### `--bin`

Emits the full bytecode.

```console
$ solx 'Simple.sol' --bin

======= Simple.sol:Simple =======
Binary:
5b3460485763000000c38038036080601f19601f8301160191680100000000000000008310607f19601f8401101615604c5782604052608039630000005f908163000000648239f35b5f5ffd5b505050634e487b7160e01b5f52604160045260245ffdfe5b60043610603a575f3560e01c635a8ac02d8114602f57633df4ddf403603a5734603a5760015b60805260206080f35b5034603a5760026026565b5f5ffdfea164736f6c637816736f6c783a302e312e383b736f6c633a302e382e3337001e

```



### `--bin-runtime`

Emits the runtime part of the bytecode.

```console
$ solx 'Simple.sol' --bin-runtime

======= Simple.sol:Simple =======
Binary of the runtime part:
5b60043610603a575f3560e01c635a8ac02d8114602f57633df4ddf403603a5734603a5760015b60805260206080f35b5034603a5760026026565b5f5ffdfea164736f6c637816736f6c783a302e312e383b736f6c633a302e382e3337001e

```



### `--asm`

Emits the text assembly produced by LLVM.

```console
$ solx 'Simple.sol' --asm

======= Simple.sol:Simple =======
Deploy LLVM EVM assembly:
	.file	"LLVMDialectModule"
	.text
	.globl	__entry                         ; -- Begin function __entry
__entry:                                ; @__entry
; %bb.0:
	JUMPDEST
	CALLVALUE
	PUSH4           @.BB0_3
...

Runtime LLVM EVM assembly:
	.file	"LLVMDialectModule"
	.text
	.globl	__entry                         ; -- Begin function __entry
__entry:                                ; @__entry
; %bb.0:
	JUMPDEST
	PUSH1           0x4
	CALLDATASIZE
...
```



### `--metadata`

Emits the contract metadata. The metadata is a JSON object that contains information about the contract, such as its name, source code hash, the list of dependencies, compiler versions, and so on.

The **solx** metadata format is compatible with the [Solidity metadata format](https://docs.soliditylang.org/en/latest/metadata.html#contract-metadata). This means that the metadata output can be used with other tools that support Solidity metadata. The metadata that is hashed into [the CBOR trailer of the bytecode](#--metadata-hash) additionally carries extra **solx** data inserted into the metadata with this JSON object:

```javascript
{
  "slang": {
    "llvm_options": [],
    "optimizer_settings": {
      "is_debug_logging_enabled": false,
      "is_fallback_to_size_enabled": false,
      "is_verify_each_enabled": false,
      "level_back_end": "Aggressive",
      "level_middle_end": "Aggressive",
      "level_middle_end_size": "Zero"
    },
    // Optional: the Solidity language version, only set for Solidity and Yul contracts.
    "solc_version": "0.8.37",
    // Mandatory: current version of solx.
    "solx_version": "0.1.8"
  }
}
```

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --metadata
```



### `--ast-json`

Emits the AST of each Solidity file.

```console
$ solx 'Simple.sol' --ast-json

======= Simple.sol =======
JSON AST:
{"id":1,"type":"SourceUnit","range":{"start":32,"end":319},"file":"Simple.sol","members":[[..]]}

======= Simple.sol:Simple =======

```

The AST body is abbreviated here; it spans several thousand characters even for this small contract.



### `--abi`

Emits the contract ABI specification.

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --abi
```



### `--hashes`

Emits the contract function signatures.

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --hashes
```



### `--storage-layout`

Emits the contract storage layout.

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --storage-layout
```



### `--transient-storage-layout`

Emits the contract transient storage layout.

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --transient-storage-layout
```



### `--userdoc`

Emits the contract user documentation.

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --userdoc
```



### `--devdoc`

Emits the contract developer documentation.

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --devdoc
```



### `--debug-info`

Emits the ELF-wrapped DWARF debug info of the deploy code.

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --debug-info
```



### `--debug-info-runtime`

Emits the ELF-wrapped DWARF debug info of the runtime code.

Not emitted by the Slang frontend yet. Usage:

```bash
solx 'Simple.sol' --debug-info-runtime
```



### `--emit-llvm-ir`

Emits LLVM IR (both unoptimized and optimized).

When used with `--output-dir`, writes `.ll` files to the output directory. Without `--output-dir`, outputs to stdout.

Usage with `--output-dir`:

```console
$ solx 'Simple.sol' --emit-llvm-ir --output-dir './build-llvm-ir/'
Compiler run successful. Artifact(s) can be found in directory "./build-llvm-ir/".

$ ls './build-llvm-ir/'
Simple_sol_Simple.optimized.ll
Simple_sol_Simple.unoptimized.ll
Simple_sol_Simple_deployed.optimized.ll
Simple_sol_Simple_deployed.unoptimized.ll

```

Usage with stdout:

```console
$ solx 'Simple.sol' --emit-llvm-ir --bin

======= Simple.sol:Simple =======
Binary:
...
Deploy LLVM IR (unoptimized):
; ModuleID = 'Simple.sol:Simple'
...
Deploy LLVM IR:
; ModuleID = 'Simple.sol:Simple'
...
```



### `--benchmarks`

Emits benchmarks of the compilation pipeline.

```console
$ solx 'Simple.sol' --benchmarks
Benchmarks:
Slang_RunStandardJSON: [..]us
solx_BuildProject: [..]us
solx_Compile: [..]us
Slang_ParseAndBind: [..]us
Slang_SerializeAST:Simple.sol: [..]us
solx_CreateMLIRContext:Simple.sol: [..]us
solx_EmitSol:Simple.sol:Simple: [..]us
solx_RunSolPasses:Simple.sol:Simple: [..]us
solx_ExtractMLIRObjects:Simple.sol:Simple: [..]us

======= Simple.sol:Simple =======
Benchmarks:
    Simple.sol:Simple:deploy/CreateMLIRContext/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:deploy/ParseMLIR/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:deploy/MLIRToLLVMIR/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:deploy/InitVerify/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:deploy/OptimizeVerify/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:deploy/WorkerRoundtrip(0)/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:runtime/CreateMLIRContext/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:runtime/ParseMLIR/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:runtime/MLIRToLLVMIR/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple_deployed:runtime/InitVerify/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple_deployed:runtime/OptimizeVerify/M3B3/SpillArea(0): [..]us
    Simple.sol:Simple:runtime/WorkerRoundtrip(0)/M3B3/SpillArea(0): [..]us

```



### Input Files

**solx** supports multiple input files. The following command compiles two Solidity files and prints the bytecode:

```bash
solx 'Simple.sol' 'Complex.sol' --bin
```

[Solidity import remappings](https://docs.soliditylang.org/en/latest/path-resolution.html#import-remapping) are passed the same way as input files, but they are distinguished by a `=` symbol between source and destination. The following command compiles a Solidity file with a remapping and prints the bytecode:

```bash
solx 'Simple.sol' 'github.com/ethereum/dapp-bin/=/usr/local/lib/dapp-bin/' --bin
```

**solx** applies remappings following **solc**'s semantics.
Visit [the **solc** documentation](https://docs.soliditylang.org/en/latest/using-the-compiler.html#base-path-and-import-remapping) to learn more about the processing of remappings.



### `--libraries`

Specifies the libraries to link with compiled contracts. The option accepts multiple string arguments. The safest way is to wrap each argument in single quotes, and separate them with a space.

The specifier has the following format: `<ContractPath>:<ContractName>=<LibraryAddress>`.

Usage:

```bash
solx 'Simple.sol' --bin --libraries 'Simple.sol:Simple=0x1234567890abcdef1234567890abcdef12345678'
```



### `--base-path`, `--include-path`, `--allow-paths`

These options are accepted for **solc** compatibility. The **Slang** frontend ignores them and resolves imports only against the sources it is given.

Visit [the **solc** documentation](https://docs.soliditylang.org/en/latest/path-resolution.html) to learn more about the processing of these options.



### `--output-dir`

Specifies the output directory for build artifacts. Can only be used in [basic CLI](#basic-cli) mode.

Usage in basic CLI mode:

```console
$ solx 'Simple.sol' --bin --asm --output-dir './build/'
Compiler run successful. Artifact(s) can be found in directory "./build/".

$ ls './build/'
Simple_sol_Simple.asm
Simple_sol_Simple.bin
Simple_sol_Simple_deployed.asm
Simple_sol_Simple_llvm.asm
Simple_sol_Simple_llvm.asm-runtime

```



### `--overwrite`

Overwrites the output files if they already exist in the output directory. By default, **solx** does not overwrite existing files.

Can only be used in combination with the [`--output-dir`](#--output-dir) option.

Usage:

```console
$ solx 'Simple.sol' --bin --output-dir './build/' --overwrite
Compiler run successful. Artifact(s) can be found in directory "./build/".

```

Here `./build/` already contains the artifacts emitted in [the `--output-dir` section](#--output-dir), so the flag has existing files to overwrite.

If the `--overwrite` option is not specified and the output files already exist, **solx** refuses to overwrite them and exits with an error:

```console
$ solx 'Simple.sol' --bin --output-dir './build-overwrite/'
Compiler run successful. Artifact(s) can be found in directory "./build-overwrite/".

$ solx 'Simple.sol' --bin --output-dir './build-overwrite/'
? failed
Error: Refusing to overwrite an existing file "./build-overwrite/Simple_sol_Simple.bin" (use --overwrite to force).

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
- [`--yul`](#--yul-or---strict-assembly)
- [`--llvm-ir`](#--llvm-ir)



### `--standard-json`

For the standard JSON mode usage, see the [Standard JSON](./03-standard-json.md) page.



## **solx** Compilation Settings

The options in this section configure the **solx** compilation pipeline.



### `--threads`

Sets the number of threads used for parallel compilation. Each thread compiles a separate translation unit in a child process. By default, the number of threads equals the number of CPU cores.

> Large projects can consume a lot of RAM during compilation on machines with a high number of cores.
> If you encounter memory issues, consider reducing the number of threads.

Usage:

```bash
solx 'Simple.sol' --bin --threads 4
```



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

> Optimization can affect exact gas consumption, memory expansion, and the preservation of source operations.
> See [Optimizer and Assembly Semantics](./04-limitations.md#optimizer-and-assembly-semantics).

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

```console
$ solx 'Simple.sol' --bin --metadata-hash 'ipfs'

======= Simple.sol:Simple =======
Binary:
5b3460485763000000c38038036080601f19601f8301160191680100000000000000008310607f19601f8401101615604c5782604052608039630000005f908163000000648239f35b5f5ffd5b505050634e487b7160e01b5f52604160045260245ffdfe5b60043610603a575f3560e01c635a8ac02d8114602f57633df4ddf403603a5734603a5760015b60805260206080f35b5034603a5760026026565b5f5ffdfea164736f6c637816736f6c783a302e312e383b736f6c633a302e382e3337001e

```

The byte array starting with `a1` at the end of the bytecode is a CBOR-encoded compiler version data and, when the contract metadata is emitted, its hash.

The last two bytes of the metadata (`0x001e`) are not a part of the CBOR payload, but the length of it, which must be known to correctly decode the payload.

JSON representation of the CBOR payload:

```javascript
{
    // Optional: included if `--metadata-hash` is set to `ipfs` and the contract metadata is emitted.
    "ipfs": "1220bec8fa0149a786c5810200ef5a436a154cff832af68ace5beeabcbb82166cb92",

    // Required: consists of semicolon-separated pairs of colon-separated compiler names and versions.
    // `solx:<version>` is always included.
    // `solc:<version>` is the Solidity language version and is only included for Solidity and Yul contracts, but not included for LLVM IR ones.
    "solc": "solx:0.1.8;solc:0.8.37"
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



## Frontend Settings

The options in this section configure the **Slang** frontend.



### `--evm-version`

Specifies the EVM version **solx** will produce bytecode for. For instance, with version *osaka*, **solx** will be producing `clz` instructions, whereas for older EVM versions it will not.

Only the following EVM versions are supported:

- cancun
- prague
- osaka (default)

Usage:

```bash
solx 'Simple.sol' --bin --evm-version 'osaka'
```



### `--metadata-literal`

Stores referenced sources as literal data in the metadata output.

Usage:

```bash
solx 'Simple.sol' --bin --metadata --metadata-literal
```



### `--no-import-callback`

Disables the default import resolution callback.

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

Yul input is optimized through LLVM and is not emitted as a verbatim EVM opcode sequence. See [Optimizer and Assembly Semantics](./04-limitations.md#optimizer-and-assembly-semantics).

The **Slang** frontend does not accept Yul input yet, so the mode reports an error. The example passes this Yul object, `Simple.yul`:

```yul
{{#include 02-command-line-interface.in/Simple.yul}}
```

Usage:

```console
$ solx --yul 'Simple.yul' --bin
? 1
Error: Slang frontend only supports Solidity sources.

```



### `--llvm-ir`

Enables the LLVM IR mode. In this mode, input is expected to be in the LLVM IR language. The output works the same way as with Solidity input.

> In this mode, every input file is treated as runtime code, while deploy code will be generated automatically by **solx**.
> It is not possible to write deploy code manually yet, but it will be supported in the future.

Unlike **solc**, **solx** is an LLVM-based compiler toolchain, so it uses LLVM IR as an intermediate representation. It is not recommended to write LLVM IR manually, but it can be useful for debugging and optimization purposes. LLVM IR is more low-level than Yul and EVM assembly in the **solx** IR hierarchy.

The example input `Simple.ll` is the optimized runtime module of `Simple.sol` above, as produced by [`--emit-llvm-ir`](#--emit-llvm-ir).

Usage:

```console
$ solx --llvm-ir 'Simple.ll' --bin

======= Simple.ll =======
Binary:
5b630000004f8063000000115f395ff3fe34600b57600336116016575b5f5ffd5b5060016031565b5f3560e01c633df4ddf48114600f57635a8ac02d03600b5760025b60805260206080f3fea164736f6c63780a736f6c783a302e312e380012

```



## Debugging


### IR Output Flags

For selective IR output, use the following flags with `--output-dir`:

- [`--emit-llvm-ir`](#--emit-llvm-ir) - LLVM IR (unoptimized and optimized)
- [`--asm`](#--asm) - LLVM EVM assembly

These flags respect the `--overwrite` option. Without `--overwrite`, the compiler will refuse to overwrite existing files.


### `SOLX_OUTPUT_DIR` Environment Variable

For debugging purposes, all intermediate build artifacts can be dumped to a directory using the `SOLX_OUTPUT_DIR` environment variable. This is useful for toolkits where arbitrary solx-specific options are not supported.

When this environment variable is set, **solx** will output all intermediate representations to the specified directory, always overwriting existing files.

The intermediate build artifacts include:

| Name          | Extension   |
|:--------------|:------------|
| LLVM IR       | *ll*        |
| LLVM Assembly | *asm*       |

Usage:

```console
$ SOLX_OUTPUT_DIR='./debug/' solx 'Simple.sol' --bin
...
$ ls './debug/'
Simple_sol_Simple.asm
Simple_sol_Simple.optimized.ll
Simple_sol_Simple.unoptimized.ll
Simple_sol_Simple_deployed.asm
Simple_sol_Simple_deployed.optimized.ll
Simple_sol_Simple_deployed.unoptimized.ll

```

The output file name is constructed as follows: `<ContractPath>_<ContractName>[_runtime].<Modifiers>.<Extension>`.

Additionally, it is possible to dump the standard JSON input file with the `SOLX_STANDARD_JSON_DEBUG` environment variable:

```bash
SOLX_STANDARD_JSON_DEBUG='./debug/input.json' solx 'Simple.sol' --bin
cat './debug/input.json' | jq .
```



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

