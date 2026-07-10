//!
//! The LLVM builder utilities.
//!

use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use colored::Colorize;
use path_slash::PathBufExt;

use crate::llvm::path::Path as LlvmPath;

/// The minimum required XCode version.
pub const XCODE_MIN_VERSION: u32 = 11;

/// The XCode version 15.
pub const XCODE_VERSION_15: u32 = 15;

///
/// The subprocess runner.
///
/// Passes all output through.
///
pub fn command(command: &mut Command, description: &str) -> anyhow::Result<()> {
    eprintln!("{description}: {command:?}");

    let status = command
        .status()
        .map_err(|error| anyhow::anyhow!("{command:?} process spawning error: {error:?}"))?;

    if status.code() != Some(solx_utils::EXIT_CODE_SUCCESS) {
        anyhow::bail!(
            "{command:?} subprocess failed {}",
            match status.code() {
                Some(code) => format!("with exit code {code:?}"),
                None => "without exit code".to_owned(),
            },
        );
    }

    Ok(())
}

///
/// Retrying subprocess runner.
///
/// Passes all output through and ignores failures and retries `retries` times if specified.
///
pub fn command_with_retries(
    command: &mut Command,
    description: &str,
    retries: usize,
) -> anyhow::Result<()> {
    for attempt in 0..=retries {
        eprintln!("{description} (attempt {attempt}): {command:?}");

        let status = command
            .status()
            .map_err(|error| anyhow::anyhow!("{command:?} process spawning error: {error:?}"))?;

        if status.code() == Some(solx_utils::EXIT_CODE_SUCCESS) {
            return Ok(());
        } else {
            eprintln!(
                "{command:?} subprocess failed {}",
                match status.code() {
                    Some(code) => format!("with exit code {code:?}"),
                    None => "without exit code".to_owned(),
                },
            );
        }
    }

    anyhow::bail!("{command:?} subprocess failed after {retries} retries");
}

///
/// The subprocess runner.
///
/// Returns a JSON deserialized output.
///
pub fn command_with_json_output<T: serde::de::DeserializeOwned>(
    command: &mut Command,
    description: &str,
    ignore_failure: bool,
) -> anyhow::Result<T> {
    eprintln!("{description}: {command:?}");

    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut process = command
        .spawn()
        .map_err(|error| anyhow::anyhow!("{command:?} process spawning error: {error:?}"))?;

    let stderr = process
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("{command:?} failed to take stderr"))?;
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            if line.contains(r#""$message_type":"diagnostic""#) {
                continue;
            }
            eprintln!("{line}");
        }
    });

    let result = process.wait_with_output().map_err(|error| {
        anyhow::anyhow!("{command:?} subprocess output reading error: {error:?}")
    })?;

    if !ignore_failure && result.status.code() != Some(solx_utils::EXIT_CODE_SUCCESS) {
        anyhow::bail!(
            "{command:?} subprocess failed {}:\n{}\n{}",
            match result.status.code() {
                Some(code) => format!("with exit code {code:?}"),
                None => "without exit code".to_owned(),
            },
            String::from_utf8_lossy(result.stdout.as_slice()),
            String::from_utf8_lossy(result.stderr.as_slice()),
        );
    }

    solx_utils::deserialize_from_slice::<T>(result.stdout.as_slice())
        .map_err(|error| anyhow::anyhow!("{command:?} output parsing: {error:?}"))
}

