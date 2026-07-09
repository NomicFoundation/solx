# Development Container

The repository ships a [devcontainer](https://containers.dev/) at `.devcontainer/` that reproduces the CI build environment locally: it layers a non-root user and a shared Rust toolchain on top of `ghcr.io/nomicfoundation/solx-ci-runner`, the same image every CI job runs in. Anything that builds in the container builds in CI, and vice versa.

It works with VS Code (**Dev Containers: Reopen in Container**), the [devcontainer CLI](https://github.com/devcontainers/cli), and GitHub Codespaces.

## Host requirements

A cold LLVM build is the dominant cost, and it is resource-hungry:

- **CPU:** 8+ cores recommended. The cold build takes roughly an hour on 8 cores.
- **Memory:** 16 GB. Each LLVM tool links at 2–4 GB RSS; the container caps parallel link jobs at 2. Docker Desktop's default 8 GB VM will thrash — raise it in Docker Desktop settings.
- **Disk:** 64 GB free for Docker. The LLVM build tree alone takes tens of GB (all of it lives in named volumes, not in your checkout).

## What the image contains — and what it deliberately does not

The `solx-ci-runner` image provides the *system* toolchain: cmake, ninja, clang/LLD 21, ccache, rustup, and Node.js. Two things are **not** in the image and are built from source inside the container:

1. **The custom LLVM framework with the EVM backend** (the `solx-llvm` submodule). It is not baked into the image because it changes with every submodule bump and weighs several GB — a baked-in copy would bloat every CI image pull and be stale the moment the submodule moves. CI builds it per-job behind an Actions cache keyed on the submodule commit; the devcontainer builds it once locally and keeps it in a named volume, with ccache absorbing most of the cost of rebuilds.
2. **The solc fork libraries** (the `solx-solidity` submodule), which **solx** links statically.

The Rust toolchain itself is also resolved lazily: rustup downloads the version pinned in `rust-toolchain.toml` on the first `cargo` invocation, so a toolchain bump never requires an image rebuild.

## Getting started

The `solx-ci-runner` package is **private**, so pulling it requires a GitHub token with the `read:packages` scope — an unauthenticated first open fails the container build with `unauthorized`. The login does not need to persist: the image is pinned by digest, so once it is in the local Docker cache every (re)build reuses it without contacting GHCR. Pull it once and log straight back out:

```shell
gh auth token | docker login ghcr.io --username <your-github-username> --password-stdin
grep -oP 'ghcr\.io\S+' .devcontainer/Dockerfile | xargs docker pull
docker logout ghcr.io
```

(If your `gh` token lacks the scope, `gh auth refresh --scopes read:packages` adds it, or use any PAT with `read:packages`. Docker Desktop stores logins in the OS keychain rather than in `~/.docker/config.json`, so keeping the login is also fine there.)

The pull only needs repeating when the pinned digest changes. Then:

1. Clone the repository (submodules can be left uninitialized; the bootstrap handles them) and open it in VS Code.
2. Run **Dev Containers: Reopen in Container**. The first open builds the thin local image layer and then prints the next step.
3. Build the toolchain:

   ```shell
   .devcontainer/bootstrap.sh
   ```

   This is the ~1 hour step (cold). It is kept out of the automatic container setup precisely because of that cost — you should know when you are paying it.

4. Build and test **solx**:

   ```shell
   cargo build --release
   cargo test
   ```

## How LLVM is installed

`bootstrap.sh` is a thin wrapper over the same `solx-dev` builder CI uses. Step by step:

1. `git submodule update --init --recursive --depth 1` — fetches `solx-llvm` and `solx-solidity` shallowly, exactly as CI does. If you need full history in a submodule (e.g. to bisect), run `git fetch --unshallow` inside it.
2. `cargo build --release --bin solx-dev` — builds the builder. This first `cargo` call also downloads the pinned Rust toolchain.
3. `./target/release/solx-dev llvm build --enable-assertions --enable-mlir --ccache-variant ccache` — configures and builds LLVM:
   - The build tree lives in `target-llvm/build-final/`, the installation in `target-llvm/target-final/`.
   - `.cargo/config.toml` already points `LLVM_SYS_211_PREFIX`, `MLIR_SYS_210_PREFIX`, and `TABLEGEN_210_PREFIX` at that installation, so no environment setup is needed — `cargo` and rust-analyzer find it as soon as it exists.
   - MLIR is enabled by default because the Slang frontend pipeline (`cargo test-slang`) requires it. Pass `--no-mlir` to skip it and shave build time if you only work on the solc/Yul pipeline.
   - Assertions are enabled, matching CI.
4. `./target/release/solx-dev solc build --build-boost --ccache-variant ccache --enable-mlir` — builds the solc fork libraries into `solx-solidity/build/` (again already wired up via `SOLC_PREFIX`/`BOOST_PREFIX`). `--build-boost` downloads and builds a static Boost into `solx-solidity/boost/` first — the runner image deliberately ships no system Boost, matching how CI builds solc.

Until step 3 has completed, `cargo check`/rust-analyzer fail in `llvm-sys`'s build script with a missing `llvm-config` — that is the expected symptom of "LLVM not built yet", not a broken container.

### Troubleshooting

First stop: `.devcontainer/smoke-test.sh` checks the container's basic health (non-root user, writable volumes, toolchain on PATH, Rust pin resolution) in under a minute — CI runs the same script to validate devcontainer changes. Known failure modes:

- **`error: Missing manifest in toolchain '1.96.0-…'`** — an interrupted first `cargo` run (e.g. Ctrl+C during the toolchain download) leaves a half-extracted toolchain, and it persists in the `solx-rustup` volume across container rebuilds. Fix: `rustup toolchain uninstall <the toolchain from the error>`, then rerun; the pinned toolchain reinstalls automatically.
- **`llvm-sys` fails with a missing `llvm-config`** — LLVM has not been built yet (or the bootstrap was interrupted before finishing). Rerun `.devcontainer/bootstrap.sh`; ccache makes the retry cheap.
- **Files vanish or builds break mid-bootstrap in confusing ways** — the workspace is a bind mount: the container and your host checkout are the same files. Switching branches, `git clean`, or anything else that rewrites the tree on the host while a bootstrap or build runs in the container will break it in nonsensical-looking ways (and a host-side switch to a branch without `.devcontainer/` may prompt VS Code to recreate the container mid-run). Leave the checkout alone until the build finishes; the bootstrap is idempotent and resumes cheaply after a rerun.

### Rebuilding after a submodule bump

Rerun `.devcontainer/bootstrap.sh` (or the `solx-dev llvm build` invocation directly). The ccache volume persists across container rebuilds, so a rebuild after a typical `solx-llvm` bump takes minutes, not an hour. Add `--clean` to `solx-dev llvm build` if a stale build tree misbehaves.

For sanitizer or coverage LLVM builds, see [Building with Sanitizers](./04-sanitizers.md) — the same flags work inside the container.

## Persistence

Build state lives in Docker named volumes so it survives **Rebuild Container** and stays off the (slow on macOS/Windows) bind mount:

| Volume | Mount point | Holds |
|---|---|---|
| `solx-target` | `target/` | solx build artifacts, `solx-dev` |
| `solx-target-slang` | `target-slang/` | `cargo *-slang` alias artifacts |
| `solx-target-llvm` | `target-llvm/` | LLVM build tree + installation |
| `solx-rustup` | `/usr/local/rustup` | downloaded Rust toolchains |
| `solx-cargo` | `/usr/local/cargo` | cargo registry/git caches |
| `solx-ccache` | `/var/cache/solx-ccache` | LLVM compiler cache |

To start truly fresh, remove the volumes (`docker volume rm solx-target-llvm …`) in addition to rebuilding the container.

## Working on the LLVM fork itself

The devcontainer is also the intended environment for hacking on `solx-llvm`: the fork is not built standalone — `solx-dev` owns the CMake configuration (in `solx-dev/src/llvm/`), and `solx-llvm`'s own regression CI drives its builds through a **solx** checkout in the same runner image.

1. Point the submodule at your branch: `git -C solx-llvm checkout <branch>` (after `git -C solx-llvm fetch --unshallow origin <branch>` if needed).
2. Rebuild: `./target/release/solx-dev llvm build --enable-assertions --enable-mlir --ccache-variant ccache`.
3. C++ language support: `solx-dev` exports `compile_commands.json` into `target-llvm/build-final/`, and the devcontainer configures clangd to read it, so cross-references in the submodule work after the first build.

## Notes for Slang contributors

Coming from `slang`, the moving parts map as follows: there is no Hermit — the system toolchain comes from the CI image and the Rust toolchain from `rust-toolchain.toml`; the `infra` CLI's role is split between `.devcontainer/bootstrap.sh` (one-time environment setup) and `solx-dev` (LLVM/solc builds, integration test suites). The Slang-frontend crates (`solx-slang`, `solx-mlir`) are built and tested via the `cargo build-slang` / `cargo test-slang` aliases, which need the MLIR-enabled LLVM build that `bootstrap.sh` produces by default.
