;! { "cases": [ {
;!     "name": "0",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!             ]
;!         }
;!     ],
;!     "expected": [
;!       "*",
;!       "*"
;!     ]
;! } ] }

target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

; Function Attrs: noreturn nounwind
define i256 @__entry() noreturn {
runtime:
  %size = call i256 @llvm.evm.codesize()
  call void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) align 1 null, ptr addrspace(4) align 1 null, i256 %size, i1 false)
  call void @llvm.evm.return(ptr addrspace(1) null, i256 %size)
  unreachable
}

declare i256 @llvm.evm.codesize()
declare void @llvm.memcpy.p1.p4.i256(ptr addrspace(1), ptr addrspace(4), i256, i1)
declare void @llvm.evm.return(ptr addrspace(1), i256)