///
/// Clones a git repository into the given directory.
///
/// When `commit` is `Some(sha)`, performs a shallow fetch of the exact commit:
/// `git init` -> `git remote add` -> `git fetch --depth 1 origin <sha>` -> `git checkout FETCH_HEAD`
/// -> `git submodule update --init --depth 1 --recursive` (only if `.gitmodules` exists).
///
/// When `commit` is `None`, falls back to `git clone --depth 1 --recurse-submodules --shallow-submodules`.
///
pub fn clone_repository(
    url: &str,
    directory: &str,
    commit: Option<&str>,
    description: &str,
) -> anyhow::Result<()> {
    if let Some(sha) = commit {
        if sha.len() != 40 || !sha.chars().all(|character| character.is_ascii_hexdigit()) {
            anyhow::bail!("commit must be a 40-character hex SHA, got: {sha}");
        }

        let mut init_command = Command::new("git");
        init_command.args(["init", directory]);
        command(&mut init_command, description)?;

        let mut remote_command = Command::new("git");
        remote_command.args(["-C", directory, "remote", "add", "origin", url]);
        command(&mut remote_command, description)?;

        let mut fetch_command = Command::new("git");
        fetch_command.args(["-C", directory, "fetch", "--depth", "1", "origin", sha]);
        command_with_retries(&mut fetch_command, description, 16)?;

        let mut checkout_command = Command::new("git");
        checkout_command.args(["-C", directory, "checkout", "FETCH_HEAD"]);
        command(&mut checkout_command, description)?;

        let gitmodules_path = std::path::Path::new(directory).join(".gitmodules");
        if gitmodules_path.exists() {
            let mut submodule_command = Command::new("git");
            submodule_command.args([
                "-C",
                directory,
                "submodule",
                "update",
                "--init",
                "--depth",
                "1",
                "--recursive",
            ]);
            command_with_retries(&mut submodule_command, description, 16)?;
        }
    } else {
        let mut clone_command = Command::new("git");
        clone_command.arg("clone");
        clone_command.args(["--depth", "1"]);
        clone_command.arg("--recurse-submodules");
        clone_command.arg("--shallow-submodules");
        clone_command.arg(url);
        clone_command.arg(directory);
        command_with_retries(&mut clone_command, description, 16)?;
    }

    Ok(())
}

///
/// Removes the project directory after building and testing.
///
pub fn remove(project_directory: &Path, project_name: &str) -> anyhow::Result<()> {
    if !project_directory.exists() {
        return Ok(());
    }

    eprintln!(
        "{} project {}",
        solx_utils::cargo_status_ok("Removing"),
        project_name.bright_white().bold()
    );
    std::fs::remove_dir_all(project_directory).map_err(|error| {
        anyhow::anyhow!(
            "{} project directory {project_directory:?}: {error}",
            solx_utils::cargo_status_ok("Removing"),
        )
    })?;

    Ok(())
}

///
/// Call ninja to build and install LLVM via the given install target.
///
/// All platforms pass `"install-distribution"` together with
/// `LLVM_DISTRIBUTION_COMPONENTS` to skip the ~200 LLVM tool binaries solx
/// doesn't use (it consumes LLVM as a library via inkwell). See #364.
///
/// When `llvm-lit` exists in the build tree (produced by `--enable-utils`),
/// it is copied into the install prefix's `bin/` afterwards. Upstream LLVM
/// has no install rule for `llvm-lit` (it's only `configure_file`'d into the
/// build dir), so listing it in `LLVM_DISTRIBUTION_COMPONENTS` errors at
/// configure time; the manual copy bridges that gap so the cached artifact
/// contains `llvm-lit` like the rest of the toolchain.
///
pub fn ninja(build_dir: &Path, install_target: &str) -> anyhow::Result<()> {
    let mut ninja = Command::new("ninja");
    let build_dir_str = build_dir.to_string_lossy();
    ninja.args(["-C", &*build_dir_str]);
    command(
        ninja.arg(install_target),
        &format!("Running ninja {install_target}"),
    )?;
    let lit_name = if cfg!(windows) {
        "llvm-lit.py"
    } else {
        "llvm-lit"
    };
    let lit_source = build_dir.join("bin").join(lit_name);
    if lit_source.exists() {
        let install_bin = LlvmPath::llvm_target_final()?.join("bin");
        std::fs::create_dir_all(&install_bin)?;
        std::fs::copy(&lit_source, install_bin.join(lit_name))?;
    }
    Ok(())
}

///
/// Create an absolute path, appending it to the current working directory.
///
pub fn absolute_path<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    let mut full_path = std::env::current_dir()?;
    full_path.push(path);
    Ok(full_path)
}

///
/// Converts a Windows path into a Unix path.
///
pub fn path_windows_to_unix<P: AsRef<Path> + PathBufExt>(path: P) -> anyhow::Result<PathBuf> {
    path.to_slash()
        .map(|pathbuf| PathBuf::from(pathbuf.to_string()))
        .ok_or_else(|| anyhow::anyhow!("Windows-to-Unix path conversion error"))
}

///
/// Checks if the tool exists in the system.
///
pub fn exists(name: &str) -> anyhow::Result<()> {
    let mut log_string = format!("{} for `{name}`: ", solx_utils::cargo_status_ok("Looking"));

    let mut command = Command::new("which");
    command.arg(name);

    command.stdout(Stdio::piped());
    let process = command
        .spawn()
        .map_err(|error| anyhow::anyhow!("{command:?} process spawning error: {error:?}"))?;

    let result = process.wait_with_output().map_err(|error| {
        anyhow::anyhow!("{command:?} subprocess output reading error: {error:?}")
    })?;

    let log_result = if !result.status.success() {
        solx_utils::cargo_status_error("not found")
    } else {
        String::from_utf8_lossy(result.stdout.as_slice())
            .trim()
            .to_owned()
    };
    log_string.push_str(log_result.as_str());
    eprintln!("{log_string}");
    if !result.status.success() {
        anyhow::bail!("Tool `{name}` not found in the system");
    }
    Ok(())
}

