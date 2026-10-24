//!
//! The LLVM builder utilities.
//!

use std::borrow::Cow;
use std::ffi::OsStr;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::LazyLock;

use colored::Colorize;
use path_slash::PathBufExt;
use regex::Regex;

use crate::llvm::path::Path as LlvmPath;

/// Version-expression characters only, so `pragma solidity` in a comment never swallows code up
/// to the next `;`.
static VERSION_PRAGMA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"pragma\s+solidity\s+[0-9xX*.^~<>=|\s-]+;").expect("Always valid")
});

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
/// Returns the standard output.
///
pub fn command_with_output(command: &mut Command, description: &str) -> anyhow::Result<String> {
    eprintln!("{description}: {command:?}");

    let output = command
        .output()
        .map_err(|error| anyhow::anyhow!("{command:?} process spawning error: {error:?}"))?;

    if output.status.code() != Some(solx_utils::EXIT_CODE_SUCCESS) {
        anyhow::bail!(
            "{command:?} subprocess failed {}:\n{}",
            match output.status.code() {
                Some(code) => format!("with exit code {code:?}"),
                None => "without exit code".to_owned(),
            },
            String::from_utf8_lossy(output.stderr.as_slice()),
        );
    }

    String::from_utf8(output.stdout)
        .map_err(|error| anyhow::anyhow!("{command:?} output is not UTF-8: {error}"))
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
/// Lists the tracked files of the repository at `directory` matching `pathspec`, submodules included.
///
pub fn git_tracked_files(directory: &Path, pathspec: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut command = Command::new("git");
    command.arg("-C");
    command.arg(directory);
    command.args(["ls-files", "-z", "--recurse-submodules", "--", pathspec]);
    Ok(command_with_output(
        &mut command,
        format!("{} tracked files", solx_utils::cargo_status_ok("Listing")).as_str(),
    )?
    .split('\0')
    .filter(|path| !path.is_empty())
    .map(|path| directory.join(path))
    .collect())
}

///
/// Replaces `destination` with a copy of `source` and rewrites the copy's version pragmas
/// to `>=solidity_version`.
///
pub fn copy_project(
    source: &Path,
    destination: &Path,
    project_name: &str,
    solidity_version: &str,
    rewrite_pragmas_in: &[String],
) -> anyhow::Result<()> {
    remove(destination, project_name)?;
    let mut copy_command = Command::new("cp");
    copy_command.arg("-a");
    copy_command.arg(source);
    copy_command.arg(destination);
    command(
        &mut copy_command,
        format!(
            "{} project {}",
            solx_utils::cargo_status_ok("Copying"),
            project_name.bright_white().bold()
        )
        .as_str(),
    )?;

    eprintln!(
        "{} pragmas in project {}",
        solx_utils::cargo_status_ok("Fixing"),
        project_name.bright_white().bold()
    );
    // Package-manager installs keep their own pragmas unless the project lists them.
    for solidity_file in git_tracked_files(destination, "*.sol")? {
        rewrite_version_pragmas_in_file(
            solidity_file.as_path(),
            format!("pragma solidity >={solidity_version};").as_str(),
        )?;
    }
    for directory in rewrite_pragmas_in {
        let directory = destination.join(directory);
        if !directory.is_dir() {
            anyhow::bail!("Project {project_name}: {directory:?} not found");
        }
        rewrite_version_pragmas_under(directory.as_path(), solidity_version)?;
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
/// Returns what `program --version` prints, which identifies the tool in cache keys and reports.
///
pub fn tool_version(program: impl AsRef<OsStr>) -> anyhow::Result<String> {
    let program = program.as_ref();
    let mut command = Command::new(program);
    command.arg("--version");
    command_with_output(
        &mut command,
        format!(
            "{} version of {}",
            solx_utils::cargo_status_ok("Checking"),
            program.to_string_lossy()
        )
        .as_str(),
    )
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
/// Replaces every version pragma in `source` with `pragma`.
///
pub fn rewrite_version_pragmas<'a>(source: &'a str, pragma: &str) -> Cow<'a, str> {
    VERSION_PRAGMA.replace_all(source, regex::NoExpand(pragma))
}

///
/// Replaces every version pragma in the file with `pragma`, writing it back when it changed.
///
pub fn rewrite_version_pragmas_in_file(path: &Path, pragma: &str) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("Reading file {path:?}: {error}"))?;
    if let Cow::Owned(rewritten) = rewrite_version_pragmas(content.as_str(), pragma) {
        std::fs::write(path, rewritten)
            .map_err(|error| anyhow::anyhow!("Writing file {path:?}: {error}"))?;
    }
    Ok(())
}

