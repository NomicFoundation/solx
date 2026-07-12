#!/usr/bin/env bash
# Builds the full solx toolchain inside the devcontainer: submodules, the
# solx-dev builder, the custom LLVM backend, and the solc fork libraries.
# Idempotent — rerun it after a solx-llvm submodule bump; ccache (persisted in
# a named volume) makes rebuilds much cheaper than the cold build.
set -euo pipefail
cd "$(dirname "$0")/.."

# MLIR is on by default: it is required for the Slang frontend pipeline
# (cargo test-slang) and CI caches it as a separate artifact variant anyway.
MLIR_FLAG="--enable-mlir"
CLEAN_FLAG=""
for arg in "$@"; do
    case "${arg}" in
        --no-mlir) MLIR_FLAG="" ;;
        # Wipes target-llvm before building. Needed when the persistent build
        # tree was configured in a previous container image (see the guide's
        # troubleshooting section).
        --clean) CLEAN_FLAG="--clean" ;;
        *) echo "unknown option: ${arg} (supported: --no-mlir, --clean)" >&2; exit 1 ;;
    esac
done

# ccache's stock 5G cap evicts mid-build on LLVM; the setting persists in the
# volume-backed CCACHE_DIR.
ccache --set-config=max_size=20G

echo "==> Initializing submodules (shallow, as in CI)"
# A submodule left on a branch is the guide's fork-hacking flow; updating it
# would silently detach it back to the recorded SHA and rebuild upstream.
while read -r submodule; do
    if [ -e "${submodule}/.git" ] \
        && branch=$(git -C "${submodule}" symbolic-ref --short -q HEAD); then
        echo "==> ${submodule} is on branch '${branch}' — leaving it untouched"
        continue
    fi
    git submodule update --init --recursive --depth 1 "${submodule}"
done < <(git config --file .gitmodules --get-regexp '^submodule\..*\.path$' | awk '{print $2}')

# Boost artifacts land inside the solx-solidity submodule and are not in the
# fork's .gitignore; without this local exclude, git status enumerates the
# extracted Boost tree (tens of thousands of files) and VS Code's git
# extension drowns in it ("too many changes").
# TODO: delete once the submodule pin includes the fork's boost .gitignore
# entries (cherry-pick of NomicFoundation/solx-solidity#111 onto 0.8.34).
EXCLUDE_FILE=$(git -C solx-solidity rev-parse --git-path info/exclude)
grep -qxF '/boost*' "${EXCLUDE_FILE}" 2>/dev/null || echo '/boost*' >> "${EXCLUDE_FILE}"

echo "==> Building solx-dev"
cargo build --release --bin solx-dev

echo "==> Building LLVM ${MLIR_FLAG:+(with MLIR) }— ~1h cold, minutes on warm ccache"
# LLVM_PARALLEL_LINK_JOBS=2 caps peak memory: each tool links at 2-4 GB RSS.
./target/release/solx-dev llvm build \
    --enable-assertions \
    --ccache-variant ccache \
    ${MLIR_FLAG} ${CLEAN_FLAG} \
    --extra-args "-DLLVM_PARALLEL_LINK_JOBS='2'"

echo "==> Building solc libraries (downloads and builds a static Boost first)"
# --build-boost matches CI: the runner image ships no system Boost.
# MLIR must match the LLVM build, as in slang-tests.yaml.
./target/release/solx-dev solc build \
    --build-boost \
    --ccache-variant ccache \
    ${MLIR_FLAG}

cat <<'EOF'
==> Toolchain ready. Next steps:

    cargo build --release    # build solx
    cargo test               # unit + CLI tests
    cargo test-slang         # Slang/MLIR frontend tests (needs MLIR-enabled LLVM)
EOF
