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
        .unwrap_or_else(|error| panic!("{command:?} process spawning error: {error:?}"));

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
            .unwrap_or_else(|error| panic!("{command:?} process spawning error: {error:?}"));

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
        .unwrap_or_else(|error| panic!("{command:?} process spawning error: {error:?}"));

    let stderr = process.stderr.take().expect("Failed to take stderr");
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            if line.contains(r#""$message_type":"diagnostic""#) {
                continue;
            }
            eprintln!("{line}");
        }
    });

    let result = process
        .wait_with_output()
        .unwrap_or_else(|error| panic!("{command:?} subprocess output reading error: {error:?}"));

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
/// Call ninja to build the LLVM.
///
pub fn ninja(build_dir: &Path) -> anyhow::Result<()> {
    let mut ninja = Command::new("ninja");
    let build_dir_str = build_dir.to_string_lossy();
    ninja.args(["-C", &*build_dir_str]);
    command(ninja.arg("install"), "Running ninja install")?;
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
        .unwrap_or_else(|error| panic!("{command:?} process spawning error: {error:?}"));

    let result = process
        .wait_with_output()
        .unwrap_or_else(|error| panic!("{command:?} subprocess output reading error: {error:?}"));

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
        .stdin(Stdio::from(pkgutil.stdout.expect(
            "Failed to identify XCode version - XCode or CLI tools are not installed",
        )))
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
