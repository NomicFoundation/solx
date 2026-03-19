//!
//! MLIR module builder for EVM code generation.
//!
//! Provides the [`Context`] type that accumulates MLIR operations into a
//! module. Frontends (e.g. `solx-slang`) use this to emit LLVM dialect
//! operations without dealing with raw `melior` API details.
//!

pub(crate) mod constant;
pub(crate) mod function_signature;
pub(crate) mod llvm;
pub(crate) mod passes;
pub(crate) mod sol;
pub(crate) mod translation;

use std::collections::HashMap;
use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::Type;
use melior::ir::r#type::IntegerType;

use solx_utils::AddressSpace;

use self::function_signature::FunctionSignature;

/// Accumulated MLIR state threaded through the AST visitors.
///
/// Owns a `melior::ir::Module` being populated and provides helpers for
/// creating common MLIR types, SSA naming, and function registration.
/// Also provides pass pipeline execution and LLVM translation.
///
/// Mirrors the single-context pattern used by `solx-codegen-evm`.
pub struct Context<'context> {
    /// The MLIR context with all dialects and translations registered.
    context: &'context melior::Context,
    /// The MLIR module being built.
    module: Module<'context>,
    /// Cached `i256` type (MLIR interns types, but avoids repeated lookups).
    i256_type: Type<'context>,
    /// Cached `i1` type.
    i1_type: Type<'context>,
    /// Cached unknown source location.
    unknown_location: Location<'context>,
    /// State variable name -> storage slot mapping.
    /// TODO: get storage slots from slang-solidity instead of re-deriving them here
    state_variables: HashMap<String, u64>,
    /// All function signatures for call resolution (bare name -> overloads).
    function_signatures: HashMap<String, Vec<FunctionSignature>>,
    /// Cached `!sol.ptr<i256, Stack>` type for alloca operations.
    sol_ptr_type: Type<'context>,
}

impl<'context> Context<'context> {
    // ---- Private constants ----

    /// Bit width for ICmp predicate attributes.
    const PREDICATE_ATTRIBUTE_BIT_WIDTH: u32 = 64;

    /// Bit width of each limb for wide constant decomposition.
    const LIMB_BIT_WIDTH: i64 = solx_utils::BIT_LENGTH_X32 as i64;

    /// Bit width of a Solidity function selector (4 bytes).
    const SELECTOR_BIT_WIDTH: u32 = 32;

    // ---- Public static ----

    /// Creates a fully-initialized `melior::Context` with all upstream
    /// dialects, Sol dialect, Yul dialect, and LLVM translation interfaces
    /// registered.
    ///
    /// `register_all_llvm_translations` MUST be called before any
    /// MLIR-to-LLVM translation. Without it, `mlirTranslateModuleToLLVMIR`
    /// returns null. This function enforces that invariant.
    pub fn create_mlir_context() -> melior::Context {
        let registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&registry);

        // SAFETY: FFI calls to register Sol and Yul dialects into the
        // registry. The registry and dialect handles are valid C objects
        // produced by the MLIR C API; no aliasing or lifetime issues.
        unsafe {
            crate::ffi::mlirDialectHandleInsertDialect(
                crate::ffi::mlirGetDialectHandle__sol__(),
                registry.to_raw(),
            );
            crate::ffi::mlirDialectHandleInsertDialect(
                crate::ffi::mlirGetDialectHandle__yul__(),
                registry.to_raw(),
            );
        }

        let context = melior::Context::new();
        context.append_dialect_registry(&registry);
        context.load_all_available_dialects();
        melior::utility::register_all_llvm_translations(&context);

        // Register Sol dialect passes so they can be added to a PassManager.
        // Only call once — double-registration may crash.
        static REGISTER_PASSES: Once = Once::new();
        // SAFETY: `mlirRegisterSolPasses` is idempotent within a single
        // call but must not be called concurrently. `Once` guarantees
        // single-threaded, one-time execution.
        REGISTER_PASSES.call_once(|| unsafe {
            crate::ffi::mlirRegisterSolPasses();
        });

