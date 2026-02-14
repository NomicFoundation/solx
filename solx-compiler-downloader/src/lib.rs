//!
//! The compiler downloader config.
//!

pub(crate) mod config;

use self::config::Config;
use self::config::compiler_list::CompilerList;
use self::config::executable::protocol::Protocol;

#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;

use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use colored::Colorize;
use reqwest::Url;
use reqwest::blocking::Client as HttpClient;

///
/// The compiler downloader.
///
#[derive(Debug)]
pub struct Downloader {
    /// The HTTP client.
    http_client: HttpClient,
    /// The compiler-bin JSON list metadata.
    compiler_list: Option<CompilerList>,
}

impl Default for Downloader {
    fn default() -> Self {
        let http_client = HttpClient::builder()
            .connect_timeout(Duration::from_secs(600))
            .timeout(Duration::from_secs(600))
            .pool_idle_timeout(Duration::from_secs(600))
            .pool_max_idle_per_host(0)
            .tcp_keepalive(Duration::from_secs(600))
            .build()
            .expect("Always valid");
        Self::new(http_client)
    }
}

impl Downloader {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(http_client: HttpClient) -> Self {
        Self {
            http_client,
            compiler_list: None,
        }
    }

    ///
    /// Downloads the compilers described in the config.
    ///
    pub fn download(mut self, config_path: &Path) -> anyhow::Result<Config> {
        let config_file = std::fs::File::open(config_path).map_err(|error| {
            anyhow::anyhow!("Executable downloader config {config_path:?} opening error: {error}")
        })?;
        let config_reader = std::io::BufReader::new(config_file);
        let config: Config = serde_json::from_reader(config_reader).map_err(|error| {
            anyhow::anyhow!("Executable downloader config {config_path:?} parsing error: {error}")
        })?;

        for executable in config.executables.iter() {
            if !executable.is_enabled {
                continue;
            }

            let platform_directory = executable.get_remote_platform_directory()?;

            let mut source_path = executable
                .source
                .replace("${PLATFORM}", platform_directory.as_str());

            let destination_path = PathBuf::from_str(
                format!("{}{}", executable.destination, std::env::consts::EXE_SUFFIX).as_str(),
            )
            .map_err(|_| {
                anyhow::anyhow!(
                    "Executable `{}` destination is invalid",
                    executable.destination
                )
            })?;

            match executable.protocol {
                Protocol::File => {
                    source_path += std::env::consts::EXE_SUFFIX;
                    if source_path == destination_path.to_string_lossy() {
                        println!(
                            "    {} executable {destination_path:?}. The source and destination are the same.",
                            "Skipping".bright_green().bold(),
                        );
                        continue;
                    }

                    println!(
                        "     {} executable `{source_path}` => {destination_path:?}",
                        "Copying".bright_green().bold(),
                    );

                    std::fs::copy(source_path.as_str(), executable.destination.as_str()).map_err(
                        |error| {
                            anyhow::anyhow!("Executable {source_path:?} copying error: {error}",)
                        },
                    )?;
                    continue;
                }
                Protocol::HTTPS => {
                    source_path += std::env::consts::EXE_SUFFIX;

                    if destination_path.exists() {
                        println!(
                            "    {} executable {destination_path:?}. Already exists.",
                            "Skipping".bright_green().bold(),
                        );
                        continue;
                    }

                    println!(
                        " {} executable {source_path:?} => {destination_path:?}",
                        "Downloading".bright_green().bold(),
                    );
                    self.download_file(source_path.as_str(), destination_path)?;
                }
                Protocol::CompilerBinList => {
                    if destination_path.exists() {
                        println!(
                            "    {} executable {destination_path:?}. Already exists.",
                            "Skipping".bright_green().bold(),
                        );
                        continue;
                    }

                    let compiler_list_path = PathBuf::from(source_path.as_str());
                    let compiler_list = self.compiler_list.get_or_insert_with(|| {
                        CompilerList::try_from(compiler_list_path.to_str().expect("Always valid"))
                            .expect("compiler-bin JSON list downloading error")
                    });
                    if compiler_list.releases.is_empty() {
                        return Ok(config);
                    }

                    let version = executable.version.as_deref().ok_or_else(|| {
                        anyhow::anyhow!(
                            "Version is not specified for the compiler-bin-list protocol"
                        )
                    })?;
                    let source_executable_name = match compiler_list.releases.get(version) {
                        Some(source_executable_name) => source_executable_name,
                        None => anyhow::bail!(
                            "Executable for version v{version} not found in the compiler JSON list",
                        ),
                    };
                    #[cfg(target_os = "windows")]
                    if !source_executable_name.ends_with(std::env::consts::EXE_SUFFIX) {
                        println!(
                            "    {} downloading {source_executable_name:?}. Not an executable file.",
                            "Skipping".bright_green().bold(),
                        );
                        continue;
                    }
                    let mut source_path = compiler_list_path;
                    source_path.pop();
                    source_path.push(source_executable_name);

                    println!(
                        " {} executable {source_path:?} => {destination_path:?}",
                        "Downloading".bright_green().bold(),
                    );
                    self.download_file(
                        source_path.to_str().expect("Always valid"),
                        destination_path,
                    )?;
                }
            }
        }

        Ok(config)
    }

