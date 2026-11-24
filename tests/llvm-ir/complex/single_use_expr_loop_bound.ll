;! { "cases": [ {
;!     "name": "three_words",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                "0xDEADBEEF00000000000000000000000000000000000000000000000000000000",
;!                "128"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!                "4096",
;!                "0xDEADBEEF00000000000000000000000000000000000000000000000000000000"
;!     ]
;! } ] }

; Regression test for NomicFoundation/solx#960: the loop bound '%end' is
; computed before the loop from the decoder's SECOND result, and the loop
; pointer starts at the same value, so the register coalescer merges them.
; 'evm-single-use-expressions' must not rematerialize the bound inside the
; loop, or the comparison reads 'ptr < ptr + 131072' and the loop never ends.

target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

define i256 @__entry() noreturn {
entry:
  %pair = call { i256, i256 } @decoder()
  %seed = extractvalue { i256, i256 } %pair, 0
  %base = extractvalue { i256, i256 } %pair, 1
  %end = add i256 %base, 131072
  br label %loop

loop:
  %ptr = phi i256 [ %base, %entry ], [ %next, %loop ]
  %count = phi i256 [ 0, %entry ], [ %count.next, %loop ]
  %dst = inttoptr i256 %ptr to ptr addrspace(1)
  store i256 %count, ptr addrspace(1) %dst, align 1
  %next = add i256 %ptr, 32
  %count.next = add i256 %count, 1
  %continue = icmp ult i256 %next, %end
  br i1 %continue, label %loop, label %exit

exit:
  store i256 %count.next, ptr addrspace(1) null, align 1
  store i256 %seed, ptr addrspace(1) inttoptr (i256 32 to ptr addrspace(1)), align 1
  call void @llvm.evm.return(ptr addrspace(1) null, i256 64)
  unreachable
}

; Returns a checksum seed and the base offset, like the shared ABI decoder
; returns the decoded value's base pointer as a later result.
define private { i256, i256 } @decoder() noinline {
  %seed = call i256 @llvm.evm.calldataload(ptr addrspace(2) null)
  %base = call i256 @llvm.evm.calldataload(ptr addrspace(2) inttoptr (i256 32 to ptr addrspace(2)))
  %pair0 = insertvalue { i256, i256 } poison, i256 %seed, 0
  %pair = insertvalue { i256, i256 } %pair0, i256 %base, 1
  ret { i256, i256 } %pair
}

declare i256 @llvm.evm.calldataload(ptr addrspace(2))
declare void @llvm.evm.return(ptr addrspace(1), i256)
