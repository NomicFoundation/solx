; ModuleID = 'Simple.sol:Simple.runtime'
source_filename = "Simple.sol:Simple.runtime"
target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

; Function Attrs: mustprogress nofree nosync nounwind willreturn memory(none)
declare i256 @llvm.evm.calldatasize() #0

; Function Attrs: mustprogress nofree nosync nounwind willreturn memory(none)
declare i256 @llvm.evm.callvalue() #0

; Function Attrs: nofree noreturn nounwind memory(read)
declare void @llvm.evm.return(ptr addrspace(1) readonly, i256) #1

; Function Attrs: nofree noreturn nounwind memory(argmem: read)
declare void @llvm.evm.revert(ptr addrspace(1) readonly, i256) #2

; Function Attrs: nofree noreturn nounwind null_pointer_is_valid memory(readwrite, inaccessiblemem: read)
define void @__entry() local_unnamed_addr #3 {
entry:
  %callvalue = tail call i256 @llvm.evm.callvalue()
  %comparison_result = icmp eq i256 %callvalue, 0
  br i1 %comparison_result, label %"block_rt_1/0", label %"block_rt_2/0"

"block_rt_1/0":                                   ; preds = %entry
  %calldatasize = tail call i256 @llvm.evm.calldatasize()
  %comparison_result1 = icmp ult i256 %calldatasize, 4
  br i1 %comparison_result1, label %"block_rt_2/0", label %conditional_rt_2_join_block

"block_rt_2/0":                                   ; preds = %conditional_rt_2_join_block, %entry, %"block_rt_1/0"
  tail call void @llvm.evm.revert(ptr addrspace(1) noalias nofree noundef nonnull align 32 captures(none) null, i256 0)
  unreachable

"block_rt_3/0":                                   ; preds = %conditional_rt_2_join_block
  store i256 1, ptr addrspace(1) inttoptr (i256 128 to ptr addrspace(1)), align 128
  tail call void @llvm.evm.return(ptr addrspace(1) noalias nofree noundef nonnull align 32 captures(none) inttoptr (i256 128 to ptr addrspace(1)), i256 32)
  unreachable

"block_rt_4/0":                                   ; preds = %conditional_rt_2_join_block
  store i256 2, ptr addrspace(1) inttoptr (i256 128 to ptr addrspace(1)), align 128
  tail call void @llvm.evm.return(ptr addrspace(1) noalias nofree noundef nonnull align 32 captures(none) inttoptr (i256 128 to ptr addrspace(1)), i256 32)
  unreachable

conditional_rt_2_join_block:                      ; preds = %"block_rt_1/0"
  %calldata_load_result = load i256, ptr addrspace(2) null, align 4294967296
  %shift_right_non_overflow_result = lshr i256 %calldata_load_result, 224
  %trunc = trunc nuw i256 %shift_right_non_overflow_result to i32
  switch i32 %trunc, label %"block_rt_2/0" [
    i32 1039457780, label %"block_rt_3/0"
    i32 1519042605, label %"block_rt_4/0"
  ]
}

attributes #0 = { mustprogress nofree nosync nounwind willreturn memory(none) }
attributes #1 = { nofree noreturn nounwind memory(read) }
attributes #2 = { nofree noreturn nounwind memory(argmem: read) }
attributes #3 = { nofree noreturn nounwind null_pointer_is_valid memory(readwrite, inaccessiblemem: read) "evm-entry-function" "target-features"="+osaka" }
