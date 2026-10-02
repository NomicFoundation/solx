//!
//! The debug-info compile unit every subprogram of a module points at.
//!
//! melior wraps none of the debug-info attributes, so the compile unit and its file are built
//! through the mlir-sys raw bindings, and a subprogram through `solxFuseSubprogram`.
//!

use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::Location;
use melior::ir::attribute::StringAttribute;

use crate::FunctionOrigin;

/// The debug-info compile unit a module's subprograms point at, built once per module.
///
/// The compile unit is fused onto the module's own location, which is where MLIR looks for it and
/// where the Sol-to-Yul lowering finds it for the functions it generates. Fusing it onto the
/// contract too carries it into the runtime module, with nothing to copy.
#[derive(Clone, Copy)]
pub struct DebugInfoCompileUnit<'context> {
    /// The distinct `DICompileUnitAttr` every subprogram in this module points at.
    attribute: Attribute<'context>,
}

impl<'context> DebugInfoCompileUnit<'context> {
    /// The dialect of the debug-info attributes, which attribute construction does not load.
    const DIALECT: &'static str = "llvm";
    /// `DW_LANG_Assembly` in LLVM's `Dwarf.def`: DWARF has no language code for Solidity.
    const DW_LANG_ASSEMBLY: u32 = 0x0031;

    /// Builds the compile unit for a module whose source is named `file_name`. The file's
    /// directory is empty, so the debug info does not depend on the working directory.
    pub fn new(melior: &'context melior::Context, file_name: &str) -> Self {
        melior.get_or_load_dialect(Self::DIALECT);
        let attribute = unsafe {
            let file = mlir_sys::mlirLLVMDIFileAttrGet(
                melior.to_raw(),
                StringAttribute::new(melior, file_name).to_raw(),
                StringAttribute::new(melior, "").to_raw(),
            );
            Attribute::from_raw(mlir_sys::mlirLLVMDICompileUnitAttrGet(
                melior.to_raw(),
                mlir_sys::mlirDisctinctAttrCreate(Attribute::unit(melior).to_raw()),
                Self::DW_LANG_ASSEMBLY,
                file,
                StringAttribute::new(melior, "").to_raw(),
                true,
                mlir_sys::MlirLLVMDIEmissionKind_MlirLLVMDIEmissionKindFull,
                mlir_sys::MlirLLVMDINameTableKind_MlirLLVMDINameTableKindDefault,
            ))
        };
        Self { attribute }
    }

    /// `location` with the compile unit fused onto it, for the module and the `sol.contract`.
    ///
    /// Unlike a subprogram, a fused compile unit does not replace the enclosing scope during
    /// translation, so it is safe on a location the Sol-to-Yul lowering copies onto other
    /// operations.
    pub fn fuse_compile_unit(
        self,
        melior: &'context melior::Context,
        location: Location<'context>,
    ) -> Location<'context> {
        Location::fused(melior, &[location], self.attribute)
    }

    /// `location` with a fresh subprogram named `name` fused onto it, for a `sol.func` or
    /// `yul.func`: artificial when the frontend synthesized the function. `name` is what the source
    /// calls the function, which is the linkage name too.
    ///
    /// The result belongs on the function operation and nowhere else. An operation that borrowed
    /// it would be attributed to this subprogram, which the verifier rejects in another function.
    pub fn fuse_subprogram(
        self,
        name: &str,
        origin: FunctionOrigin,
        location: Location<'context>,
    ) -> Location<'context> {
        unsafe {
            Location::from_raw(crate::ffi::solxFuseSubprogram(
                self.attribute.to_raw(),
                name.as_ptr() as *const std::ffi::c_char,
                name.len(),
                location.to_raw(),
                origin == FunctionOrigin::Synthesized,
            ))
        }
    }
}
