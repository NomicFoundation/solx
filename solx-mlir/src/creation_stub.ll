source_filename = "LLVMDialectModule"
target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

define void @__entry() local_unnamed_addr #0 !dbg !6 {
  %1 = tail call i256 @llvm.evm.callvalue(), !dbg !9
  %.not = icmp eq i256 %1, 0, !dbg !9
  br i1 %.not, label %3, label %2, !dbg !9

2:
  tail call void @llvm.evm.revert(ptr addrspace(1) null, i256 0), !dbg !9
  unreachable, !dbg !9

3:
  %4 = tail call i256 @llvm.evm.datasize(metadata !10), !dbg !9
  %5 = tail call i256 @llvm.evm.codesize(), !dbg !9
  %6 = sub i256 %5, %4, !dbg !9
  %7 = add i256 %6, 31, !dbg !11
  %8 = and i256 %7, -32, !dbg !11
  %9 = add i256 %8, 128, !dbg !11
  %10 = icmp ugt i256 %9, 18446744073709551615, !dbg !11
  %11 = icmp ugt i256 %7, -129, !dbg !11
  %12 = or i1 %11, %10, !dbg !11
  br i1 %12, label %13, label %__sol.alloc.exit, !dbg !11

13:
  store i256 35408467139433450592217433187231851964531694900788300625387963629091585785856, ptr addrspace(1) null, align 4294967296, !dbg !14
  store i256 65, ptr addrspace(1) inttoptr (i256 4 to ptr addrspace(1)), align 4, !dbg !14
  tail call void @llvm.evm.revert(ptr addrspace(1) null, i256 36), !dbg !14
  unreachable, !dbg !14

__sol.alloc.exit:
  store i256 %9, ptr addrspace(1) inttoptr (i256 64 to ptr addrspace(1)), align 64, !dbg !11
  %14 = inttoptr i256 %4 to ptr addrspace(4), !dbg !9
  tail call void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) nonnull align 128 inttoptr (i256 128 to ptr addrspace(1)), ptr addrspace(4) align 1 %14, i256 %6, i1 false), !dbg !9
  %15 = tail call i256 @llvm.evm.dataoffset(metadata !17), !dbg !18
  %16 = tail call i256 @llvm.evm.datasize(metadata !17), !dbg !18
  %17 = inttoptr i256 %9 to ptr addrspace(1), !dbg !18
  %18 = inttoptr i256 %15 to ptr addrspace(4), !dbg !18
  tail call void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) align 32 %17, ptr addrspace(4) align 1 %18, i256 %16, i1 false), !dbg !18
  tail call void @llvm.evm.return(ptr addrspace(1) %17, i256 %16), !dbg !18
  unreachable, !dbg !18
}

declare i256 @llvm.evm.callvalue() #1

declare void @llvm.evm.revert(ptr addrspace(1) readonly, i256) #2

declare i256 @llvm.evm.datasize(metadata) #3

declare i256 @llvm.evm.codesize() #3

declare void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) noalias writeonly captures(none), ptr addrspace(4) noalias readonly captures(none), i256, i1 immarg) #4

declare i256 @llvm.evm.dataoffset(metadata) #3

declare void @llvm.evm.return(ptr addrspace(1) readonly, i256) #5

attributes #0 = { nofree noreturn nounwind null_pointer_is_valid memory(readwrite, inaccessiblemem: read) "evm-entry-function" }
attributes #1 = { mustprogress nofree nosync nounwind willreturn memory(none) }
attributes #2 = { nofree noreturn nounwind memory(argmem: read) }
attributes #3 = { mustprogress nocallback nofree nosync nounwind willreturn memory(none) }
attributes #4 = { mustprogress nocallback nofree nounwind willreturn memory(argmem: readwrite) }
attributes #5 = { nofree noreturn nounwind memory(read) }

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
!9 = !DILocation(line: CONSTRUCTOR_LINE, column: CONSTRUCTOR_COLUMN, scope: !6)
!10 = !{!"DEPLOY_OBJECT"}
!11 = !DILocation(line: 0, scope: !12, inlinedAt: !13)
!12 = distinct !DISubprogram(name: "__sol.alloc", linkageName: "__sol.alloc", scope: !1, file: !1, line: CONTRACT_LINE, type: !7, scopeLine: CONTRACT_LINE, flags: DIFlagArtificial, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!13 = distinct !DILocation(line: CONSTRUCTOR_LINE, column: CONSTRUCTOR_COLUMN, scope: !6)
!14 = !DILocation(line: 0, scope: !15, inlinedAt: !16)
!15 = distinct !DISubprogram(name: "__sol.panic", linkageName: "__sol.panic", scope: !1, file: !1, line: CONTRACT_LINE, type: !7, scopeLine: CONTRACT_LINE, flags: DIFlagArtificial, spFlags: DISPFlagLocalToUnit | DISPFlagDefinition, unit: !0)
!16 = distinct !DILocation(line: 0, scope: !12, inlinedAt: !13)
!17 = !{!"RUNTIME_OBJECT"}
!18 = !DILocation(line: CONTRACT_LINE, column: CONTRACT_COLUMN, scope: !6)
