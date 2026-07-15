#!/usr/bin/env bash
# Container plumbing that must run once per container creation. The heavy
# toolchain build (LLVM, solc) is deliberately NOT run here — it takes about
# an hour cold, so it stays an explicit step: bootstrap.sh.
set -euo pipefail

# Named volumes are created root-owned; hand them to the dev user.
sudo chown "$(id -u):$(id -g)" \
    target target-slang target-llvm \
    /var/cache/solx-ccache /usr/local/rustup /usr/local/cargo

# Docker Desktop file sharing can surface the bind mount with foreign
# ownership, which git rejects as "dubious ownership" for the repo and every
# submodule. Scoped to this container's git config only.
git config --global --add safe.directory '*'

cat <<'EOF'

solx devcontainer is ready. To build the toolchain (custom LLVM + solc fork):

    .devcontainer/bootstrap.sh

First run takes ~1h (LLVM from source); reruns are much faster via ccache.
Guide: docs/src/developer-guide/00-development-container.md
EOF