        context
    }

    // ---- Constructor ----

    /// Maps `solx_utils::EVMVersion` to the Sol dialect `EvmVersionAttr` `u32`
    /// encoding expected by the C++ backend.
    fn sol_dialect_evm_version(version: solx_utils::EVMVersion) -> u32 {
        match version {
            solx_utils::EVMVersion::Cancun => 11,
            solx_utils::EVMVersion::Prague => 12,
            solx_utils::EVMVersion::Osaka => 13,
        }
    }

    /// Creates a new MLIR state with an empty module.
    ///
    /// Sets the `sol.evm_version` module attribute required by the
    /// `convert-sol-to-std` pass.
    pub fn new(context: &'context melior::Context, evm_version: solx_utils::EVMVersion) -> Self {
        let location = Location::unknown(context);
        let module = Module::new(location);

        // Set the EVM version attribute on the module — required by the
        // Sol-to-standard conversion pass.
        // SAFETY: `solxCreateEvmVersionAttr` returns a valid MlirAttribute
        // from the C++ Sol dialect. The context pointer is valid.
        let evm_version_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateEvmVersionAttr(
                context.to_raw(),
                Self::sol_dialect_evm_version(evm_version),
            ))
        };
        // SAFETY: Setting a named attribute on the module operation. Both
        // the operation and attribute are valid MLIR objects owned by this
        // context.
        unsafe {
            mlir_sys::mlirOperationSetAttributeByName(
                module.as_operation().to_raw(),
                mlir_sys::mlirStringRefCreateFromCString(c"sol.evm_version".as_ptr()),
                evm_version_attribute.to_raw(),
            );
        }

        Self {
            context,
            module,
            i256_type: IntegerType::new(context, solx_utils::BIT_LENGTH_FIELD as u32).into(),
            i1_type: IntegerType::new(context, 1).into(),
            unknown_location: location,
            state_variables: HashMap::new(),
            function_signatures: HashMap::new(),
            sol_ptr_type: Type::parse(context, "!sol.ptr<i256, Stack>")
                .expect("valid sol.ptr type syntax"),
        }
    }

    // ---- Public self (consuming) ----

    /// Consumes the builder and returns the accumulated MLIR module.
    pub fn into_module(self) -> Module<'context> {
        self.module
    }

    // ---- Public &mut self ----

    /// Registers a state variable with its storage slot.
    pub fn register_state_variable(&mut self, name: String, slot: u64) {
        self.state_variables.insert(name, slot);
    }

    /// Registers a function signature for call resolution.
    pub fn register_function_signature(
        &mut self,
        bare_name: &str,
        mlir_name: String,
        parameter_count: usize,
        return_count: usize,
    ) {
        self.function_signatures
            .entry(bare_name.to_owned())
            .or_default()
            .push(FunctionSignature::new(
                mlir_name,
                parameter_count,
                return_count,
            ));
    }

    /// Returns a mutable reference to the underlying MLIR module.
    pub fn module_mut(&mut self) -> &mut Module<'context> {
        &mut self.module
    }

    // ---- Public &self ----

    /// Returns a reference to the melior context.
    pub fn context(&self) -> &'context melior::Context {
        self.context
    }

    /// Returns the module body block for appending top-level operations.
    pub fn body(&self) -> BlockRef<'context, '_> {
        self.module.body()
    }

    /// Returns an unknown source location.
    pub fn location(&self) -> Location<'context> {
        self.unknown_location
    }

    /// Returns the EVM word type (`i256`).
    pub fn i256(&self) -> Type<'context> {
        self.i256_type
    }

    /// Returns the EVM boolean type (`i1`).
    pub fn i1(&self) -> Type<'context> {
        self.i1_type
    }

    /// Returns an LLVM pointer type with the given address space.
    pub fn pointer(&self, address_space: AddressSpace) -> Type<'context> {
        melior::dialect::llvm::r#type::pointer(self.context, address_space as u32)
    }

    /// Returns the storage slot for a state variable, if it exists.
    ///
    /// # Returns None
    ///
    /// Returns `None` if no state variable with the given name has been registered.
    pub fn state_variable_slot(&self, name: &str) -> Option<u64> {
        self.state_variables.get(name).copied()
    }

    /// Resolves a function call by bare name and argument count.
    ///
    /// Returns the mangled MLIR name and the number of return values.
    ///
    /// # Errors
    ///
    /// Returns an error if the function is undefined or the call is ambiguous.
    pub fn resolve_function(
        &self,
        bare_name: &str,
        argument_count: usize,
    ) -> anyhow::Result<(&str, usize)> {
        let signatures = self
            .function_signatures
            .get(bare_name)
            .ok_or_else(|| anyhow::anyhow!("undefined function: {bare_name}"))?;
        let matches: Vec<_> = signatures
            .iter()
            .filter(|s| s.parameter_count() == argument_count)
            .collect();
        match matches.len() {
            0 => anyhow::bail!("no overload of '{bare_name}' takes {argument_count} arguments"),
            1 => Ok((matches[0].mlir_name(), matches[0].return_count())),
            _ => {
                let overloads: Vec<&str> = matches.iter().map(|s| s.mlir_name()).collect();
                anyhow::bail!(
                    "ambiguous call to '{bare_name}' with {argument_count} arguments: {}",
                    overloads.join(", ")
                )
            }
        }
    }
}
