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
;!         "0"
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
;!         "1"
;!     ]
;! }, {
;!     "name": "zero_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x43"
;!     ]
;! }, {
;!     "name": "zero_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
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
;!         "0"
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
;!     "name": "one_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34348443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8443"
;!     ]
;! }, {
;!     "name": "one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!     ]
;! }, {
;!     "name": "ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "17", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "4", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_positive",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "22", "0x5F21235A343BB4343C1A8543545FF3443340904201D8AC"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x5F21235A343BB4343C1A8543545FF3443340904201D8AC"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_big_positive",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "17", "0x121B4343C1A8543545FF3443340904201D8AC"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x21B4343C1A8543545FF3443340904201D8AC"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_negative",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "22", "0xAF21235A343BB4343C1A8543545FF3443340904201D8AC"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFAF21235A343BB4343C1A8543545FF3443340904201D8AC"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_big_negative",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "17", "0xE34F4334AB343443DDC1B4343C1A8543545FF3443340904201D8AC"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFC1B4343C1A8543545FF3443340904201D8AC"
;!     ]
;! }, {
;!     "name": "ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "25", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!     ]
;! }, {
;!     "name": "255_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "255", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "255_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "255", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "255_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "255", "0x9348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x9348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!     ]
;! }, {
;!     "name": "255_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "255", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!     ]
;! }, {
;!     "name": "256_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "256", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "256_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "256", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "256_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "256", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!     ]
;! }, {
;!     "name": "256_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "256", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!     ]
;! }, {
;!     "name": "big_ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "35242523534534534233424343343443", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "big_ordinary_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "35242523534534534233424343343443", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "big_ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "35242523534534534233424343343443", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!     ]
;! }, {
;!     "name": "big_ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "35242523534534534233424343343443", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!     ]
;! }, {
;!     "name": "max_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "max_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!     ]
;! }, {
;!     "name": "max_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
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
  %res = call i256 @__signextend(i256 noundef %arg1, i256 noundef %arg2)
  store i256 %res, ptr addrspace(1) null, align 4
  call void @llvm.evm.return(ptr addrspace(1) null, i256 32)
  unreachable
}

declare i256 @__signextend(i256 %arg1, i256 %arg2) #0
declare void @llvm.evm.return(ptr addrspace(1), i256)
declare i256 @llvm.evm.calldataload(ptr addrspace(2))

attributes #0 = { alwaysinline mustprogress nofree norecurse nosync nounwind readnone willreturn }
