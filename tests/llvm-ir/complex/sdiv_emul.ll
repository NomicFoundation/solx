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
;!         "0"
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
;!         "0"
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
;!                 "3456789009876545678", "45116789098765678"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "76"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "6873546235472354672354762358492378590345034805934590348534", "6873546235472354672354762358492378590345034805934590348534"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_plus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x0b000000000000000000000000000000000000000000000000000000000000a1", "0x0b000000000000000000000000000000000000000000000000000000000000a2"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_ordinary_to_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "456789098765467892304234234234234234234", "7863249563247857289401203492047823764573465734573434537783"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
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
;!                 "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
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
;!                 "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!     ]
;! }, {
;!     "name": "ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "7437834752357434334343423343443375834785783474", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
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
;!                 "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "23784273472384723848213821342323233223"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "2434215393876951004058478270785519540778"
;!     ]
;! }, {
;!     "name": "max_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "zero_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "-1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_one_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-1", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_one_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-1", "-1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "zero_minus_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "-433478394034343"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_ordinary_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-121563127839120", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_one_minus_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-1", "-433478394034343"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_ordinary_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-121563127839120", "-1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "121563127839120"
;!     ]
;! }, {
;!     "name": "minus_ordinary_minus_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-3456789009876545678", "-45116789098765678"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "76"
;!     ]
;! }, {
;!     "name": "minus_ordinary_minus_ordinary_to_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-6873546235472354672354762358492378590345034805934590348534", "-6873546235472354672354762358492378590345034805934590348534"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "minus_ordinary_minus_ordinary_plus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-4975441334415930272106565762092058540570194653601742986070443062840017289377", "-4975441334415930272106565762092058540570194653601742986070443062840017289378"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_ordinary_minus_ordinary_to_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-456789098765467892304234234234234234234", "-7863249563247857289401203492047823764573465734573434537783"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "zero_min",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0", "0x8000000000000000000000000000000000000000000000000000000000000000"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "min_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x8000000000000000000000000000000000000000000000000000000000000000", "0"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_one_min",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-1", "0x8000000000000000000000000000000000000000000000000000000000000000"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "min_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x8000000000000000000000000000000000000000000000000000000000000000", "-1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x8000000000000000000000000000000000000000000000000000000000000000"
;!     ]
;! }, {
;!     "name": "minus_ordinary_min",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-7437834752357434334343423343443375834785783474", "0x8000000000000000000000000000000000000000000000000000000000000000"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "min_minus_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x8000000000000000000000000000000000000000000000000000000000000000", "-23784273472384723848213821342323233223"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "2434215393876951004058478270785519540778"
;!     ]
;! }, {
;!     "name": "min_min",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x8000000000000000000000000000000000000000000000000000000000000000", "0x8000000000000000000000000000000000000000000000000000000000000000"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "1"
;!     ]
;! }, {
;!     "name": "one_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "-1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-1"
;!     ]
;! }, {
;!     "name": "minus_one_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-1", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-1"
;!     ]
;! }, {
;!     "name": "one_minus_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "-433478394034343"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_one_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-1", "433478394034343"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "121563127839120", "-1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-121563127839120"
;!     ]
;! }, {
;!     "name": "minus_ordinary_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-121563127839120", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-121563127839120"
;!     ]
;! }, {
;!     "name": "ordinary_minus_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "3456789009876545678", "-45116789098765678"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-76"
;!     ]
;! }, {
;!     "name": "minus_ordinary_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-3456789009876545678", "45116789098765678"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-76"
;!     ]
;! }, {
;!     "name": "ordinary_minus_ordinary_to_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "6873546235472354672354762358492378590345034805934590348534", "-6873546235472354672354762358492378590345034805934590348534"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-1"
;!     ]
;! }, {
;!     "name": "minus_ordinary_ordinary_to_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-6873546235472354672354762358492378590345034805934590348534", "6873546235472354672354762358492378590345034805934590348534"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-1"
;!     ]
;! }, {
;!     "name": "ordinary_minus_ordinary_plus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "4975441334415930272106565762092058540570194653601742986070443062840017289377", "-4975441334415930272106565762092058540570194653601742986070443062840017289378"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_ordinary_ordinary_plus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-4975441334415930272106565762092058540570194653601742986070443062840017289377", "4975441334415930272106565762092058540570194653601742986070443062840017289378"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "ordinary_minus_ordinary_to_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "456789098765467892304234234234234234234", "-7863249563247857289401203492047823764573465734573434537783"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_ordinary_ordinary_to_zero",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-456789098765467892304234234234234234234", "7863249563247857289401203492047823764573465734573434537783"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "one_min",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "1", "0x8000000000000000000000000000000000000000000000000000000000000000"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_one_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-1", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_minus_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "-1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x8000000000000000000000000000000000000000000000000000000000000001"
;!     ]
;! }, {
;!     "name": "min_one",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x8000000000000000000000000000000000000000000000000000000000000000", "1"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0x8000000000000000000000000000000000000000000000000000000000000000"
;!     ]
;! }, {
;!     "name": "ordinary_min",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "7437834752357434334343423343443375834785783474", "0x8000000000000000000000000000000000000000000000000000000000000000"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "minus_ordinary_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "-7437834752357434334343423343443375834785783474", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "max_minus_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "-23784273472384723848213821342323233223"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-2434215393876951004058478270785519540778"
;!     ]
;! }, {
;!     "name": "min_ordinary",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x8000000000000000000000000000000000000000000000000000000000000000", "23784273472384723848213821342323233223"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-2434215393876951004058478270785519540778"
;!     ]
;! }, {
;!     "name": "max_min",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", "0x8000000000000000000000000000000000000000000000000000000000000000"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "0"
;!     ]
;! }, {
;!     "name": "min_max",
;!     "inputs": [
;!         {
;!             "method": "#fallback",
;!             "calldata": [
;!                 "0x8000000000000000000000000000000000000000000000000000000000000000", "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
;!             ]
;!         }
;!     ],
;!     "expected": [
;!         "-1"
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
  %res = call i256 @sdiv(i256 noundef %arg1, i256 noundef %arg2)
  store i256 %res, ptr addrspace(1) null, align 4
  call void @llvm.evm.return(ptr addrspace(1) null, i256 32)
  unreachable
}

define i256 @sdiv(i256 %arg1, i256 %arg2) #0 {
entry:
  %is_divider_zero = icmp eq i256 %arg2, 0
  br i1 %is_divider_zero, label %return, label %division_overflow

division_overflow:
  %is_divided_int_min = icmp eq i256 %arg1, -57896044618658097711785492504343953926634992332820282019728792003956564819968
  %is_minus_one = icmp eq i256 %arg2, -1
  %is_overflow = and i1 %is_divided_int_min, %is_minus_one
  br i1 %is_overflow, label %return, label %division

division:
  %div_res = sdiv i256 %arg1, %arg2
  br label %return

return:
  %res = phi i256 [ 0, %entry ], [ %arg1, %division_overflow ], [ %div_res, %division ]
  ret i256 %res
}

declare void @llvm.evm.return(ptr addrspace(1), i256)
declare i256 @llvm.evm.calldataload(ptr addrspace(2))

attributes #0 = { mustprogress nofree norecurse nosync nounwind readnone willreturn }
