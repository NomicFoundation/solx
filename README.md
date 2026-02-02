<div align="center">
  <img src=".github/assets/logo.png" alt="solx logo" />
</div>

# Optimizing Solidity Compiler

**solx** is a new optimizing compiler for EVM developed by [Matter Labs](https://matter-labs.io/) and [Nomic Foundation](https://nomic.foundation/).

> [!WARNING]  
> The project is in beta and must be used with caution. Please use it only for testing and experimentation.
> If you want to use it in production, make sure to test your contracts thoroughly, or [contact us](#contact-us) first.

**solx** passes multiple test suites, including:

- [Foundry projects](./solx-dev/foundry-tests.toml)
- [Hardhat projects](./solx-dev/hardhat-tests.toml)
- [DeFi protocols](./tests/solidity/complex/defi): UniswapV2, UniswapV3, Mooniswap, StarkEx
- [Semantic tests](https://github.com/NomicFoundation/solx-solidity/tree/0.8.33/test/libsolidity/semanticTests) from the **solc** repository
- [Additional tests](./tests/solidity) written by the **solx** team

## Documentation

**solx** documentation is powered by [GitHub Pages](https://nomicfoundation.github.io/solx/latest/) and provided as an [mdBook](https://github.com/rust-lang/mdBook), while its Markdown sources can be found in [this directory](./docs/src/).
To build the book, follow these [instructions](./docs/README.md).

See also:

- [Solidity Documentation](https://docs.soliditylang.org/en/v0.8.33/)
- [LLVM Documentation](https://llvm.org/docs/)

## Installation

For the detailed installation and usage guide, visit [the respective page of our documentation](https://nomicfoundation.github.io/solx/latest/#installation).

## Quick Start

```shell
# Compile a Solidity file
solx Contract.sol --bin --abi

# Compile with optimizations
solx Contract.sol --bin --abi -O3

# Output to a directory
solx Contract.sol --bin --abi -o ./build
```

## Architecture

For details on the compilation pipeline and components, see [the architecture documentation](https://nomicfoundation.github.io/solx/latest/04-architecture.html).

## Testing

For details on running unit tests, integration tests, and project tests, see [the testing documentation](https://nomicfoundation.github.io/solx/latest/05-testing.html).

## Troubleshooting

If you are building **solx** from source and you have multiple LLVM builds in your system, ensure that you choose the correct one to build the compiler.
The environment variable `LLVM_SYS_211_PREFIX` sets the path to the directory with LLVM build artifacts, which typically ends with `target-llvm/build-final`.
For example:

```shell
export LLVM_SYS_211_PREFIX="${HOME}/src/solx/target-llvm/build-final"
```

If you suspect that the compiler is not using the correct LLVM build, check by running `set | grep LLVM`, and reset all LLVM-related environment variables.

For reference, see [llvm-sys](https://crates.io/crates/llvm-sys) and [Local LLVM Configuration Guide](https://llvm.org/docs/GettingStarted.html#local-llvm-configuration).

## License

- Crate **solx** is licensed under [GNU General Public License v3.0](./solx/LICENSE.txt)
- All other crates are licensed under the terms of either
  - Apache License, Version 2.0 ([LICENSE-APACHE](./solx-standard-json/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
  - MIT license ([LICENSE-MIT](./solx-standard-json/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- [`solx-solidity`](https://github.com/NomicFoundation/solx-solidity/) is licensed under [GNU General Public License v3.0](https://github.com/NomicFoundation/solx-solidity/blob/0.8.33/LICENSE.txt)
- [`solx-llvm`](https://github.com/matter-labs/solx-llvm) is licensed under the terms of Apache License, Version 2.0 with LLVM Exceptions, ([LICENSE](https://github.com/matter-labs/solx-llvm/blob/main/LICENSE) or https://llvm.org/LICENSE.txt)

Additionally, this repository vendors tests and test projects that preserve their original licenses:

- [UniswapV2](./tests/solidity/complex/defi/UniswapV2)
- [UniswapV3](./tests/solidity/complex/defi/UniswapV3)
- [Mooniswap](./tests/solidity/complex/defi/Mooniswap)
- [StarkEx Verifier](./tests/solidity/complex/defi/starkex-verifier)

These projects are modified for the purposes of testing our compiler toolchain and are not used outside of this repository.

Visit the project directories to discover the terms of each license in detail. These and other projects are licensed in either per-file or per-project manner.

## Contact Us

Email us at [solx@matterlabs.dev](mailto:solx@matterlabs.dev) or join our [Telegram group](https://t.me/solx_devs).
