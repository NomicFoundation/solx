//!
//! MLIR compilation context for EVM code generation.
//!

pub mod environment;
pub mod function;
pub mod modifier;
pub mod pointer;
pub mod try_fallback_kind;
pub mod r#type;
pub mod user_defined_operator;
pub mod value;
pub mod yul_value;

pub use self::user_defined_operator::UserDefinedOperator;

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::BlockLike;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::Type;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::operation::OperationMutLike;
use melior::pass::PassManager;
use slang_solidity_v2::ast::NodeId;

use crate::llvm_module::RawLlvmModule;

use self::function::Function;

/// Accumulated MLIR state threaded through the AST visitors.
pub struct Context<'context> {
    /// The MLIR module being built.
    pub module: Module<'context>,
    /// The MLIR context with all dialects and translations registered.
    mlir_context: &'context melior::Context,
    /// Cached unknown source location.
    unknown_location: Location<'context>,
    /// Resolution metadata keyed by the AST definition id of each function.
    pub function_signatures: HashMap<NodeId, Function<'context>>,
    /// MLIR type of the contract being emitted (types `this`); set by the frontend before bodies.
    pub current_contract_type: Option<Type<'context>>,
    /// User-defined operator bindings, keyed by `(udvt_definition_id, operator)` → bound function id.
    pub operator_bindings: HashMap<(NodeId, UserDefinedOperator), NodeId>,
    /// Cross-contract references in encounter order, drained into the linker output.
    pub dependencies: RefCell<Vec<String>>,
    /// Monotonic internal-function-pointer dispatch tag; starts at 1 (0 is the null pointer).
    function_id_counter: Cell<i64>,
}

impl<'context> Context<'context> {
    /// MLIR `builtin.module` operation name used to locate nested modules.
    const BUILTIN_MODULE: &'static str = "builtin.module";

    /// Creates a `melior::Context` with all upstream, Sol, and Yul dialects plus LLVM translations registered.
    pub fn create_mlir_context() -> melior::Context {
        let registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&registry);

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

        // Register Sol dialect passes once.
        static REGISTER_PASSES: Once = Once::new();
        REGISTER_PASSES.call_once(|| unsafe {
            crate::ffi::mlirRegisterSolPasses();
        });

