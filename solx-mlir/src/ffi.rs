//!
//! FFI bindings for Sol and Yul dialect C API functions.
//!
//! These functions are provided by the `libMLIRCAPISol`, `libMLIRSolTransforms`,
//! `libMLIRSolToStandard`, and `libMLIRCAPIYul` static libraries built from
//! solx-llvm.
//!

use mlir_sys::MlirBlock;
use mlir_sys::MlirContext;
use mlir_sys::MlirDialectHandle;
use mlir_sys::MlirDialectRegistry;
use mlir_sys::MlirPass;
use mlir_sys::MlirRegion;

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

    // ---- Canonicalization ----

    /// Creates the `canonicalize` pass.
    pub fn mlirCreateTransformsCanonicalizer() -> MlirPass;

    /// Creates the `sol-modifier-op-lowering` pass.
    pub fn mlirCreateSolModifierOpLoweringPass() -> MlirPass;

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
    pub fn mlirDialectHandleInsertDialect(handle: MlirDialectHandle, registry: MlirDialectRegistry);

    // ---- Sol attribute constructors (from sol_attr_stubs.cpp) ----

    /// Creates a `ContractKindAttr` (0=Interface, 1=Contract, 2=Library).
    pub fn solxCreateContractKindAttr(context: MlirContext, kind: u32) -> mlir_sys::MlirAttribute;

    /// Creates a `StateMutabilityAttr` (0=Pure, 1=View, 2=NonPayable, 3=Payable).
    pub fn solxCreateStateMutabilityAttr(
        context: MlirContext,
        mutability: u32,
    ) -> mlir_sys::MlirAttribute;

    /// Creates a `FunctionKindAttr` (0=Constructor, 1=Fallback, 2=Receive).
    pub fn solxCreateFunctionKindAttr(context: MlirContext, kind: u32) -> mlir_sys::MlirAttribute;

    /// Creates an `EvmVersionAttr`.
    pub fn solxCreateEvmVersionAttr(context: MlirContext, version: u32) -> mlir_sys::MlirAttribute;

    // ---- Sol type constructors (from sol_attr_stubs.cpp) ----

    /// Creates a `sol::PointerType` with the given element type and data location.
    ///
    /// `data_location` maps to `mlir::sol::DataLocation` (0=Storage, 1=CallData,
    /// 2=Memory, 3=Stack, 4=Immutable, 5=Transient).
    pub fn solxCreatePointerType(
        context: MlirContext,
        element_type: mlir_sys::MlirType,
        data_location: u32,
    ) -> mlir_sys::MlirType;

    /// Creates a `sol::AddressType` with the given payability.
    pub fn solxCreateAddressType(context: MlirContext, payable: bool) -> mlir_sys::MlirType;

    /// Creates a `sol::ContractType` for a contract with the given name and payability.
    pub fn solxCreateContractType(
        context: MlirContext,
        name_ptr: *const std::ffi::c_char,
        name_len: usize,
        payable: bool,
    ) -> mlir_sys::MlirType;

    /// Creates a `sol::StringType` with the given data location.
    ///
    /// `data_location` maps to `mlir::sol::DataLocation` (0=Storage, 1=CallData,
    /// 2=Memory, 3=Stack, 4=Immutable, 5=Transient).
    pub fn solxCreateStringType(context: MlirContext, data_location: u32) -> mlir_sys::MlirType;

    /// Creates a `sol::FixedBytesType` of the given byte width.
    pub fn solxCreateFixedBytesType(context: MlirContext, size: u32) -> mlir_sys::MlirType;

    /// Creates a `sol::ArrayType` with the given size, element type, and data
    /// location. `size = -1` denotes a dynamic array.
    pub fn solxCreateArrayType(
        context: MlirContext,
        size: i64,
        element_type: mlir_sys::MlirType,
        data_location: u32,
    ) -> mlir_sys::MlirType;

    /// Creates a `sol::MappingType` with the given key and value types.
    pub fn solxCreateMappingType(
        context: MlirContext,
        key_type: mlir_sys::MlirType,
        value_type: mlir_sys::MlirType,
    ) -> mlir_sys::MlirType;

    /// Creates a `sol::StructType` from a slice of member types and a data location.
    pub fn solxCreateStructType(
        context: MlirContext,
        member_types: *const mlir_sys::MlirType,
        member_count: usize,
        data_location: u32,
    ) -> mlir_sys::MlirType;

    // ---- MLIR core (not in mlir-sys) ----

    /// Returns the region that owns the given block.
    pub fn mlirBlockGetParentRegion(block: MlirBlock) -> MlirRegion;
}

/// Returns the parent region of a block as a `RegionRef`.
///
/// # Safety
///
/// The block must be attached to a region (i.e., not detached).
pub fn block_parent_region<'context, 'block>(
    block: &melior::ir::BlockRef<'context, 'block>,
) -> melior::ir::RegionRef<'context, 'block> {
    // SAFETY: The block is attached (guaranteed by melior's ownership model).
    // `mlirBlockGetParentRegion` returns a non-owning handle to the parent.
    unsafe {
        melior::ir::RegionRef::from_raw(mlirBlockGetParentRegion(melior::ir::BlockLike::to_raw(
            block,
        )))
    }
}