    ///
    /// Downloads a file from a URL to a destination path, with resume support and retries.
    ///
    fn download_file(&self, url: &str, destination: impl AsRef<Path>) -> anyhow::Result<()> {
        std::fs::create_dir_all(destination.as_ref().parent().expect("Always exists"))?;

        let destination = destination.as_ref();
        let tmp_destination = destination.with_file_name(format!(
            "{}.part",
            destination
                .file_name()
                .and_then(|name| name.to_str())
                .expect("Always exists")
        ));

        let max_attempts = 8;
        let mut backoff = Duration::from_millis(200);

        for attempt in 1..=max_attempts {
            let mut file = OpenOptions::new()
                .create(true)
                .read(true)
                .append(true)
                .open(&tmp_destination)?;

            let url = Url::from_str(url)
                .map_err(|error| anyhow::anyhow!("URL `{url}` parsing error: {error}"))?;
            let mut request = self.http_client.get(url);
            let downloaded_bytes = file.metadata()?.len();
            if downloaded_bytes > 0 {
                request = request.header("Range", format!("bytes={downloaded_bytes}-"));
            }

            let mut response = match request.send() {
                Ok(response) => response,
                Err(error) => {
                    if attempt < max_attempts {
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(Duration::from_secs(2));
                        continue;
                    }
                    return Err(anyhow::anyhow!(error));
                }
            };

            let status = response.status().as_u16();

            if status == 429 || (500..600).contains(&status) {
                if attempt < max_attempts {
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(Duration::from_secs(2));
                    continue;
                }
                return Err(anyhow::anyhow!("HTTP {}", response.status()));
            }

            if downloaded_bytes > 0 && status == 200 {
                drop(file);
                let _ = std::fs::remove_file(&tmp_destination);
                backoff = Duration::from_millis(200);
                continue;
            }

            if !(200..300).contains(&status) {
                return Err(anyhow::anyhow!("HTTP {}", response.status()));
            }

            file.seek(SeekFrom::End(0))?;

            if let Err(error) = response.copy_to(&mut file) {
                if attempt < max_attempts {
                    std::thread::sleep(backoff);
                    backoff = (backoff * 2).min(Duration::from_secs(2));
                    continue;
                }
                return Err(anyhow::anyhow!(error));
            }

            if destination.exists() {
                std::fs::remove_file(destination)?;
            }
            std::fs::rename(tmp_destination, destination)?;

            #[cfg(target_family = "unix")]
            std::fs::set_permissions(destination, std::fs::Permissions::from_mode(0o755))?;

            return Ok(());
        }

        anyhow::bail!("Downloading failed after {max_attempts} attempts");
    }
}
