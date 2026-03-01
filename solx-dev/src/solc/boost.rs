//!
//! Boost library download and build utilities for solc.
//!

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use sha2::Digest;

/// Default Boost version.
pub const DEFAULT_BOOST_VERSION: &str = "1.83.0";

/// Boost download URL template.
const BOOST_DOWNLOAD_URL: &str = "https://archives.boost.io/release";

/// SHA256 checksum of `boost_1_83_0.tar.gz` from the official Boost release page.
const BOOST_SHA256: &str = "c0685b68dd44cc46574cce86c4e17c0f611b15e195be9848dfd0769a0a207628";

/// Marker file to track the installed Boost version.
const BOOST_VERSION_MARKER: &str = ".solx-boost-version";

/// Boost libraries required for solc.
const BOOST_LIBRARIES: [&str; 10] = [
    "filesystem",
    "system",
    "program_options",
    "test",
    "thread",
    "date_time",
    "regex",
    "chrono",
    "random",
    "atomic",
];

///
/// Boost configuration.
///
#[derive(Debug, Clone)]
pub struct BoostConfig {
    /// Boost version (e.g., "1.83.0").
    pub version: String,
    /// Base directory where Boost is installed.
    pub base_dir: PathBuf,
}

impl BoostConfig {
    ///
    /// Creates a new Boost configuration.
    ///
    pub fn new(version: String, base_dir: PathBuf) -> Self {
        Self { version, base_dir }
    }

    ///
    /// Returns the Boost library directory.
    ///
    pub fn lib_dir(&self) -> PathBuf {
        self.base_dir.join("lib")
    }

    ///
    /// Returns the Boost include directory.
    ///
    pub fn include_dir(&self) -> PathBuf {
        self.base_dir.join("include")
    }

    ///
    /// Returns the Boost include directory for Windows.
    ///
    /// Windows uses a versioned include directory (e.g., `include/boost-1_83`).
    ///
    pub fn windows_include_dir(&self) -> PathBuf {
        let short_version = self
            .version
            .split('.')
            .take(2)
            .collect::<Vec<_>>()
            .join("_");
        self.base_dir
            .join("include")
            .join(format!("boost-{short_version}"))
    }

    ///
    /// Returns the source directory for the downloaded Boost.
    ///
    pub fn source_dir(&self, working_dir: &Path) -> PathBuf {
        let filename = format!("boost_{}", self.version.replace('.', "_"));
        working_dir.join(filename)
    }

    ///
    /// Returns the archive filename.
    ///
    pub fn archive_filename(&self) -> String {
        format!("boost_{}.tar.gz", self.version.replace('.', "_"))
    }

    ///
    /// Returns the download URL.
    ///
    pub fn download_url(&self) -> String {
        format!(
            "{BOOST_DOWNLOAD_URL}/{}/source/{}",
            self.version,
            self.archive_filename()
        )
    }
}

///
/// Downloads and builds Boost.
///
/// Returns the absolute path where Boost was installed.
///
pub fn download_and_build(
    working_dir: &Path,
    boost_config: &BoostConfig,
) -> anyhow::Result<PathBuf> {
    // Canonicalize working directory to get absolute paths
    let working_dir = normalize_path_buf(&working_dir.canonicalize()?);
    let install_prefix = normalize_path_buf(&working_dir.join("boost"));

    // Skip if already built
    if install_prefix.join("lib").exists() {
        if let Some(existing_version) = read_boost_version_marker(&install_prefix) {
            if existing_version == boost_config.version {
                eprintln!(
                    "Boost {} already exists at {}",
                    boost_config.version,
                    install_prefix.display()
                );
                return Ok(install_prefix);
            }
            eprintln!(
                "Boost version mismatch at {}: found {}, requested {}. Rebuilding.",
                install_prefix.display(),
                existing_version,
                boost_config.version
            );
        } else {
            eprintln!(
                "Boost exists at {} but version marker is missing. Rebuilding.",
                install_prefix.display()
            );
        }
        std::fs::remove_dir_all(&install_prefix)?;
    }

    eprintln!("Downloading Boost {}...", boost_config.version);

    // Download
    let archive_path = working_dir.join(boost_config.archive_filename());
    if !archive_path.exists() {
        download(&boost_config.download_url(), &archive_path)?;
        verify_checksum(&archive_path, BOOST_SHA256)?;
    }

    // Extract
    let source_dir = boost_config.source_dir(&working_dir);
    if !source_dir.exists() {
        extract(&archive_path, &working_dir)?;
    }

    // Bootstrap with absolute install path
    bootstrap(&source_dir, &install_prefix)?;

    // Build
    build(&source_dir)?;

    write_boost_version_marker(&install_prefix, &boost_config.version)?;

    eprintln!(
        "Boost {} built successfully at {}",
        boost_config.version,
        install_prefix.display()
    );

    Ok(install_prefix)
}

///
/// Downloads a file from URL.
///
fn download(url: &str, output_path: &Path) -> anyhow::Result<()> {
    let mut curl = Command::new("curl");
    curl.arg("-L");
    curl.arg("-o");
    curl.arg(normalize_path_for_shell(output_path));
    curl.arg(url);

    crate::utils::command(&mut curl, "Downloading Boost")?;
    Ok(())
}

