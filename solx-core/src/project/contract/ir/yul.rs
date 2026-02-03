//!
//! The contract Yul source code.
//!

use solx_yul::yul::lexer::Lexer;
use solx_yul::yul::parser::statement::object::Object;

///
/// The contract Yul source code.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Yul {
    /// Yul AST object.
    pub object: Object,
    /// Dependencies of the Yul object.
    pub dependencies: solx_codegen_evm::Dependencies,
    /// Runtime code object that is only set in deploy code.
    pub runtime_code: Option<Box<Self>>,
}

impl Yul {
    ///
    /// Transforms the `solc` standard JSON output contract into a Yul object.
    ///
    pub fn try_from_source(
        path: &str,
        source_code: &str,
        debug_config: Option<&solx_codegen_evm::DebugConfig>,
    ) -> anyhow::Result<Option<Self>> {
        if source_code.is_empty() {
            return Ok(None);
        }

        if let Some(debug_config) = debug_config {
            debug_config.dump_yul(path, source_code)?;
        }

        let mut lexer = Lexer::new(source_code);
        let mut object = Object::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy)
            .map_err(|error| anyhow::anyhow!("Yul parsing: {error:?}"))?;

        let runtime_code = object.inner_object.take().map(|inner_object| {
            let dependencies = inner_object.get_evm_dependencies(None);
            Self {
                object: *inner_object,
                dependencies,
                runtime_code: None,
            }
        });
        let dependencies = object.get_evm_dependencies(
            runtime_code
                .as_ref()
                .map(|runtime_code| &runtime_code.object),
        );

        Ok(Some(Self {
            object,
            dependencies,
            runtime_code: runtime_code.map(Box::new),
        }))
    }
}
