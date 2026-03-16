//!
//! FFI bindings for Sol and Yul dialect C API functions.
//!
//! These functions are provided by the `libMLIRCAPISol`, `libMLIRSolTransforms`,
//! `libMLIRSolToStandard`, and `libMLIRCAPIYul` static libraries built from
//! solx-llvm.
//!

use mlir_sys::MlirContext;
use mlir_sys::MlirDialectHandle;
use mlir_sys::MlirPass;

unsafe extern "C" {
    // ---- Sol dialect registration ----

    /// Returns the dialect handle for the Sol dialect.
    pub fn mlirGetDialectHandle__sol__() -> MlirDialectHandle;

    // ---- Yul dialect registration ----

    /// Returns the dialect handle for the Yul dialect.
    pub fn mlirGetDialectHandle__yul__() -> MlirDialectHandle;

    // ---- Sol dialect passes ----

    /// Registers all Sol dialect passes.
    pub fn mlirRegisterSolPasses();

    /// Creates the Sol modifier op lowering pass.
    pub fn mlirCreateSolModifierOpLoweringPass() -> MlirPass;

    /// Creates the Sol loop-invariant code motion pass.
    pub fn mlirCreateSolLoopInvariantCodeMotionPass() -> MlirPass;

    // ---- Sol-to-Standard conversion ----

    /// Creates the `convert-sol-to-std` pass.
    pub fn mlirCreateConversionConvertSolToStandardPass() -> MlirPass;

    // ---- Standard-to-LLVM conversion passes ----

    /// Creates the `convert-func-to-llvm` pass.
    pub fn mlirCreateConversionConvertFuncToLLVMPass() -> MlirPass;

    /// Creates the `convert-scf-to-cf` pass.
    pub fn mlirCreateConversionSCFToControlFlowPass() -> MlirPass;

    /// Creates the `convert-cf-to-llvm` pass.
    pub fn mlirCreateConversionConvertControlFlowToLLVMPass() -> MlirPass;

    /// Creates the `convert-arith-to-llvm` pass.
    pub fn mlirCreateConversionArithToLLVMConversionPass() -> MlirPass;

    /// Creates the `reconcile-unrealized-casts` pass.
    pub fn mlirCreateConversionReconcileUnrealizedCastsPass() -> MlirPass;

    // ---- Dialect loading ----

    /// Loads a dialect into the context by handle.
    pub fn mlirDialectHandleInsertDialect(
        handle: MlirDialectHandle,
        registry: mlir_sys::MlirDialectRegistry,
    );

    /// Loads a dialect into the context directly.
    pub fn mlirDialectHandleLoadDialect(handle: MlirDialectHandle, ctx: MlirContext);

    // ---- Sol attribute constructors (from sol_attr_stubs.cpp) ----

    /// Creates a `ContractKindAttr` (0=Interface, 1=Contract, 2=Library).
    pub fn solxCreateContractKindAttr(ctx: MlirContext, kind: u32) -> mlir_sys::MlirAttribute;

    /// Creates a `StateMutabilityAttr` (0=Pure, 1=View, 2=NonPayable, 3=Payable).
    pub fn solxCreateStateMutabilityAttr(
        ctx: MlirContext,
        mutability: u32,
    ) -> mlir_sys::MlirAttribute;

    /// Creates an `EvmVersionAttr`.
    pub fn solxCreateEvmVersionAttr(ctx: MlirContext, version: u32) -> mlir_sys::MlirAttribute;
}