///
/// Verifies the SHA256 checksum of a downloaded file.
///
fn verify_checksum(file_path: &Path, expected_hex: &str) -> anyhow::Result<()> {
    eprintln!("Verifying checksum of {}...", file_path.display());
    let file_bytes = std::fs::read(file_path)?;
    let actual_hex = hex::encode(sha2::Sha256::digest(&file_bytes));
    if actual_hex != expected_hex {
        // Remove the corrupted download so it isn't reused on retry
        let _ = std::fs::remove_file(file_path);
        anyhow::bail!(
            "SHA256 checksum mismatch for {}:\n  expected: {}\n  actual:   {}",
            file_path.display(),
            expected_hex,
            actual_hex
        );
    }
    eprintln!("Checksum verified.");
    Ok(())
}

///
/// Extracts a tar.gz archive.
///
fn extract(archive_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
    // Use the filename only since we're setting current_dir to output_dir
    let archive_filename = archive_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid archive path: {}", archive_path.display()))?;

    let output_dir = normalize_path_buf(output_dir);
    let mut tar = Command::new("tar");
    tar.arg("xzf");
    tar.arg(archive_filename);
    tar.current_dir(&output_dir);

    crate::utils::command(&mut tar, "Extracting Boost")?;
    Ok(())
}

///
/// Runs Boost bootstrap script.
///
fn bootstrap(source_dir: &Path, install_prefix: &Path) -> anyhow::Result<()> {
    // Convert path to string for passing to shell
    let install_prefix_str = normalize_path_for_shell(install_prefix);

    #[cfg(target_os = "windows")]
    {
        // On Windows/MSYS2, run bootstrap.sh through sh
        // Use "sh" instead of "bash" because "bash" on Windows PATH often
        // resolves to WSL's bash, while "sh" resolves to MSYS2's shell
        let mut bootstrap = Command::new("sh");
        bootstrap.current_dir(source_dir);
        bootstrap.arg("-c");
        bootstrap.arg(format!("./bootstrap.sh --prefix={}", install_prefix_str));
        crate::utils::command(&mut bootstrap, "Boost bootstrap")?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let mut bootstrap = Command::new("./bootstrap.sh");
        bootstrap.current_dir(source_dir);
        bootstrap.arg(format!("--prefix={}", install_prefix_str));

        // On macOS x86_64, specify Python version for compatibility
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            bootstrap.arg("--with-python-version=2.7");
        }

        crate::utils::command(&mut bootstrap, "Boost bootstrap")?;
    }

    Ok(())
}

///
/// Normalizes a path for use with MSYS2 shell commands.
///
/// On Windows, converts backslashes to forward slashes (e.g., C:\foo\bar -> C:/foo/bar).
/// This format works with both native Windows tools and MSYS2 shell commands.
///
fn normalize_path_for_shell(path: &Path) -> String {
    let path = normalize_path_buf(path);

    #[cfg(target_os = "windows")]
    {
        // Convert backslashes to forward slashes for shell compatibility
        // Keep the Windows drive letter format (C:/...) which works in MSYS2
        return path.display().to_string().replace('\\', "/");
    }

    #[cfg(not(target_os = "windows"))]
    path.display().to_string()
}

///
/// Normalizes a path, returning a PathBuf.
///
/// On Windows, removes the extended-length path prefix (\\?\) that canonicalize() adds.
///
fn normalize_path_buf(path: &Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let path_str = path.display().to_string();
        // Strip the \\?\ prefix that Windows canonicalize() adds
        if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }

    path.to_path_buf()
}

///
/// Reads the Boost version marker if present.
///
fn read_boost_version_marker(install_prefix: &Path) -> Option<String> {
    let marker_path = install_prefix.join(BOOST_VERSION_MARKER);
    std::fs::read_to_string(marker_path)
        .ok()
        .map(|contents| contents.trim().to_owned())
        .filter(|contents| !contents.is_empty())
}

///
/// Writes the Boost version marker.
///
fn write_boost_version_marker(install_prefix: &Path, version: &str) -> anyhow::Result<()> {
    let marker_path = install_prefix.join(BOOST_VERSION_MARKER);
    std::fs::write(marker_path, format!("{version}\n"))?;
    Ok(())
}

///
/// Builds Boost with b2.
///
fn build(source_dir: &Path) -> anyhow::Result<()> {
    let job_count = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1);

    let with_libraries: Vec<String> = BOOST_LIBRARIES
        .iter()
        .map(|lib| format!("--with-{lib}"))
        .collect();

    #[cfg(target_os = "windows")]
    let b2_cmd = {
        // Build b2 command string for sh
        // Use "sh" instead of "bash" because "bash" on Windows PATH often
        // resolves to WSL's bash, while "sh" resolves to MSYS2's shell
        let b2_args = format!(
            "./b2 -d0 link=static runtime-link=static variant=release threading=multi address-model=64 {} -j{} install",
            with_libraries.join(" "),
            job_count
        );
        let mut cmd = Command::new("sh");
        cmd.current_dir(source_dir);
        cmd.arg("-c");
        cmd.arg(b2_args);
        cmd
    };

    #[cfg(not(target_os = "windows"))]
    let b2_cmd = {
        let mut cmd = Command::new("./b2");
        cmd.current_dir(source_dir);
        cmd.arg("-d0"); // Suppress output
        cmd.arg("link=static");
        cmd.arg("runtime-link=static");
        cmd.arg("variant=release");
        cmd.arg("threading=multi");
        cmd.arg("address-model=64");
        cmd.args(&with_libraries);
        cmd.arg(format!("-j{job_count}"));
        cmd.arg("install");
        cmd
    };

    crate::utils::command(&mut { b2_cmd }, "Boost b2 build")?;
    Ok(())
}
