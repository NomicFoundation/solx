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
;!         "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
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
;!         "0"
;!     ]
;! }, {
;!     "name": "one_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x29A44AC49A2C49A4AC81A482C81A482C81E6F9A1A21D59A1A21A1E661A1A1A21"
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
;!         "0x7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!     ]
;! }, {
;!     "name": "ordinary_bytes_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "40", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_bytes_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "40", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_bytes_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "40", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x00000000005348958934589349590349059034905903CDF343443AB34344343C"
;!     ]
;! }, {
;!     "name": "ordinary_bytes_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "40", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x0000000000FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!     ]
;! }, {
;!     "name": "ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "171", "0"
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
;!                 "171", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "171", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0xA6912B1268B12692B2069"
;!     ]
;! }, {
;!     "name": "ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "171", "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x0000000000000000000000000000000000000000001FFFFFFFFFFFFFFFFFFFFF"
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
;!         "0"
;!     ]
;! }, {
;!     "name": "255_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "255", "0x5348958934589349590349059034905903CDF343443AB34344343CCC34343443"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
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
;!         "1"
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
;!         "0"
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
;!         "0"
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
;!         "0"
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
;!         "0"
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
;!         "0"
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
;!         "0"
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
;!         "0"
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
;!         "0"
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
;!         "0"
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
  %res = call i256 @shr(i256 noundef %arg1, i256 noundef %arg2)
  store i256 %res, ptr addrspace(1) null, align 4
  call void @llvm.evm.return(ptr addrspace(1) null, i256 32)
  unreachable
}

define i256 @shr(i256 %shift, i256 %value) #0 {
entry:
  %is_overflow = icmp ugt i256 %shift, 255
  br i1 %is_overflow, label %return, label %shift_value

shift_value:
  %shift_res = lshr i256 %value, %shift
  br label %return

return:
  %res = phi i256 [ 0, %entry ], [ %shift_res, %shift_value ]
  ret i256 %res
}

declare void @llvm.evm.return(ptr addrspace(1), i256)
declare i256 @llvm.evm.calldataload(ptr addrspace(2))

attributes #0 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
