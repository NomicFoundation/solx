//!
//! The LLVM debug information.
//!

use inkwell::debug_info::AsDIScope;
use num::Zero;

use crate::{codegen::context::Context, IContext};

///
/// The LLVM debug information.
///
pub struct DebugInfo<'ctx> {
    /// The compile unit.
    compile_unit: inkwell::debug_info::DICompileUnit<'ctx>,
    /// The debug info builder.
    builder: inkwell::debug_info::DebugInfoBuilder<'ctx>,
}

impl<'ctx> DebugInfo<'ctx> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(module: &inkwell::module::Module<'ctx>, filename: &str) -> Self {
        let (builder, compile_unit) = module.create_debug_info_builder(
            true,
            inkwell::debug_info::DWARFSourceLanguage::Assembly,
            filename,
            std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_str()
                .unwrap_or_default(),
            "",
            false,
            "",
            0,
            "",
            inkwell::debug_info::DWARFEmissionKind::Full,
            0,
            false,
            false,
            "",
            "",
        );

        Self {
            compile_unit,
            builder,
        }
    }

    ///
    /// Creates a function info.
    ///
    /// If `is_artificial` is true, the function does not come from Solidity source code,
    /// and marked as artificial in the debug info.
    ///
    pub fn create_function(
        &self,
        name: &str,
        line: usize,
        is_artificial: bool,
    ) -> inkwell::debug_info::DISubprogram<'ctx> {
        let subroutine_type = self.builder.create_subroutine_type(
            self.compile_unit.get_file(),
            None,
            &[],
            inkwell::debug_info::DIFlags::zero(),
        );

        let mut flags = inkwell::debug_info::DIFlags::zero();
        if is_artificial {
            flags |= llvm_sys::debuginfo::LLVMDIFlagArtificial;
        }

        self.builder.create_function(
            self.compile_unit.get_file().as_debug_info_scope(),
            name,
            None,
            self.compile_unit.get_file(),
            line as u32,
            subroutine_type,
            true,
            true,
            line as u32,
            flags,
            false,
        )
    }

    ///
    /// Creates a location.
    ///
    pub fn create_location(
        &self,
        context: &Context<'ctx>,
        line: usize,
        column: usize,
    ) -> Option<inkwell::debug_info::DILocation<'ctx>> {
        let subprogram = context
            .current_function()
            .borrow()
            .declaration()
            .value
            .get_subprogram()?;
        Some(self.builder.create_debug_location(
            context.llvm(),
            line as u32,
            column as u32,
            subprogram.as_debug_info_scope(),
            None,
        ))
    }

    ///
    /// Finalizes the builder.
    ///
    pub fn finalize(&self, context: &Context<'ctx>) {
        context.module().add_basic_value_flag(
            "Dwarf Version",
            inkwell::module::FlagBehavior::Warning,
            context.integer_const(solx_utils::BIT_LENGTH_X32, 5),
        );
        context.module().add_basic_value_flag(
            "Debug Info Version",
            inkwell::module::FlagBehavior::Warning,
            context.integer_const(solx_utils::BIT_LENGTH_X32, 3),
        );

        self.builder.finalize();
    }
}
