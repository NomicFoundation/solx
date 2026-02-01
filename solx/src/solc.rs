//!
//! `solc` compiler client.
//!

use std::ffi::CStr;
use std::ffi::CString;
use std::path::PathBuf;

///
/// The Solidity compiler.
///
#[derive(Debug)]
pub struct Solc {
    /// The `solc` compiler version.
    version: solx_standard_json::Version,
}

#[link(name = "solc", kind = "static")]
unsafe extern "C" {
    ///
    /// Pass standard JSON input to the Solidity compiler.
    ///
    fn solidity_compile(
        input: *const ::libc::c_char,
        callback: *const ::libc::c_void,
        context: *const ::libc::c_void,
    ) -> *const std::os::raw::c_char;

    ///
    /// Pass standard JSON input to the Solidity compiler.
    ///
    /// Passes `--base-path`, `--include-paths`, and `--allow-paths` just like it is done with the CLI.
    ///
    fn solidity_compile_default_callback(
        input: *const ::libc::c_char,
        base_path: *const ::libc::c_char,
        include_paths_size: u64,
        include_paths: *const *const ::libc::c_char,
        allow_paths_size: u64,
        allow_paths: *const *const ::libc::c_char,
        error_pointer: *mut *mut ::libc::c_char,
    ) -> *const std::os::raw::c_char;

    ///
    /// Get the Solidity compiler version.
    ///
    fn solidity_version_extended() -> *const std::os::raw::c_char;
}

impl Default for Solc {
    fn default() -> Self {
        Self {
            version: Self::parse_version(),
        }
    }
}

impl solx_core::Solc for Solc {
    fn standard_json(
        &self,
        input_json: &mut solx_standard_json::Input,
        use_import_callback: bool,
        base_path: Option<&str>,
        include_paths: &[String],
        mut allow_paths: Option<String>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let original_output_selection = input_json.settings.output_selection.to_owned();
        input_json.settings.output_selection.normalize();
        input_json.settings.output_selection.retain_solc();
        input_json
            .settings
            .output_selection
            .set_selector(solx_standard_json::InputSelector::Metadata);
        input_json
            .settings
            .output_selection
            .set_selector(input_json.settings.via_ir.into());

        let original_optimizer = input_json.settings.optimizer.to_owned();
        input_json.settings.optimizer.mode = None;
        input_json.settings.optimizer.size_fallback = None;

        let input_string = serde_json::to_string(input_json).expect("Always valid");
        let input_c_string = CString::new(input_string).expect("Always valid");

        let base_path = base_path.map(|base_path| CString::new(base_path).expect("Always valid"));
        let base_path = match base_path.as_ref() {
            Some(base_path) => base_path.as_ptr(),
            None => std::ptr::null(),
        };

        let include_paths: Vec<CString> = include_paths
            .iter()
            .map(|path| CString::new(path.as_str()).expect("Always valid"))
            .collect();
        let include_paths: Vec<*const ::libc::c_char> =
            include_paths.iter().map(|path| path.as_ptr()).collect();
        let include_paths_ptr = if include_paths.is_empty() {
            std::ptr::null()
        } else {
            include_paths.as_ptr()
        };

        for path in input_json.sources.keys() {
            let mut path = PathBuf::from(path);
            if path.is_file() {
                path.pop();
            }
            if path.is_dir() {
                if let Some(allow_paths) = allow_paths.as_mut() {
                    allow_paths.push(',');
                    allow_paths.push_str(path.to_str().expect("Always valid"));
                } else {
                    allow_paths = Some(path.to_str().expect("Always valid").to_owned());
                }
            }
        }
        let allow_paths = allow_paths
            .map(|allow_paths| {
                allow_paths
                    .split(',')
                    .map(|path| CString::new(path.to_owned()).expect("Always valid"))
                    .collect::<Vec<CString>>()
            })
            .unwrap_or_default();
        let allow_paths: Vec<*const ::libc::c_char> =
            allow_paths.iter().map(|path| path.as_ptr()).collect();
        let allow_paths_ptr = if allow_paths.is_empty() {
            std::ptr::null()
        } else {
            allow_paths.as_ptr()
        };

        let mut error_message = std::ptr::null_mut();
        let error_pointer = &mut error_message;
        let output_string = unsafe {
            let output_pointer = if use_import_callback {
                solidity_compile_default_callback(
                    input_c_string.as_ptr(),
                    base_path,
                    include_paths.len() as u64,
                    include_paths_ptr,
                    allow_paths.len() as u64,
                    allow_paths_ptr,
                    error_pointer,
                )
            } else {
                solidity_compile(input_c_string.as_ptr(), std::ptr::null(), std::ptr::null())
            };
            if !error_message.is_null() {
                let error_message = CStr::from_ptr(error_message).to_string_lossy().into_owned();
                anyhow::bail!("solc standard JSON I/O: {error_message}");
            }
            CStr::from_ptr(output_pointer)
                .to_string_lossy()
                .into_owned()
        };

        let mut solc_output = match solx_utils::deserialize_from_str::<solx_standard_json::Output>(
            output_string.as_str(),
        ) {
            Ok(solc_output) => solc_output,
            Err(error) => {
                anyhow::bail!("solc standard JSON output parsing: {error:?}");
            }
        };

        input_json.settings.output_selection = original_output_selection;
        input_json.settings.optimizer = original_optimizer;
        solc_output
            .errors
            .retain(|error| match error.error_code.as_deref() {
                Some(code) => {
                    !solx_standard_json::OutputError::IGNORED_WARNING_CODES.contains(&code)
                }
                None => true,
            });

        Ok(solc_output)
    }

