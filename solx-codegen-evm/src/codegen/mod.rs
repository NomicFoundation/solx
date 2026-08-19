//!
//! The LLVM EVM context library.
//!

pub mod attribute;
pub mod build;
pub mod context;
pub mod instructions;
pub mod profiler;
pub mod warning;

use std::collections::BTreeMap;

use self::context::Context;

///
/// Initializes the EVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_evm(&inkwell::targets::InitializationConfig::default());
}

///
/// Appends metadata to runtime code.
///
pub fn append_metadata(
    bytecode_buffer: inkwell::memory_buffer::MemoryBuffer,
    metadata: &[u8],
) -> anyhow::Result<inkwell::memory_buffer::MemoryBuffer> {
    if metadata.is_empty() {
        return Ok(bytecode_buffer);
    }

    bytecode_buffer
        .append_metadata_evm(metadata)
        .map_err(|error| anyhow::anyhow!("bytecode metadata appending error: {error}"))
}

///
/// Assembles the main buffer and its dependencies from `bytecode_buffers`.
///
pub fn assemble(
    bytecode_buffers: &[&inkwell::memory_buffer::MemoryBuffer],
    bytecode_buffer_ids: &[&str],
    code_segment: solx_utils::CodeSegment,
) -> anyhow::Result<inkwell::memory_buffer::MemoryBuffer> {
    let code_segment = match code_segment {
        solx_utils::CodeSegment::Deploy => inkwell::memory_buffer::CodeSegment::Deploy,
        solx_utils::CodeSegment::Runtime => inkwell::memory_buffer::CodeSegment::Runtime,
    };
    inkwell::memory_buffer::MemoryBuffer::assemble_evm(
        bytecode_buffers,
        bytecode_buffer_ids,
        code_segment,
    )
    .map_err(|error| anyhow::anyhow!("linking: {error}"))
}

///
/// Links `bytecode_buffer` with `linker_symbols`.
///
pub fn link(
    bytecode_buffer: inkwell::memory_buffer::MemoryBuffer,
    linker_symbols: &BTreeMap<String, [u8; solx_utils::BYTE_LENGTH_ETH_ADDRESS]>,
) -> anyhow::Result<inkwell::memory_buffer::MemoryBuffer> {
    let bytecode_buffer_linked = bytecode_buffer
        .link_evm(linker_symbols)
        .map_err(|error| anyhow::anyhow!("linking: {error}"))?;

    Ok(bytecode_buffer_linked)
}

///
/// Returns minimal deploy code patched with specified identifiers.
///
pub fn minimal_deploy_code(deploy_code_identifier: &str, runtime_code_identifier: &str) -> String {
    format!(
        r#"
; ModuleID = '{deploy_code_identifier}'
source_filename = "{deploy_code_identifier}"
target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

; Function Attrs: mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i256 @llvm.evm.datasize(metadata) #0

; Function Attrs: mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none)
declare i256 @llvm.evm.dataoffset(metadata) #0

; Function Attrs: noreturn nounwind
declare void @llvm.evm.return(ptr addrspace(1), i256) #1

; Function Attrs: mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite)
declare void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) noalias nocapture writeonly, ptr addrspace(4) noalias nocapture readonly, i256, i1 immarg) #2

; Function Attrs: nofree noreturn null_pointer_is_valid
define void @__entry() local_unnamed_addr #3 {{
entry:
  %datasize = tail call i256 @llvm.evm.datasize(metadata !0)
  %dataoffset = tail call i256 @llvm.evm.dataoffset(metadata !0)
  %codecopy_source_pointer = inttoptr i256 %dataoffset to ptr addrspace(4)
  tail call void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) align 4294967296 null, ptr addrspace(4) align 1 %codecopy_source_pointer, i256 %datasize, i1 false)
  tail call void @llvm.evm.return(ptr addrspace(1) noalias nocapture nofree noundef nonnull align 32 null, i256 %datasize)
  unreachable
}}

attributes #0 = {{ mustprogress nocallback nofree nosync nounwind speculatable willreturn memory(none) }}
attributes #1 = {{ noreturn nounwind }}
attributes #2 = {{ mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite) }}
attributes #3 = {{ nofree noreturn null_pointer_is_valid }}

!0 = !{{!"{runtime_code_identifier}"}}
"#
    )
}

///
/// Implemented by items which are translated into LLVM IR.
///
pub trait WriteLLVM {
    ///
    /// Declares the entity in the LLVM IR.
    /// Is usually performed in order to use the item before defining it.
    ///
    fn declare(&mut self, _context: &mut Context) -> anyhow::Result<()> {
        Ok(())
    }

    ///
    /// Translates the entity into LLVM IR.
    ///
    fn into_llvm(self, context: &mut Context) -> anyhow::Result<()>;
}

///
/// The dummy LLVM writable entity.
///
#[derive(Debug, Default, Clone)]
pub struct DummyLLVMWritable {}

impl WriteLLVM for DummyLLVMWritable {
    fn into_llvm(self, _context: &mut Context) -> anyhow::Result<()> {
        Ok(())
    }
}
