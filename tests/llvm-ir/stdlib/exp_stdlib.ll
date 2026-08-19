;! { "cases": [ {
;!     "name": "zero_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "zero_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "one_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "zero_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "433478394034343"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "121563127839120", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "one_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "433478394034343"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "ordinary_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "121563127839120", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "121563127839120"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "21", "52"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "569381465857367090636427305760163241950353347303833610101782245331441"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_big",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x00000000000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "2"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xfffffffffffffffffffffffffffffffe00000000000000000000000000000001"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_min_overflow",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x0000000000000000000000000000000000000000000000000000000000010000", "16"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_overflow",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x2fff1ffffffffff5ffffff0fffffffff2ffffffafffafffcffff1ff234ffffff", "0xef231ffffffffff4f12fff34ffffffff2fffffbbfffafffdffff22f538ffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "46756260758475007021788099943083655901358133181480408838873172916982662561791"
;!     ]
;! }, {
;!     "name": "zero_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "max_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!     ]
;! }, {
;!     "name": "ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "7437834752357434334343423343443375834785783474", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "23784273472384723848213821342323233223"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "115792089237316195423570985008687907853269984665640564039457584007913129639935"
;!     ]
;! }, {
;!     "name": "max_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!     ]
;! } ] }


target datalayout = "E-p:256:256-i256:256:256-S256-a:256:256"
target triple = "evm-unknown-unknown"

define void @test() noreturn {
bb:
  %off1 = inttoptr i256 0 to ptr addrspace(2)
  %off2 = inttoptr i256 32 to ptr addrspace(2)
  %arg1 = call i256 @llvm.evm.calldataload(ptr addrspace(2) %off1)
  %arg2 = call i256 @llvm.evm.calldataload(ptr addrspace(2) %off2)
  %res = call i256 @__exp(i256 noundef %arg1, i256 noundef %arg2)
  store i256 %res, ptr addrspace(1) null, align 4
  call void @llvm.evm.return(ptr addrspace(1) null, i256 32)
  unreachable
}

declare i256 @__exp(i256 %arg1, i256 %arg2) #0
declare void @llvm.evm.return(ptr addrspace(1), i256)
declare i256 @llvm.evm.calldataload(ptr addrspace(2))

attributes #0 = { alwaysinline mustprogress nofree norecurse nosync nounwind readnone willreturn }
