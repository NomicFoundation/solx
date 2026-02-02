//!
//! The LLVM debug information.
//!

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;

use inkwell::debug_info::AsDIScope;
use num::Zero;

use crate::codegen::context::Context;
use crate::context::IContext;

///
/// The LLVM debug information.
///
pub struct DebugInfo<'ctx> {
    /// The debug info builder.
    builder: inkwell::debug_info::DebugInfoBuilder<'ctx>,
    /// The main compile unit.
    /// Directory of the current translation unit. Stored to prevent memory freeing.
    _directory: PathBuf,
    /// The files used for the current translation unit.
    files: HashMap<usize, inkwell::debug_info::DIFile<'ctx>>,
}

impl<'ctx> DebugInfo<'ctx> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(module: &inkwell::module::Module<'ctx>, sources: &BTreeMap<usize, String>) -> Self {
        let directory = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        let (main_source_id, main_source_path) = sources.iter().next().expect("Always exists");
        let (builder, compile_unit) = module.create_debug_info_builder(
            true,
            inkwell::debug_info::DWARFSourceLanguage::Assembly,
            main_source_path.as_str(),
            directory.to_str().expect("Always valid"),
            "",
            true,
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

        let mut files = sources
            .iter()
            .skip(1)
            .map(|(source_id, path)| {
                (
                    source_id.to_owned(),
                    builder.create_file(path.as_str(), directory.to_str().expect("Always valid")),
                )
            })
            .collect::<HashMap<usize, inkwell::debug_info::DIFile<'ctx>>>();
        files.insert(*main_source_id, compile_unit.get_file());

        Self {
            builder,
            _directory: directory,
            files,
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
        source_id: usize,
        line: usize,
        is_artificial: bool,
    ) -> inkwell::debug_info::DISubprogram<'ctx> {
        let file = self
            .files
            .get(&source_id)
            .copied()
            .expect("Source ID not found in debug info");
        let subroutine_type = self.builder.create_subroutine_type(
            file,
            None,
            &[],
            inkwell::debug_info::DIFlags::zero(),
        );

        let mut flags = inkwell::debug_info::DIFlags::zero();
        if is_artificial {
            flags |= llvm_sys::debuginfo::LLVMDIFlagArtificial;
        }

        self.builder.create_function(
            file.as_debug_info_scope(),
            name,
            None,
            file,
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