///
/// Reads a file, applies sed-like regex patterns, and writes the file back.
///
pub fn sed_file<P: AsRef<Path>>(file_path: P, patterns: &[&str]) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(&file_path)
        .map_err(|error| anyhow::anyhow!("Reading file {:?}: {error}", file_path.as_ref()))?;
    let modified_content =
        sedregex::find_and_replace(content.as_str(), patterns).map_err(|error| {
            anyhow::anyhow!(
                "Applying sed-like patterns to file {:?}: {error}",
                file_path.as_ref()
            )
        })?;
    if modified_content != content {
        std::fs::write(&file_path, modified_content.to_string())
            .map_err(|error| anyhow::anyhow!("Writing file {:?}: {error}", file_path.as_ref()))?;
    }
    Ok(())
}

///
/// A native shim binary that wraps a compiler executable and records every
/// invocation in a marker file.
///
/// Build systems resolve compilers through their own configuration layers, so
/// a harness bug can leave them silently compiling with a bundled compiler
/// while the report attributes the results to the configured one (see #497).
/// Passing the shim path instead of the real compiler lets test runners verify
/// after each build that the configured compiler was actually invoked.
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
    /// File created by the shim on every compiler invocation.
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
    let _ = std::fs::write({marker_path:?}, b\"\");
    let error = std::os::unix::process::CommandExt::exec(
        std::process::Command::new({compiler_path:?}).args(std::env::args_os().skip(1)),
    );
    eprintln!(\"compiler invocation shim: {{error}}\");
    std::process::exit(127);
}}
"
            );
            std::fs::write(source_path.as_path(), source).map_err(|error| {
                anyhow::anyhow!("Writing compiler shim source {source_path:?}: {error}")
            })?;
            let mut rustc_command = Command::new("rustc");
            rustc_command.arg("-O");
            rustc_command.arg(source_path.as_path());
            rustc_command.arg("-o");
            rustc_command.arg(shim_path.as_path());
            command(
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
    /// Errors if the compiler was never invoked since the last `reset`.
    ///
    pub fn verify(&self, toolchain_name: &str, project_name: &str) -> anyhow::Result<()> {
        if let Some(marker_path) = self.marker_path.as_deref()
            && !marker_path.exists()
        {
            anyhow::bail!(
                "Harness self-check failed: project {project_name} built successfully with {toolchain_name}, \
                but the configured compiler was never invoked — the build system used another compiler \
                and the results would be attributed to the wrong toolchain (see #497)."
            );
        }
        Ok(())
    }
}

///
/// Identify XCode version using `pkgutil`.
///
pub fn get_xcode_version() -> anyhow::Result<u32> {
    let pkgutil = Command::new("pkgutil")
        .args(["--pkg-info", "com.apple.pkg.CLTools_Executables"])
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| anyhow::anyhow!("`pkgutil` process: {error}"))?;
    let grep_version = Command::new("grep")
        .arg("version")
        .stdin(Stdio::from(pkgutil.stdout.ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to identify XCode version - XCode or CLI tools are not installed"
            )
        })?))
        .output()
        .map_err(|error| anyhow::anyhow!("`grep` process: {error}"))?;
    let version_string = String::from_utf8(grep_version.stdout)?;
    let version_regex = regex::Regex::new(r"version: (\d+)\..*")?;
    let captures = version_regex
        .captures(version_string.as_str())
        .ok_or(anyhow::anyhow!(
            "Failed to parse XCode version: {version_string}"
        ))?;
    let xcode_version: u32 = captures
        .get(1)
        .expect("Always has a major version")
        .as_str()
        .parse()
        .map_err(|error| anyhow::anyhow!("Failed to parse XCode version: {error}"))?;
    Ok(xcode_version)
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
            shim.verify("toolchain", "project").is_ok(),
            "must pass after the compiler is invoked"
        );

        shim.reset().expect("Always valid");
        assert!(
            shim.verify("toolchain", "project").is_err(),
            "must fail again after a reset"
        );

        std::fs::remove_dir_all(directory).expect("Always valid");
    }
}
