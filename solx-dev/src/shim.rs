//!
//! The compiler invocation shim for harness self-verification.
//!

use std::path::Path;
use std::path::PathBuf;

///
/// A native shim binary that wraps a compiler executable and records the
/// arguments of every invocation in a marker file.
///
/// Build systems resolve compilers through their own configuration layers, so
/// a harness bug can leave them silently compiling with a bundled compiler
/// while the report attributes the results to the configured one (see #497).
/// Passing the shim path instead of the real compiler lets test runners verify
/// after each build that the configured compiler actually compiled something.
///
/// The shim must be a real native executable, not a script: Hardhat 3 sniffs
/// the compiler file's content (`isBinaryFile`) and loads anything that looks
/// like text as a solc-js module. It is therefore generated as a tiny Rust
/// program and compiled with `rustc` at harness startup (always available,
/// since `solx-dev` itself is built by cargo).
///
pub struct CompilerInvocationShim {
    /// Path to pass to the build system instead of the real compiler.
    pub compiler_path: PathBuf,
    /// File the shim appends each invocation's arguments to.
    /// `None` on platforms without shims, where verification is disabled.
    marker_path: Option<PathBuf>,
}

impl CompilerInvocationShim {
    ///
    /// Creates a shim for `compiler_path` inside `directory`.
    ///
    /// On non-Unix hosts the real compiler path is passed through unchanged
    /// and `verify` always succeeds.
    ///
    pub fn new(compiler_path: PathBuf, directory: &Path) -> anyhow::Result<Self> {
        #[cfg(unix)]
        {
            std::fs::create_dir_all(directory).map_err(|error| {
                anyhow::anyhow!("Creating compiler shim directory {directory:?}: {error}")
            })?;
            let marker_path = directory.join("invoked");
            let shim_path = directory.join(compiler_path.file_name().ok_or_else(|| {
                anyhow::anyhow!("Compiler path {compiler_path:?} has no file name")
            })?);
            let source_path = directory.join("shim.rs");
            let source = format!(
                "fn main() {{
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let mut line = args
        .iter()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(\" \");
    line.push('\\n');
    if let Ok(mut marker) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open({marker_path:?})
    {{
        let _ = std::io::Write::write_all(&mut marker, line.as_bytes());
    }}
    let error = std::os::unix::process::CommandExt::exec(
        std::process::Command::new({compiler_path:?}).args(args),
    );
    eprintln!(\"compiler invocation shim: {{error}}\");
    std::process::exit(127);
}}
"
            );
            std::fs::write(source_path.as_path(), source).map_err(|error| {
                anyhow::anyhow!("Writing compiler shim source {source_path:?}: {error}")
            })?;
            let mut rustc_command = std::process::Command::new("rustc");
            rustc_command.arg("-O");
            rustc_command.arg(source_path.as_path());
            rustc_command.arg("-o");
            rustc_command.arg(shim_path.as_path());
            crate::utils::command(
                &mut rustc_command,
                format!(
                    "{} compiler invocation shim for {compiler_path:?}",
                    solx_utils::cargo_status_ok("Compiling"),
                )
                .as_str(),
            )?;
            Ok(Self {
                compiler_path: shim_path,
                marker_path: Some(marker_path),
            })
        }
        #[cfg(not(unix))]
        {
            let _ = directory;
            Ok(Self {
                compiler_path,
                marker_path: None,
            })
        }
    }

    ///
    /// Clears the invocation marker; call before each build.
    ///
    pub fn reset(&self) -> anyhow::Result<()> {
        if let Some(marker_path) = self.marker_path.as_deref()
            && marker_path.exists()
        {
            std::fs::remove_file(marker_path).map_err(|error| {
                anyhow::anyhow!("Removing compiler invocation marker {marker_path:?}: {error}")
            })?;
        }
        Ok(())
    }

    ///
    /// Errors unless a compile invocation was recorded since the last `reset`.
    ///
    /// Merely being invoked is not proof of compiling: Forge and Hardhat probe
    /// the configured compiler with `--version` before deciding how to compile,
    /// so `verify` requires an invocation with `--standard-json` — the interface
    /// both harnesses drive compilers through.
    ///
    pub fn verify(&self, toolchain_name: &str, project_name: &str) -> anyhow::Result<()> {
        let Some(marker_path) = self.marker_path.as_deref() else {
            return Ok(());
        };
        let invocations = match std::fs::read_to_string(marker_path) {
            Ok(invocations) => invocations,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => anyhow::bail!(
                "Harness self-check failed: project {project_name} built successfully with {toolchain_name}, \
                but the configured compiler was never invoked — the build system used another compiler \
                and the results would be attributed to the wrong toolchain (see #497)."
            ),
            Err(error) => {
                anyhow::bail!("Reading compiler invocation marker {marker_path:?}: {error}")
            }
        };
        if !invocations.lines().any(|invocation| {
            invocation
                .split_whitespace()
                .any(|argument| argument == "--standard-json")
        }) {
            anyhow::bail!(
                "Harness self-check failed: project {project_name} built successfully with {toolchain_name}, \
                but the configured compiler was only probed ({invocations:?}) and never compiled — \
                the build system compiled with another compiler and the results would be attributed \
                to the wrong toolchain (see #497)."
            );
        }
        Ok(())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::process::Command;

    use super::CompilerInvocationShim;

    #[test]
    fn shim_records_compiler_invocation() {
        let directory =
            std::env::temp_dir().join(format!("solx-dev-shim-test-{}", std::process::id()));
        let compiler_path = directory.join("fake-compiler");

        std::fs::create_dir_all(directory.as_path()).expect("Always valid");
        std::fs::write(compiler_path.as_path(), "#!/bin/sh\nexit 0\n").expect("Always valid");
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                compiler_path.as_path(),
                std::fs::Permissions::from_mode(0o755),
            )
            .expect("Always valid");
        }

        let shim = CompilerInvocationShim::new(compiler_path, directory.join("shim").as_path())
            .expect("Always valid");
        shim.reset().expect("Always valid");
        assert!(
            shim.verify("toolchain", "project").is_err(),
            "must fail before the compiler is invoked"
        );

        let status = Command::new(shim.compiler_path.as_path())
            .arg("--version")
            .status()
            .expect("Always valid");
        assert!(status.success(), "shim must exec the wrapped compiler");
        assert!(
            shim.verify("toolchain", "project").is_err(),
            "a version probe must not count as a compile"
        );

        let status = Command::new(shim.compiler_path.as_path())
            .arg("--standard-json")
            .status()
            .expect("Always valid");
        assert!(status.success(), "shim must exec the wrapped compiler");
        assert!(
            shim.verify("toolchain", "project").is_ok(),
            "must pass after a compile invocation"
        );

        shim.reset().expect("Always valid");
        assert!(
            shim.verify("toolchain", "project").is_err(),
            "must fail again after a reset"
        );

        std::fs::remove_dir_all(directory).expect("Always valid");
    }
}
