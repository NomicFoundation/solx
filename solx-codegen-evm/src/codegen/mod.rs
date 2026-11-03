//!
//! The LLVM EVM context library.
//!

pub mod build;
pub mod context;

use std::sync::atomic::AtomicBool;

///
/// Initializes the EVM target machine.
///
pub fn initialize_target() {
    inkwell::targets::Target::initialize_evm(&inkwell::targets::InitializationConfig::default());
}

/// Whether the size fallback is activated during the compilation.
/// Only set once, as we're only compiling one traslation unit in a process.
pub static IS_SIZE_FALLBACK: AtomicBool = AtomicBool::new(false);

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
