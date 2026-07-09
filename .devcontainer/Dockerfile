# Development container for solx — a thin layer over the CI runner image, so
# local builds run on the exact toolchain CI uses.
#
# The custom LLVM EVM backend is intentionally NOT part of this image (nor of
# solx-ci-runner itself): it changes with every solx-llvm submodule bump and
# weighs several GB, so baking it in would bloat and stale the image. It is
# built from source inside the container instead — see bootstrap.sh and
# docs/src/developer-guide/00-development-container.md.
#
# KEEP-IN-SYNC: when the solx-ci-runner digest is repinned across the CI
# workflows, repin it here too.

FROM ghcr.io/nomicfoundation/solx-ci-runner@sha256:79b926f62cddf00086b8a385f89e3f3281e2f37b993491e773f17d7549f7363a

# ── Non-root user with sudo ──────────────────────────────────────────────────
# CI runs as root; a devcontainer wants a UID-mapped non-root user so files
# created in the bind-mounted workspace belong to the host user (the
# devcontainer runtime remaps the UID/GID to the host's on Linux).
# ubuntu:26.04 ships a default `ubuntu` user squatting UID 1000; replace it.
ARG USERNAME=dev
RUN userdel --remove ubuntu 2>/dev/null; \
    apt-get update && apt-get install -y --no-install-recommends sudo \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 1000 --user-group --create-home --shell /bin/bash ${USERNAME} \
    && echo "${USERNAME} ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/${USERNAME} \
    && chmod 440 /etc/sudoers.d/${USERNAME}

# ── Shared rustup/cargo ──────────────────────────────────────────────────────
# The base image homes rustup in /root, unreachable for the non-root user.
# Install a shared one in the official rust-image layout instead; the actual
# toolchain is not baked in — rustup downloads the version pinned in
# rust-toolchain.toml on the first cargo invocation, so this layer never goes
# stale on a toolchain bump.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain none --profile default \
    && chmod -R a+w ${RUSTUP_HOME} ${CARGO_HOME}
