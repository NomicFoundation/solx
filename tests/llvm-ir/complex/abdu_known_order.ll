;! { "cases": [ {
;!     "name": "high_diff",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x0000000000000000000000000000000000000000000000000000000000000003",
;!                 "0xc000000000000000000000000000000000000000000000000000000000000000"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xbfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!     ]
;! } ] }

target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

; Function Attrs: noreturn nounwind
define i256 @__entry() noreturn {
entry:
  %xptr = inttoptr i256 0 to ptr addrspace(2)
  %x = call i256 @llvm.evm.calldataload(ptr addrspace(2) %xptr)
  %yptr = inttoptr i256 32 to ptr addrspace(2)
  %y = call i256 @llvm.evm.calldataload(ptr addrspace(2) %yptr)
  %l = lshr i256 %x, 1
  %r = or i256 %y, 57896044618658097711785492504343953926634992332820282019728792003956564819968
  %c = icmp ugt i256 %l, %r
  %s1 = sub i256 %l, %r
  %s2 = sub i256 %r, %l
  %res = select i1 %c, i256 %s1, i256 %s2
  store i256 %res, ptr addrspace(1) null, align 32
  call void @llvm.evm.return(ptr addrspace(1) null, i256 32)
  unreachable
}

declare i256 @llvm.evm.calldataload(ptr addrspace(2))
declare void @llvm.evm.return(ptr addrspace(1), i256)
