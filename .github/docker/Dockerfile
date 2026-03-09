# CI runner image for solx — replaces the third-party Matter Labs image.
# Provides system toolchain (compiler, cmake, ninja, LLVM tools, Rust) needed
# to build the custom LLVM EVM backend and solx itself inside CI jobs.
#
# The custom LLVM 21.1 backend is NOT baked into this image — it is built from
# source in every CI job via the build-llvm composite action.

FROM ubuntu:24.04

LABEL org.opencontainers.image.source="https://github.com/NomicFoundation/solx"
LABEL org.opencontainers.image.description="CI runner image for solx"

ENV DEBIAN_FRONTEND=noninteractive

# ── System packages ──────────────────────────────────────────────────────────
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        cmake \
        ninja-build \
        ccache \
        git \
        curl \
        wget \
        pkg-config \
        python3 \
        jq \
        ca-certificates \
        gnupg \
        lsb-release \
        software-properties-common \
        libssl-dev \
        openssh-client \
        xz-utils \
        libc6-dbg \
    && rm -rf /var/lib/apt/lists/*

# ── LLVM 21 (system toolchain for coverage, sanitizers, lipo) ────────────────
# Must match the LLVM version shipped inside rustc (1.93.0 → LLVM 21) so that
# llvm-cov / llvm-profdata can read the profraw format emitted by instrumented
# Rust binaries.
RUN curl -fsSL https://apt.llvm.org/llvm-snapshot.gpg.key \
        | gpg --dearmor -o /usr/share/keyrings/llvm-archive-keyring.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/llvm-archive-keyring.gpg] \
        https://apt.llvm.org/noble/ llvm-toolchain-noble-21 main" \
        > /etc/apt/sources.list.d/llvm-21.list \
    && apt-get update && apt-get install -y --no-install-recommends \
        clang-21 \
        lld-21 \
        llvm-21 \
        libclang-rt-21-dev \
    && rm -rf /var/lib/apt/lists/*

# Symlink versioned LLVM binaries to unversioned names so workflows can call
# e.g. `clang`, `lld`, `llvm-cov` without the `-21` suffix.
RUN for bin in /usr/lib/llvm-21/bin/*; do \
        name=$(basename "${bin}"); \
        update-alternatives --install "/usr/bin/${name}" "${name}" "${bin}" 100; \
    done

# ── Replace glibc's libm.a linker script with a real merged archive ────────
# glibc ships libm.a as a GNU ld linker script that references libm-<ver>.a
# and libmvec.a.  LLVM's lld cannot parse this format, which breaks Rust
# builds that use -C target-feature=+crt-static -C link-arg=-fuse-ld=lld.
# Approach copied from matter-labs/compiler-infra.
ARG TARGETPLATFORM
RUN set -eux; \
    case "${TARGETPLATFORM}" in \
        linux/amd64) LIBDIR=/usr/lib/x86_64-linux-gnu ;; \
        linux/arm64) LIBDIR=/usr/lib/aarch64-linux-gnu ;; \
        *)           echo "unsupported platform: ${TARGETPLATFORM}"; exit 1 ;; \
    esac; \
    rm -f "${LIBDIR}/libm.a"; \
    LIBM_STATIC=$(find "${LIBDIR}" -maxdepth 1 -name 'libm-*.a' ! -name 'libmvec.a' | head -1); \
    [ -n "${LIBM_STATIC}" ] || { echo "error: no libm-*.a found in ${LIBDIR}"; exit 1; }; \
    ar -cqT "${LIBDIR}/libm.a" "${LIBM_STATIC}" "${LIBDIR}/libmvec.a"; \
    bash -c "ar -M <<< \$(echo -e \"create ${LIBDIR}/libm.a\naddlib ${LIBDIR}/libm.a\nsave\nend\")"

# ── Symlink libstdc++.a into system library path for rustc/lld ────────────
# GCC installs libstdc++.a inside its own versioned directory (e.g.
# /usr/lib/gcc/x86_64-linux-gnu/13/) which neither rustc nor lld search by
# default.  Coverage builds use -C target-feature=+crt-static which requires
# the static C++ standard library to be discoverable.
RUN set -eux; \
    case "${TARGETPLATFORM}" in \
        linux/amd64) TRIPLE=x86_64-linux-gnu ;; \
        linux/arm64) TRIPLE=aarch64-linux-gnu ;; \
        *)           echo "unsupported platform: ${TARGETPLATFORM}"; exit 1 ;; \
    esac; \
    GCC_VER=$(ls /usr/lib/gcc/${TRIPLE}/ | sort -V | tail -1); \
    ln -sf "/usr/lib/gcc/${TRIPLE}/${GCC_VER}/libstdc++.a" "/usr/lib/${TRIPLE}/libstdc++.a"

# ── Valgrind (for LLVM regression tests under valgrind) ───────────────────
ENV VALGRIND_VERSION=3.23.0
RUN curl -fsSL -o valgrind-${VALGRIND_VERSION}.tar.bz2 \
        https://sourceware.org/pub/valgrind/valgrind-${VALGRIND_VERSION}.tar.bz2 \
    && tar -xf valgrind-${VALGRIND_VERSION}.tar.bz2 \
    && cd valgrind-${VALGRIND_VERSION} && ./configure && make && make install \
    && cd .. && rm -rf valgrind-${VALGRIND_VERSION}.tar.bz2 valgrind-${VALGRIND_VERSION}

# ── Rust ─────────────────────────────────────────────────────────────────────
# Version must match rust-toolchain.toml in the solx repo.
ARG RUST_VERSION=1.93.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain "${RUST_VERSION}" --profile default \
    && . "$HOME/.cargo/env" \
    && rustup component add rust-src

ENV PATH="/root/.cargo/bin:/usr/lib/llvm-21/bin:${PATH}" \
    LD_LIBRARY_PATH="/usr/lib/llvm-21/lib"

# ── Cargo tools ────────────────────────────────────────────────────────────
# When updating versions here, also update the matching pins in:
#   .github/actions/cargo-check/action.yml
#   .github/actions/rust-unit-tests/action.yml
RUN cargo install \
    cargo2junit@0.1.15 \
    cargo-llvm-cov@0.8.4 \
    cargo-udeps@0.1.60 \
    --locked

# ── Node.js (for SFW and npm-based CI tools) ─────────────────────────────
ARG NODE_MAJOR=24
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_MAJOR}.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

# Smoke-test: ensure key tools are on PATH
RUN set -eux \
    && rustc --version \
    && cargo --version \
    && cmake --version \
    && ninja --version \
    && clang --version \
    && lld -flavor gnu --version \
    && command -v llvm-cov \
    && command -v llvm-profdata \
    && command -v llvm-lipo \
    && command -v llvm-symbolizer \
    && node --version \
    && npm --version \
    && valgrind --version \
    && case "${TARGETPLATFORM}" in \
        linux/amd64) test -f /usr/lib/x86_64-linux-gnu/libstdc++.a ;; \
        linux/arm64) test -f /usr/lib/aarch64-linux-gnu/libstdc++.a ;; \
       esac
