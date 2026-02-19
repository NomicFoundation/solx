# Installation

You can start using **solx** in the following ways:

1. Use the installation script.
   ```shell
   curl -L https://raw.githubusercontent.com/NomicFoundation/solx/main/install-solx | bash
   ```

   The script will download the latest stable release of **solx** and install it in your `PATH`.
   > ⚠️ The script requires `curl` to be installed on your system.<br>
   > This is the recommended way to install **solx** for MacOS users to bypass gatekeeper checks.

2. Download [stable releases](https://github.com/NomicFoundation/solx/releases). See [Static Executables](#static-executables).
3. Build **solx** from sources. See [Building from Source](#building-from-source).



## System Requirements

It is recommended to have at least 4 GB of RAM to compile large projects. The compilation process is parallelized by default, so the number of threads used is
equal to the number of CPU cores.

> Large projects can consume a lot of RAM during compilation on machines with a high number of cores.
> If you encounter memory issues, consider reducing the number of threads using the `--threads` option.

The table below outlines the supported platforms and architectures:

| CPU/OS | MacOS | Linux | Windows |
|:------:|:-----:|:-----:|:-------:|
| x86_64 |   ✅   |   ✅   |    ✅    |
| arm64  |   ✅   |   ✅   |    ❌    |

> Please avoid using outdated distributions of operating systems, as they may lack the necessary dependencies or include outdated versions of them.
> **solx** is only tested on recent versions of popular distributions, such as MacOS 11.0 and Windows 10.



## Versioning

The **solx** version consists of two parts:

1. **solx** version itself.
2. Version of **solc** libraries **solx** is statically linked with.

> We recommend always using the latest version of **solx** to benefit from the latest features and bug fixes.



## Ethereum Development Toolkits

For large codebases, it is more convenient to use **solx** via toolkits such as Hardhat.
These tools manage compiler input and output on a higher level, and provide additional features like incremental compilation and caching.



## Static Executables

We ship **solx** binaries on the [releases page of the eponymous repository](https://github.com/NomicFoundation/solx/releases). 
This repository maintains intuitive and stable naming for the executables and provides a changelog for each release. Tools using **solx** must download the binaries from this repository and cache them locally.

> All executables are statically linked and must work on all recent platforms without issues.



## Building from Source

> Please consider using the pre-built executables before building from source.
> Building from source is only necessary for development, research, and debugging purposes.
> Deployment and production use cases should rely only on [the officially released executables](#static-executables).

1. Install the necessary system-wide dependencies.

   * For Linux (Debian):

    ```shell
    apt install cmake ninja-build curl git libssl-dev pkg-config clang lld
    ```

   * For Linux (Arch):

    ```shell
    pacman -Syu which cmake ninja curl git pkg-config clang lld
    ```

   * For MacOS:

     1. Install the **Homebrew** package manager by following the instructions at [brew.sh](https://brew.sh).
     2. Install the necessary system-wide dependencies:

        ```shell
        brew install cmake ninja coreutils
        ```

     3. Install a recent build of the LLVM/[Clang](https://clang.llvm.org) compiler using one of the following tools:
        * [Xcode](https://developer.apple.com/xcode/)
        * [Apple’s Command Line Tools](https://developer.apple.com/library/archive/technotes/tn2339/_index.html)
        * Your preferred package manager.

2. Install Rust.

   The easiest way to do it is following the latest [official instructions](https://www.rust-lang.org/tools/install).

> The Rust version used for building is pinned in the [rust-toolchain.toml](../rust-toolchain.toml) file at the repository root.
> **cargo** will automatically download the pinned version of *rustc* when you start building the project.

3. Clone and checkout this repository with submodules.

   ```shell
   git clone https://github.com/NomicFoundation/solx --recursive
   ```

   By default, submodules checkout is disabled to prevent cloning large repositories via `cargo`.
   If you're building locally, ensure all submodules are checked out with:
   ```shell
   git submodule update --recursive --checkout
   ```
    
4. Build the development tools.

    ```shell
    cargo build --release --bin solx-dev
    ```

5. Build the LLVM framework using **solx-dev**.

   ```shell
   ./target/release/solx-dev llvm build --enable-mlir
   ```

   This builds LLVM with the EVM target, MLIR, and LLD projects enabled. The build artifacts will be placed in `target-llvm/`.

   For more information and available build options, run `./target/release/solx-dev llvm build --help`.

6. Build the **solc** libraries using **solx-dev**.

   ```shell
   ./target/release/solx-dev solc build
   ```

   This will configure and build the solc libraries in `solx-solidity/build/`. The command automatically detects MLIR and LLD paths if LLVM was built with those projects.

   For more options, run `./target/release/solx-dev solc build --help`.

7. Build the **solx** executable.

    ```shell
    cargo build --release
    ```
   
    The **solx** executable will appear as `./target/release/solx`, where you can run it directly or move it to another location.

    If **cargo** cannot find the LLVM build artifacts, ensure that the `LLVM_SYS_211_PREFIX` environment variable is not set in your system, as it may be pointing to a location different from the one expected by **solx**.



## Tuning the LLVM build

* For more information and available build options, run `./target/release/solx-dev llvm build --help`.
* The `--enable-mlir` flag enables MLIR support in the LLVM build (required for MLIR-based optimizations). LLD is always built.
* Use the `--ccache-variant ccache` option to speed up the build process if you have [ccache](https://ccache.dev) installed.

### Building LLVM manually

If you prefer building [the LLVM framework](https://github.com/matter-labs/solx-llvm) manually, include the following flags in your CMake command:

```shell
# We recommend using the latest version of CMake.

-DLLVM_TARGETS_TO_BUILD='EVM'
-DLLVM_ENABLE_PROJECTS='lld;mlir'
-DLLVM_ENABLE_RTTI='On'
-DBUILD_SHARED_LIBS='Off'
```

> For most users, **solx-dev** is the recommended way to build the framework.
> This section was added for compiler toolchain developers and researchers with specific requirements and experience with the LLVM framework.
