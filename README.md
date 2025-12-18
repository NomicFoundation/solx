<div align="center">
  <img src=".github/assets/logo.png" alt="solx logo" />
</div>

# Optimizing Solidity Compiler

**solx** is a new optimizing compiler for EVM developed by [Matter Labs](https://matter-labs.io/).

> [!WARNING]  
> The project is in beta state and must be used with caution. Please use it only for testing and experimentation.
> If you want to use it in production, make sure to test your contracts thoroughly, or [contact us](#contact-us) first.

**solx** passes multiple test suites, including:

- [Foundry projects](.github/forge-benchmarks.toml)
- [Hardhat projects](.github/hardhat-projects.toml)
- [tests](https://github.com/matter-labs/solx-solidity/tree/0.8.31/test/libsolidity/semanticTests) from the **solc** repository
- [real-life projects](solx-tests/solidity/complex/defi) such as UniswapV2 and Mooniswap
- [additional tests](solx-tests/solidity) written by the **solx** team

## Documentation

**solx** documentation is powered by [GitHub Pages](https://nomicfoundation.github.io/solx/latest/) and provided as an [mdBook](https://github.com/rust-lang/mdBook), while its Markdown sources can be found in [this directory](./docs/src/).
To build the book, follow these [instructions](./docs/README.md).

See also:

- [Solidity Documentation](https://docs.soliditylang.org/en/latest/)
- [LLVM Documentation](https://llvm.org/docs/)

## Installation

For the detailed installation and usage guide, visit [the respective page of our documentation](https://nomicfoundation.github.io/solx/latest/#installation).

## Architecture

**solx** consists of three main parts:

1. **solx** executable from this repository. The repository also contains parts of the compiler front end: Yul and EVM assembly translators.
2. [solx-solidity](https://github.com/matter-labs/solx-solidity/), an LLVM-friendly fork of [the Solidity compiler](https://github.com/ethereum/solidity),
  that emits Yul and EVM assembly for **solx**. Despite the repository name, it is not directly related to either ZKsync or ZKsync Era.
3. [era-compiler-llvm](https://github.com/matter-labs/era-compiler-llvm), a fork of [the LLVM project](https://github.com/llvm/llvm-project)
  with an EVM target developed by the **solx** team.

The most important part of the project is the EVM target in LLVM. You can find its sources [here](https://github.com/matter-labs/era-compiler-llvm/tree/main/llvm/lib/Target/EVM).

## Testing

To run the unit and CLI tests, execute `cargo test` at the repository root.

## Troubleshooting

If you have multiple LLVM builds in your system, ensure that you choose the correct one to build the compiler.
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
- [`solx-solidity`](https://github.com/matter-labs/solx-solidity/) is licensed under [GNU General Public License v3.0](https://github.com/matter-labs/solx-solidity/blob/0.8.31/LICENSE.txt)
- [`era-compiler-llvm`](https://github.com/matter-labs/era-compiler-llvm) is licensed under the terms of Apache License, Version 2.0 with LLVM Exceptions, ([LICENSE](https://github.com/matter-labs/era-compiler-llvm/blob/main/LICENSE) or https://llvm.org/LICENSE.txt)

Additionally, this repository vendors tests and test projects that preserve their original licenses:

- [UniswapV2](./tests/solidity/complex/defi/UniswapV2)
- [UniswapV3](./tests/solidity/complex/defi/UniswapV3)
- [Mooniswap](./tests/solidity/complex/defi/Mooniswap)
- [StarkEx Verifier](./tests/solidity/complex/defi/starkex-verifier)
- [SHIT](./tests/solidity/complex/defi/shitdao)

These projects are modified for the purposes of testing our compiler toolchain and are not used outside of this repository.

Visit the project directories to discover the terms of each license in detail. These and other projects are licensed in either per-file or per-project manner.

## Contact Us

Email us at [solx@matterlabs.dev](mailto:solx@matterlabs.dev) or join our [Telegram group](https://t.me/solx_devs).
