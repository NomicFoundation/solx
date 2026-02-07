# Building with Sanitizers

This is the guide on building **solx** with sanitizers enabled.

## Introduction

Sanitizers are tools that help find bugs in code. They are used to detect memory corruption, leaks, and undefined behavior.
The most common sanitizers are `AddressSanitizer`, `MemorySanitizer`, and `ThreadSanitizer`.

If you are not familiar with sanitizers, see the [official documentation](https://rustc-dev-guide.rust-lang.org/sanitizers.html).

### Who is this guide for?

This guide is for developers who want to debug issues with **solx**.

### Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install)
- [Git](https://git-scm.com/downloads)
- [LLVM compiler](https://llvm.org/docs/GettingStarted.html)

<div class="warning">
For sanitizers build to work, the host LLVM compiler version that is used to build LLVM <b>MUST</b>
have the same version as the LLVM compiler that is used internally by `rustc` to build **solx**.

You can check the LLVM version used by `rustc` by running the following command `rustc --version --verbose`.
</div>

## Build steps

The general steps to have a sanitizer enabled build include:
1. Build the LLVM framework with the required sanitizer enabled.
2. Build **solx** with the LLVM build from the previous step.

Please, follow the common [installation instructions](../user-guide/01-installation.md#building-from-source)
until the `solx-llvm build` step.

This guide assumes the build with `AddressSanitizer` enabled.

### Build LLVM with sanitizer enabled

When building LLVM, use `--sanitizer <sanitizer>` option and set build type to `RelWithDebInfo`:

```shell
solx-llvm build --sanitizer=Address --build-type=RelWithDebInfo
```

<div class="warning">
Please note that the default Apple Clang compiler is not compatible with Rust.
You need to install LLVM using Homebrew and specify the path to the LLVM compiler in the `--extra-args` option.
For example:
</div>

```shell
solx-llvm build --sanitizer=Address
```

### Build **solx** with the sanitizer enabled

To build **solx** with the sanitizer enabled, you need to set the `RUSTFLAGS` environment variable
to `-Z sanitizer=address` and run the `cargo build` command.
Sanitizers build is a feature that is available only for the nightly Rust compiler, it is recommended
to set `RUSTC_BOOTSTRAP=1` environment variable before the build.

It is also mandatory to use `--target` option to specify the target architecture. Otherwise, the build will fail.
Please, check the table below to find the correct target for your platform.

| Platform    | LLVM Target Triple            |
|-------------|-------------------------------|
| MacOS-arm64 | `aarch64-apple-darwin`        |
| MacOS-x86   | `x86_64-apple-darwin`         |
| Linux-arm64 | `aarch64-unknown-linux-gnu`   |
| Linux-x86   | `x86_64-unknown-linux-gnu`    |


Additionally, for proper reports symbolization it is recommended to set the `ASAN_SYMBOLIZER_PATH` environment variable.
For more info, see [symbolizing reports](https://clang.llvm.org/docs/AddressSanitizer.html#id4) section of LLVM documentation.

For example, to build **solx** for MacOS-arm64 with `AddressSanitizer` enabled, run the following command:
```shell
export RUSTC_BOOTSTRAP=1
export ASAN_SYMBOLIZER_PATH=$(which llvm-symbolizer) # check the path to llvm-symbolizer
TARGET=aarch64-apple-darwin # Change to your target
RUSTFLAGS="-Z sanitizer=address" cargo test --target=${TARGET}
```

Congratulations! You have successfully built **solx** with the sanitizers enabled.

Please, refer to the [official documentation](https://rustc-dev-guide.rust-lang.org/sanitizers.html)
for more information on how to use sanitizers and their types.