///
/// Replaces every version pragma under `directory` with `pragma solidity >=<version>;`.
///
/// A dependency can pin an exact version pragma, so a project lists the installed
/// directories whose pragmas are rewritten along with the git-tracked sources.
///
pub fn rewrite_version_pragmas_under(
    directory: &Path,
    solidity_version: &str,
) -> anyhow::Result<()> {
    let pragma = format!("pragma solidity >={solidity_version};");
    for solidity_file in glob::glob(format!("{}/**/*.sol", directory.to_string_lossy()).as_str())
        .expect("Always valid")
        .filter_map(Result::ok)
    {
        if !solidity_file.is_file() {
            continue;
        }
        rewrite_version_pragmas_in_file(solidity_file.as_path(), pragma.as_str())?;
    }
    Ok(())
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
    use std::path::Path;
    use std::process::Command;

    fn test_directory(name: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("solx-dev-utils-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(directory.as_path());
        std::fs::create_dir_all(directory.as_path()).expect("Always valid");
        directory
    }

    fn git(directory: &Path, arguments: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(["-c", "protocol.file.allow=always"])
            .args(["-c", "user.name=solx", "-c", "user.email=solx@localhost"])
            .args(arguments)
            .status()
            .expect("Always valid");
        assert!(status.success(), "git {arguments:?} failed");
    }

    fn commit_file(repository: &Path, path: &str) {
        std::fs::create_dir_all(repository.join(path).parent().expect("Always exists"))
            .expect("Always valid");
        std::fs::write(repository.join(path), "pragma solidity 0.8.0;").expect("Always valid");
        git(repository, &["add", path]);
        git(repository, &["commit", "--quiet", "-m", path]);
    }

    #[test]
    fn git_tracked_files_follows_submodules_and_skips_untracked() {
        let directory = test_directory("tracked");
        let dependency = directory.join("dependency");
        let project = directory.join("project");
        git(directory.as_path(), &["init", "--quiet", "dependency"]);
        commit_file(dependency.as_path(), "src/Dependency.sol");
        git(directory.as_path(), &["init", "--quiet", "project"]);
        commit_file(project.as_path(), "src/Project.sol");
        git(
            project.as_path(),
            &[
                "submodule",
                "--quiet",
                "add",
                dependency.to_string_lossy().as_ref(),
                "lib/dependency",
            ],
        );
        git(project.as_path(), &["commit", "--quiet", "-m", "submodule"]);
        commit_file(project.as_path(), "node_modules/.keep");
        std::fs::write(project.join("node_modules/Installed.sol"), "").expect("Always valid");

        let mut files = super::git_tracked_files(project.as_path(), "*.sol")
            .expect("Always valid")
            .into_iter()
            .map(|path| {
                path.strip_prefix(project.as_path())
                    .expect("Always valid")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();
        files.sort();
        assert_eq!(
            files,
            ["lib/dependency/src/Dependency.sol", "src/Project.sol"]
        );

        std::fs::remove_dir_all(directory).expect("Always valid");
    }

    #[test]
    fn tool_version_fails_for_missing_tool() {
        assert!(
            super::tool_version("git")
                .expect("Always valid")
                .starts_with("git version")
        );
        assert!(super::tool_version("solx-dev-missing-tool").is_err());
    }

    #[test]
    fn copy_rewrites_pragmas_in_the_copy_only_and_keeps_symlinks() {
        let directory = test_directory("copy");
        let source = directory.join("source");
        git(directory.as_path(), &["init", "--quiet", "source"]);
        commit_file(source.as_path(), "src/Project.sol");
        std::os::unix::fs::symlink("src/Project.sol", source.join("link")).expect("Always valid");

        std::fs::create_dir_all(source.join("node_modules/listed")).expect("Always valid");
        std::fs::write(
            source.join("node_modules/listed/Listed.sol"),
            "pragma solidity 0.8.0;",
        )
        .expect("Always valid");
        std::fs::create_dir_all(source.join("node_modules/unlisted")).expect("Always valid");
        std::fs::write(
            source.join("node_modules/unlisted/Unlisted.sol"),
            "pragma solidity 0.8.0;",
        )
        .expect("Always valid");

        let destination = directory.join("work");
        super::copy_project(
            source.as_path(),
            destination.as_path(),
            "project",
            "0.8.34",
            &["node_modules/listed".to_owned()],
        )
        .expect("Always valid");
        assert_eq!(
            std::fs::read_to_string(destination.join("src/Project.sol")).expect("Always valid"),
            "pragma solidity >=0.8.34;"
        );
        assert_eq!(
            std::fs::read_to_string(source.join("src/Project.sol")).expect("Always valid"),
            "pragma solidity 0.8.0;",
            "the rewrite must not reach the source"
        );
        assert_eq!(
            std::fs::read_to_string(destination.join("node_modules/listed/Listed.sol"))
                .expect("Always valid"),
            "pragma solidity >=0.8.34;",
            "a listed install directory is rewritten"
        );
        assert_eq!(
            std::fs::read_to_string(destination.join("node_modules/unlisted/Unlisted.sol"))
                .expect("Always valid"),
            "pragma solidity 0.8.0;",
            "an unlisted install directory keeps its pragmas"
        );
        assert!(
            super::copy_project(
                source.as_path(),
                destination.as_path(),
                "project",
                "0.8.34",
                &["node_modules/absent".to_owned()],
            )
            .is_err(),
            "a listed directory that does not exist is an error"
        );

        std::fs::write(destination.join("output"), "new").expect("Always valid");
        super::copy_project(
            source.as_path(),
            destination.as_path(),
            "project",
            "0.8.34",
            &[],
        )
        .expect("Always valid");
        assert!(
            !destination.join("output").exists(),
            "a copy must not keep the previous toolchain's outputs"
        );
        assert!(
            destination
                .join("link")
                .symlink_metadata()
                .expect("Always valid")
                .file_type()
                .is_symlink()
        );

        std::fs::remove_dir_all(directory).expect("Always valid");
    }
}