    fn validate_yul_paths(
        &self,
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut solc_input = solx_standard_json::Input::from_yul_paths(
            paths,
            libraries,
            solx_standard_json::InputOptimizer::default(),
            &solx_standard_json::InputSelection::default(),
            solx_standard_json::InputMetadata::default(),
            vec![],
        );
        self.validate_yul_standard_json(&mut solc_input)
    }

    fn validate_yul_standard_json(
        &self,
        solc_input: &mut solx_standard_json::Input,
    ) -> anyhow::Result<solx_standard_json::Output> {
        solc_input
            .settings
            .output_selection
            .set_selector(solx_standard_json::InputSelector::Yul);
        let solc_output = self.standard_json(solc_input, true, None, &[], None)?;
        Ok(solc_output)
    }

    fn version(&self) -> &solx_standard_json::Version {
        &self.version
    }
}

impl Solc {
    ///
    /// The `solc` version parser.
    ///
    fn parse_version() -> solx_standard_json::Version {
        let output = unsafe {
            let output_pointer = solidity_version_extended();
            CStr::from_ptr(output_pointer)
                .to_string_lossy()
                .into_owned()
        };

        let lines = output.lines().collect::<Vec<&str>>();

        let long = lines
            .get(1)
            .unwrap_or_else(|| panic!("solc version parsing: missing line 1."))
            .split(' ')
            .nth(1)
            .expect("solc version parsing: missing version.")
            .to_owned();
        let default: semver::Version = long
            .split('+')
            .next()
            .expect("solc version parsing: missing semver.")
            .parse::<semver::Version>()
            .unwrap_or_else(|error| panic!("solc version parsing: {error}."));
        let llvm_revision: semver::Version = lines
            .get(2)
            .expect("LLVM revision parsing: missing line 2.")
            .split(' ')
            .nth(1)
            .expect("LLVM revision parsing: missing version.")
            .split('-')
            .nth(1)
            .expect("LLVM revision parsing: missing `solx` revision.")
            .parse::<semver::Version>()
            .unwrap_or_else(|error| panic!("LLVM revision parsing: {error}."));

        solx_standard_json::Version::new(long, default, llvm_revision)
    }
}
