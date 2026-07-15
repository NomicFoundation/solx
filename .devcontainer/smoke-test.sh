#!/usr/bin/env bash
# Fast devcontainer health checks covering its common failure paths, without
# building LLVM (bootstrap.sh stays a deliberate step — see
# docs/src/developer-guide/00-development-container.md). CI runs this via the
# validate-devcontainer job; it is also safe to run manually inside the
# container if the setup feels off.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Running as a non-root user with passwordless sudo"
test "$(id -u)" -ne 0
sudo -n true

echo "==> Workspace and named volumes are writable"
for dir in . target target-slang target-llvm "${CCACHE_DIR:?CCACHE_DIR not set}"; do
    touch "${dir}/.devcontainer-smoke"
    rm "${dir}/.devcontainer-smoke"
done

echo "==> System toolchain on PATH"
for tool in cmake ninja ccache clang lld git curl; do
    command -v "${tool}"
done

echo "==> Devcontainer scripts are executable"
test -x .devcontainer/bootstrap.sh
test -x .devcontainer/post-create.sh

echo "==> Pinned Rust toolchain resolves (downloads on first run)"
cargo --version
grep -qF "\"$(rustc --version | awk '{print $2}')\"" rust-toolchain.toml

echo "==> Devcontainer smoke test passed"
