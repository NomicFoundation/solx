source_filename = "LLVMDialectModule"
target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

define void @__entry() local_unnamed_addr #0 !dbg !6 {
  %1 = tail call i256 @llvm.evm.datasize(metadata !9), !dbg !10
  %2 = tail call i256 @llvm.evm.codesize(), !dbg !10
  %3 = sub i256 %2, %1, !dbg !10
  %4 = add i256 %3, 31, !dbg !11
  %5 = and i256 %4, -32, !dbg !11
  %6 = add i256 %5, 128, !dbg !11
  %7 = icmp ugt i256 %6, 18446744073709551615, !dbg !11
  %8 = icmp ugt i256 %4, -129, !dbg !11
  %9 = or i1 %8, %7, !dbg !11
  br i1 %9, label %10, label %__sol.alloc.exit, !dbg !11

10:
  store i256 35408467139433450592217433187231851964531694900788300625387963629091585785856, ptr addrspace(1) null, align 4294967296, !dbg !14
  store i256 65, ptr addrspace(1) inttoptr (i256 4 to ptr addrspace(1)), align 4, !dbg !14
  tail call void @llvm.evm.revert(ptr addrspace(1) null, i256 36), !dbg !14
  unreachable, !dbg !14

__sol.alloc.exit:
  store i256 %6, ptr addrspace(1) inttoptr (i256 64 to ptr addrspace(1)), align 64, !dbg !11
  %11 = inttoptr i256 %1 to ptr addrspace(4), !dbg !10
  tail call void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) nonnull align 128 inttoptr (i256 128 to ptr addrspace(1)), ptr addrspace(4) align 1 %11, i256 %3, i1 false), !dbg !10
  %12 = tail call i256 @llvm.evm.dataoffset(metadata !17), !dbg !18
  %13 = tail call i256 @llvm.evm.datasize(metadata !17), !dbg !18
  %14 = inttoptr i256 %6 to ptr addrspace(1), !dbg !18
  %15 = inttoptr i256 %12 to ptr addrspace(4), !dbg !18
  tail call void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) align 32 %14, ptr addrspace(4) align 1 %15, i256 %13, i1 false), !dbg !18
  tail call void @llvm.evm.return(ptr addrspace(1) %14, i256 %13), !dbg !18
  unreachable, !dbg !18
}

declare i256 @llvm.evm.datasize(metadata) #1

declare i256 @llvm.evm.codesize() #1

declare void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) noalias writeonly captures(none), ptr addrspace(4) noalias readonly captures(none), i256, i1 immarg) #2

declare i256 @llvm.evm.dataoffset(metadata) #1

declare void @llvm.evm.return(ptr addrspace(1) readonly, i256) #3

declare void @llvm.evm.revert(ptr addrspace(1) readonly, i256) #4

attributes #0 = { nofree noreturn nounwind null_pointer_is_valid memory(readwrite, inaccessiblemem: read) "evm-entry-function" }
attributes #1 = { mustprogress nocallback nofree nosync nounwind willreturn memory(none) }
attributes #2 = { mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #3 = { nofree noreturn nounwind memory(read) }
attributes #4 = { nofree noreturn nounwind memory(argmem: read) }

!llvm.dbg.cu = !{!0}
!llvm.module.flags = !{!3, !4, !5}

!0 = distinct !DICompileUnit(language: DW_LANG_Assembly, file: !1, isOptimized: true, runtimeVersion: 0, emissionKind: FullDebug)
!1 = !DIFile(filename: "SOURCE_FILE", directory: "")
!3 = !{i32 1, !"evm-memory-guard", i32 128}
!4 = !{i32 1, !"evm-stack-region-size", i32 0}
!5 = !{i32 2, !"Debug Info Version", i32 3}
!6 = distinct !DISubprogram(name: "__entry", linkageName: "__entry", scope: !1, file: !1, line: CONTRACT_LINE, type: !7, scopeLine: CONTRACT_LINE, flags: DIFlagArtificial, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!7 = !DISubroutineType(types: !8)
!8 = !{}
!9 = !{!"DEPLOY_OBJECT"}
!10 = !DILocation(line: CONSTRUCTOR_LINE, column: CONSTRUCTOR_COLUMN, scope: !6)
!11 = !DILocation(line: 0, scope: !12, inlinedAt: !13)
!12 = distinct !DISubprogram(name: "__sol.alloc", linkageName: "__sol.alloc", scope: !1, file: !1, line: CONTRACT_LINE, type: !7, scopeLine: CONTRACT_LINE, flags: DIFlagArtificial, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!13 = distinct !DILocation(line: CONSTRUCTOR_LINE, column: CONSTRUCTOR_COLUMN, scope: !6)
!14 = !DILocation(line: 0, scope: !15, inlinedAt: !16)
!15 = distinct !DISubprogram(name: "__sol.panic", linkageName: "__sol.panic", scope: !1, file: !1, line: CONTRACT_LINE, type: !7, scopeLine: CONTRACT_LINE, flags: DIFlagArtificial, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!16 = distinct !DILocation(line: 0, scope: !12, inlinedAt: !13)
!17 = !{!"RUNTIME_OBJECT"}
!18 = !DILocation(line: CONTRACT_LINE, column: CONTRACT_COLUMN, scope: !6)
