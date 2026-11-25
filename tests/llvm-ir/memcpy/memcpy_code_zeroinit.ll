;! { "cases": [ {
;!     "name": "zeros_in_code",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!             ]
;!         }
;!     ],
;!     "expected": [
;!       "0x0000000000000000000000000000000000000000000000000000000000000000",
;!       "0x0000000000000000000000000000000000000000000000000000000000000000",
;!       "0x0000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
;!       "0xdeadbeef99ffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!     ]
;! } ] }

target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

@zeros = private unnamed_addr addrspace(4) constant [66 x i8] zeroinitializer
@marker = private unnamed_addr addrspace(4) constant [5 x i8] c"\DE\AD\BE\EF\99"

; Function Attrs: noreturn nounwind
define i256 @__entry() noreturn {
entry:
  store i256 -1, ptr addrspace(1) null, align 32
  %p32 = inttoptr i256 32 to ptr addrspace(1)
  store i256 -1, ptr addrspace(1) %p32, align 32
  %p64 = inttoptr i256 64 to ptr addrspace(1)
  store i256 -1, ptr addrspace(1) %p64, align 32
  %p96 = inttoptr i256 96 to ptr addrspace(1)
  store i256 -1, ptr addrspace(1) %p96, align 32
  call void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) null, ptr addrspace(4) @zeros, i256 66, i1 false)
  call void @llvm.memcpy.p1.p4.i256(ptr addrspace(1) %p96, ptr addrspace(4) @marker, i256 5, i1 false)
  call void @llvm.evm.return(ptr addrspace(1) null, i256 128)
  unreachable
}

declare void @llvm.memcpy.p1.p4.i256(ptr addrspace(1), ptr addrspace(4), i256, i1)
declare void @llvm.evm.return(ptr addrspace(1), i256)
