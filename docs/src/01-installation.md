# Installation

You can start using **solx** in the following ways:

1. Use the installation script.
   ```shell
   curl -L https://raw.githubusercontent.com/matter-labs/solx/main/install-solx | bash
   ```

   The script will download the latest stable release of **solx** and install it in your `PATH`.
   > ⚠️ The script requires `curl` to be installed on your system.<br>
   > This is the recommended way to install **solx** for MacOS users to bypass gatekeeper checks.

2. Download [stable releases](https://github.com/matter-labs/solx/releases). See [Static Executables](#static-executables).
3. Build **solx** from sources. See [Building from Source](#building-from-source).



## Usage

We recommend using **solx** via [Foundry](https://github.com/foundry-rs/foundry). It behaves in the same way as
**solc** v0.8.31, so you can install **solx** executable as described in the section above and specify:

```toml
[profile.solx]
solc_version = "/path/to/solx"
```

As a shortcut, you may also use `forge --use /path/to/solx` instead.

**solx** works with **Hardhat** as well, but requires additional configuration.



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

The **solx** version consists of three parts:

1. **solx** version itself.
2. Version of **solc** libraries **solx** is statically linked with.
3. Revision of the LLVM-friendly fork of **solc** maintained by the **solx** team.

For instance, the latest revision of the latest version of **solc** is `0.8.31-1.0.2`. Here are the **solc** revisions released by now:

| Revision |                         Fixes                        |
|:---------|:-----------------------------------------------------|
| *v1.0.0* | Compatibility between EVM assembly and LLVM IR       |
| *v1.0.1* | A compiler crash with nested try-catch patterns      |
| *v1.0.2* | Metadata of recursive calls across inheritance       |

> We recommend always using the latest version of **solx** to benefit from the latest features and bug fixes.



## Ethereum Development Toolkits

For large codebases, it is more convenient to use **solx** via toolkits such as Foundry and Hardhat.
These tools manage compiler input and output on a higher level, and provide additional features like incremental compilation and caching.



## Static Executables

We ship **solx** binaries on the [releases page of the eponymous repository](https://github.com/matter-labs/solx/releases). 
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
   git clone https://github.com/matter-labs/solx --recursive
   ```

   By default, submodules checkout is disabled to prevent cloning large repositories via `cargo`.
   If you're building locally, ensure all submodules are checked out with:
   ```shell
   git submodule update --recursive --checkout
   ```
    
4. Install the Matter Labs LLVM framework builder. This tool clones the [repository of Matter Labs LLVM Framework](https://github.com/matter-labs/era-compiler-llvm) and runs a sequence of build commands tuned for the needs of **solx**.

    ```shell
    cargo install compiler-llvm-builder
    ```

    To fine-tune your build of Matter Labs LLVM framework, refer to the section on [tuning the Matter Labs LLVM build](#tuning-the-llvm-build).

> Always use the latest version of the builder to benefit from the latest features and bug fixes.
> To check for new versions and update the builder, simply run `cargo install compiler-llvm-builder` again, even if you have already installed the builder.
> The builder is not the LLVM framework itself, but only a tool to build it.
> By default, it is installed in `~/.cargo/bin/`, which is usually added to your `PATH` during the Rust installation process.

5. Build the LLVM framework using the `zksync-llvm` tool.
  
   ```shell
   # Navigate to the root of your local copy of this repository.
   cd solx
   # Build the LLVM framework.
   zksync-llvm build
   ```
  
   For more information and available build options, run `zksync-llvm build --help`.

6. Build the **solc** libraries.

   ```shell
   cd solx-solidity
   mkdir -pv build
   cd build
   cmake .. \
      -DPEDANTIC='OFF' \
      -DTESTS='OFF' \
      -DCMAKE_BUILD_TYPE='Release' \
      -DSOL_VERSION_ZKSYNC='0.8.31-1.0.2'
   cmake --build . --config Release --parallel ${YOUR_CPU_COUNT}
   cd ../..
   ```

   The sequence above may fail with clang v20 and/or Boost 1.88.
   We are currently looking for a solution and will document it when we find one.
   If you encounter compilation errors, try specifying the aforementioned versions to replace your system default:

   ```shell
   # MacOS example

   # Install Boost v1.85
   brew install boost@1.85

   # Make Boost v1.85 default
   brew unlink boost
   brew link boost@1.85
   
   # Install LLVM with clang v19
   brew install llvm@19

   # Re-run the build command
   cmake .. \
      -DCMAKE_C_COMPILER=/opt/homebrew/opt/llvm@19/bin/clang \
      -DCMAKE_CXX_COMPILER=/opt/homebrew/opt/llvm@19/bin/clang++ \
      -DPEDANTIC=OFF \
      -DTESTS=OFF \
      -DCMAKE_BUILD_TYPE="Release" \
      -DSOL_VERSION_ZKSYNC="0.8.31-1.0.2"
   cmake --build . --config Release --parallel ${YOUR_CPU_COUNT}
   ```

   The `-DSOL_VERSION_ZKSYNC` flag is used to specify the version-revision of the **solc** that is reported by **solx**.
   By default, we recommend keeping the revision at `1.0.2` to follow our [versioning](#versioning).
   Otherwise, feel free to change it according to your needs.

7. Build the **solx** executable.

    ```shell
    cargo build --release
    ```
   
    The **solx** executable will appear as `./target/release/solx`, where you can run it directly or move it to another location.

    If **cargo** cannot find the LLVM build artifacts, ensure that the `LLVM_SYS_191_PREFIX` environment variable is not set in your system, as it may be pointing to a location different from the one expected by **solx**.



## Tuning the LLVM build

* For more information and available build options, run `zksync-llvm build --help`.
* Use the `--use-ccache` option to speed up the build process if you have [ccache](https://ccache.dev) installed.
* To build the Matter Labs LLVM framework using specific C and C++ compilers, pass additional arguments to [CMake](https://cmake.org/) using the `--extra-args` option:

  ```shell
  # Pay special attention to character escaping.

  zksync-llvm build \
    --use-ccache \
    --extra-args \
      '\-DCMAKE_C_COMPILER=/opt/homebrew/Cellar/llvm@18/18.1.8/bin/clang' \
      '\-DCMAKE_BUILD_TYPE=Release' \
      '\-DCMAKE_CXX_COMPILER=/opt/homebrew/Cellar/llvm@18/18.1.8/bin/clang++' 
  ```

### Building LLVM manually

* If you prefer building [your LLVM framework](https://github.com/matter-labs/era-compiler-llvm) manually, include the following flags in your CMake command:

  ```shell
  # We recommended using the latest version of CMake.

  -DLLVM_TARGETS_TO_BUILD='EVM'
  -DLLVM_ENABLE_PROJECTS='lld'
  -DBUILD_SHARED_LIBS='Off'
  ```

> For most users, the [Matter Labs LLVM builder](#building-from-source) is the recommended way to build the framework.
> This section was added for compiler toolchain developers and researchers with specific requirements and experience with the LLVM framework.
> We are going to present a more detailed guide for LLVM contributors in the future.