        context
    }

    /// Creates a new MLIR state with an empty module carrying the EVM-version, data-layout, and target-triple attributes.
    pub fn new(context: &'context melior::Context, evm_version: solx_utils::EVMVersion) -> Self {
        let unknown_location = Location::unknown(context);
        let module = Module::new(unknown_location);

        let evm_version_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateEvmVersionAttr(
                context.to_raw(),
                evm_version.into_sol_dialect_identifier(),
            ))
        };
        unsafe {
            mlir_sys::mlirOperationSetAttributeByName(
                module.as_operation().to_raw(),
                mlir_sys::mlirStringRefCreateFromCString(c"sol.evm_version".as_ptr()),
                evm_version_attribute.to_raw(),
            );
        }

        let target = solx_utils::Target::EVM;
        let data_layout_attr: Attribute<'_> =
            StringAttribute::new(context, target.data_layout()).into();
        let target_triple_attr: Attribute<'_> =
            StringAttribute::new(context, target.triple()).into();
        unsafe {
            mlir_sys::mlirOperationSetAttributeByName(
                module.as_operation().to_raw(),
                mlir_sys::mlirStringRefCreateFromCString(c"llvm.data_layout".as_ptr()),
                data_layout_attr.to_raw(),
            );
            mlir_sys::mlirOperationSetAttributeByName(
                module.as_operation().to_raw(),
                mlir_sys::mlirStringRefCreateFromCString(c"llvm.target_triple".as_ptr()),
                target_triple_attr.to_raw(),
            );
        }

        Self {
            module,
            function_signatures: HashMap::new(),
            mlir_context: context,
            unknown_location,
            current_contract_type: None,
            operator_bindings: HashMap::new(),
            dependencies: RefCell::new(Vec::new()),
            function_id_counter: Cell::new(1),
        }
    }

    /// The MLIR context with all dialects and translations registered.
    pub fn mlir(&self) -> &'context melior::Context {
        self.mlir_context
    }

    /// The cached unknown source location.
    pub fn location(&self) -> Location<'context> {
        self.unknown_location
    }

    /// Allocates the next internal-function-pointer dispatch tag.
    pub fn next_function_id(&self) -> i64 {
        let id = self.function_id_counter.get();
        self.function_id_counter.set(id + 1);
        id
    }

    /// Records a cross-contract reference (object name); duplicates ignored.
    pub fn add_dependency(&self, name: String) {
        let mut dependencies = self.dependencies.borrow_mut();
        if !dependencies.iter().any(|existing| existing == &name) {
            dependencies.push(name);
        }
    }

    /// Registers a function signature keyed by its AST definition id.
    pub fn register_function_signature(
        &mut self,
        definition_id: NodeId,
        mlir_name: String,
        parameter_types: Vec<Type<'context>>,
        return_types: Vec<Type<'context>>,
    ) {
        self.function_signatures.insert(
            definition_id,
            Function::new(mlir_name, parameter_types, return_types),
        );
    }

    /// Resolves a registered function by definition id (panics if unregistered — an internal invariant).
    pub fn resolve_function(&self, definition_id: NodeId) -> &Function<'context> {
        self.function_signatures
            .get(&definition_id)
            .unwrap_or_else(|| panic!("undefined function for definition {definition_id:?}"))
    }

    /// Runs the Sol-to-LLVM conversion pass pipeline on `module` in place:
    /// canonicalize, modifier-op lowering, sol→yul, yul→std, canonicalize,
    /// scf→cf, func→llvm, arith→llvm, cf→llvm, reconcile-unrealized-casts.
    pub fn run_sol_passes(context: &melior::Context, module: &mut Module) -> anyhow::Result<()> {
        let pass_manager = PassManager::new(context);
        pass_manager.enable_verifier(true);

        unsafe {
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsCanonicalizer(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateSolModifierOpLoweringPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertSolToYulPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertYulToStandardPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsCanonicalizer(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionSCFToControlFlowPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertFuncToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionArithToLLVMConversionPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertControlFlowToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionReconcileUnrealizedCastsPass(),
            ));
        }

        pass_manager
            .run(module)
            .map_err(|error| anyhow::anyhow!("Sol pass pipeline failed: {error}"))
    }

    /// Runs the Sol-to-LLVM pipeline and splits the nested deploy/runtime modules
    /// into separate LLVM-dialect strings (the inner module matched by
    /// `runtime_code_identifier` is detached and stringified on its own).
    pub fn finalize_module(
        self,
        runtime_code_identifier: &str,
        capture_sol: bool,
    ) -> anyhow::Result<crate::output::MlirOutput> {
        let mut module = self.module;

        let sol_source = capture_sol.then(|| module.as_operation().to_string());

        Self::run_sol_passes(self.mlir_context, &mut module)?;

        // Detach the runtime module so the deploy text doesn't duplicate it; the
        // deploy entry still references it via `evm.datasize`/`evm.dataoffset`.
        let llvm_runtime_source =
            Self::take_nested_module_text(&mut module, runtime_code_identifier)?;
        let llvm_deploy_source = module.as_operation().to_string();

        Ok(crate::output::MlirOutput {
            sol_source,
            llvm_deploy_source,
            llvm_runtime_source,
            dependencies: self.dependencies.into_inner(),
        })
    }

    /// Finds a nested `builtin.module` in `module`'s body whose `sym_name`
    /// matches `target`, removes it from the parent, and returns its
    /// textual form.
    fn take_nested_module_text(module: &mut Module, target: &str) -> anyhow::Result<String> {
        let body = module.body();
        std::iter::successors(body.first_operation_mut(), |operation| {
            operation.next_in_block_mut()
        })
        .find_map(|mut operation| {
            if operation
                .name()
                .as_string_ref()
                .as_str()
                .expect("an MLIR operation name is valid UTF-8")
                != Self::BUILTIN_MODULE
            {
                return None;
            }
            let symbol = operation.attribute("sym_name").ok()?;
            let symbol_name: StringAttribute = symbol.try_into().ok()?;
            if symbol_name.value() != target {
                return None;
            }
            let text = operation.to_string();
            operation.remove_from_parent();
            Some(text)
        })
        .ok_or_else(|| anyhow::anyhow!("no module with sym_name `{target}` in Sol pass output"))
    }

    /// Parses, verifies, and translates LLVM-dialect MLIR text to a raw LLVM module.
    ///
    /// `immutables` lowers `llvm.setimmutable` (a library's library-address
    /// immutable, which has no LLVM-IR translation) to heap stores at the given
    /// offsets before translation; `None` leaves the module unchanged.
    pub fn translate_source_to_llvm(
        context: &melior::Context,
        source: &str,
        immutables: Option<&std::collections::BTreeMap<String, std::collections::BTreeSet<u64>>>,
    ) -> anyhow::Result<RawLlvmModule> {
        let module = Module::parse(context, source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse MLIR source text"))?;

        if !module.as_operation().verify() {
            anyhow::bail!("MLIR module verification failed");
        }

        if let Some(immutables) = immutables {
            Self::lower_set_immutables(&module, immutables);
        }

        unsafe {
            let raw_operation = module.as_operation().to_raw();
            let llvm_context = inkwell::llvm_sys::core::LLVMContextCreate();

            let llvm_module =
                mlir_sys::mlirTranslateModuleToLLVMIR(raw_operation, llvm_context as *mut _);

            if llvm_module.is_null() {
                inkwell::llvm_sys::core::LLVMContextDispose(llvm_context);
                anyhow::bail!(
                    "mlirTranslateModuleToLLVMIR returned null — \
                     ensure register_all_llvm_translations was called"
                );
            }

            Ok(RawLlvmModule {
                context: llvm_context,
                module: llvm_module as *mut _,
            })
        }
    }

    /// Lowers every `llvm.setimmutable` to heap stores at its reserved offsets via the solx-llvm C-API.
    fn lower_set_immutables(
        module: &Module,
        immutables: &std::collections::BTreeMap<String, std::collections::BTreeSet<u64>>,
    ) {
        let mut id_cstrings: Vec<std::ffi::CString> = Vec::new();
        let mut offsets: Vec<u64> = Vec::new();
        for (id, id_offsets) in immutables {
            for &offset in id_offsets {
                id_cstrings.push(
                    std::ffi::CString::new(id.as_str()).expect("immutable id has no interior NUL"),
                );
                offsets.push(offset);
            }
        }
        let id_pointers: Vec<*const std::ffi::c_char> =
            id_cstrings.iter().map(|id| id.as_ptr()).collect();
        unsafe {
            crate::ffi::mlirEvmLowerSetImmutables(
                module.to_raw(),
                id_pointers.as_ptr(),
                offsets.as_ptr(),
                offsets.len() as u64,
            );
        }
    }
}
